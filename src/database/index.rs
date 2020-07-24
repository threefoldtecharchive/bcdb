use super::*;
use crate::storage::Storage;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::prelude::*;
use sqlx::SqlitePool;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::task::spawn_blocking;

pub struct SqliteIndexBuilder {
    root: String,
}

impl SqliteIndexBuilder {
    pub fn new<P>(root: P) -> Result<SqliteIndexBuilder>
    where
        P: Into<String>,
    {
        let root = root.into();
        std::fs::create_dir_all(&root)?;

        Ok(SqliteIndexBuilder { root: root })
    }

    pub async fn build(&self, collection: &str) -> Result<SqliteIndex> {
        if collection.len() == 0 {
            bail!("collection name must not be empty");
        }
        let p = std::path::PathBuf::from(&self.root).join(format!("{}.sqlite", collection));

        let p = match p.to_str() {
            Some(p) => p,
            None => bail!("empty path to db"),
        };

        let store = SqliteIndex::new(&format!("sqlite://{}", p)).await?;
        Ok(store)
    }
}

#[derive(Clone)]
pub struct SqliteIndex {
    schema: Schema,
}

impl SqliteIndex {
    async fn new(collection: &str) -> Result<Self> {
        let pool = SqlitePool::new(collection).await?;
        let schema = Schema::new(pool);
        schema.setup().await?;

        Ok(SqliteIndex { schema })
    }
}

#[async_trait]
impl Index for SqliteIndex {
    async fn set(&self, key: Key, meta: Meta) -> Result<()> {
        if meta.deleted() {
            // the deleted flag was set on the meta
            // then we need to delete the object from
            // the schema database.
            self.schema.delete_key(key).await?;
        } else {
            self.schema.insert(key, meta).await?;
        }

        Ok(())
    }

    async fn get(&self, key: Key) -> Result<Meta> {
        self.schema.get(key).await
    }

    async fn find(&self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>> {
        self.schema.find(meta).await
    }
}

#[derive(Clone)]
struct Schema {
    // so while the Pool should allow you to execute multiple
    // operations in parallel but it seems that sqlite itself
    // has some limitations. You can have a write and read operation
    // running at the same time. Hence we still wrap the pool in a
    // RWLock
    c: Arc<RwLock<SqlitePool>>,
}

impl Schema {
    fn new(c: SqlitePool) -> Schema {
        Schema {
            c: Arc::new(RwLock::new(c)),
        }
    }

    async fn setup(&self) -> Result<()> {
        let db = self.c.write().await;
        sqlx::query(
            "
        CREATE TABLE IF NOT EXISTS metadata (
            key INT,
            tag TEXT,
            value TEXT
        );

        CREATE UNIQUE INDEX IF NOT EXISTS metadata_unique ON metadata (key, tag);
        CREATE INDEX IF NOT EXISTS metadata_value ON metadata (value);
        ",
        )
        .execute(db.deref())
        .await?;

        Ok(())
    }

    async fn insert(&self, key: Key, tags: Meta) -> Result<()> {
        let db = self.c.write().await;
        for (k, v) in tags {
            sqlx::query(
                "
                INSERT INTO metadata (key, tag, value) values
                (?, ?, ?)
                ON CONFLICT (key, tag)
                DO UPDATE SET value = ?;
                ",
            )
            .bind(key as f64) // only f64 is supported by sqlite.
            .bind(&k)
            .bind(&v)
            .bind(&v)
            .execute(db.deref())
            .await
            .context("failed to insert data to index")?;
        }

        Ok(())
    }

    async fn get(&self, key: Key) -> Result<Meta> {
        let db = self.c.read().await;
        let mut cur = sqlx::query("SELECT tag, value FROM metadata WHERE key = ?")
            .bind(key as f64)
            .fetch(db.deref());

        #[derive(sqlx::FromRow, Debug)]
        struct Row {
            tag: String,
            value: String,
        }

        let mut meta = Meta::default();
        while let Some(row) = cur.next().await? {
            let row = Row::from_row(&row)?;
            meta.insert(row.tag, row.value);
        }

        Ok(meta)
    }

    async fn delete_key(&self, key: Key) -> Result<()> {
        let db = self.c.write().await;
        sqlx::query("DELETE FROM metadata WHERE key = ?")
            .bind(key as i64)
            .execute(db.deref())
            .await?;

        Ok(())
    }

    async fn find<'a>(&'a self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>> {
        let q = "SELECT key FROM metadata WHERE tag = ? AND value = ?";
        let mut query_str = String::new();

        for _ in 0..meta.count() {
            if query_str.len() > 0 {
                query_str.push_str(" intersect ");
            }
            query_str.push_str(q);
        }

        if query_str.len() == 0 {
            //no tags where provided
            query_str.push_str("SELECT DISTINCT key FROM metadata");
        }
        #[derive(sqlx::FromRow, Debug)]
        struct Row {
            key: f64,
        }
        let (mut tx, rx) = mpsc::channel(10);
        let pool = self.c.clone();
        tokio::spawn(async move {
            let mut query = sqlx::query(&query_str);
            for (k, v) in meta {
                query = query.bind(k).bind(v);
            }
            let db = pool.read().await;
            let mut cur = query.fetch(db.deref());

            loop {
                let res = match cur.next().await {
                    Err(err) => Err(format_err!("{}", err)),
                    Ok(row) => match row {
                        None => break, // end of results
                        Some(row) => {
                            let row = Row::from_row(&row).unwrap();
                            Ok(row.key as Key)
                        }
                    },
                };

                match tx.send(res).await {
                    Ok(_) => {}
                    Err(err) => {
                        debug!("failed to send result, broken stream: {}", err);
                        break;
                    }
                };
            }
        });

        Ok(rx)
    }
}

#[derive(Serialize)]
struct ZdbMetaSer<'a> {
    key: Key,
    tags: &'a HashMap<String, String>,
}

#[derive(Deserialize)]
struct ZdbMetaDe {
    key: Key,
    tags: HashMap<String, String>,
}

/// An index interceptor that also stores the metadata in
/// a Storage.
#[derive(Clone)]
pub struct MetaInterceptor<I, S>
where
    I: Index,
    S: Storage,
{
    inner: I,
    storage: S,
}

impl<I, S> MetaInterceptor<I, S>
where
    I: Index,
    S: Storage,
{
    /// creates a new instance of the zdb interceptor
    pub fn new(index: I, storage: S) -> Self {
        MetaInterceptor {
            inner: index,
            storage: storage,
        }
    }
}

impl<I, S> MetaInterceptor<I, S>
where
    I: Index,
    S: Storage,
{
    /// rebuild index by scanning the storage database for entries,
    /// and calling set on the index store. Optionally start scanning from
    /// from.
    pub async fn rebuild(&mut self, from: Option<u32>) -> Result<()> {
        match from {
            None => self.rebuild_all().await,
            Some(ts) => self.rebuild_from(ts).await,
        }
    }

    async fn rebuild_from(&mut self, from: u32) -> Result<()> {
        let mut start = None;
        // we try to find the first KEY that is created after this timestamp
        // we iterate backwards, check each key insertion time, until we hit a key
        // that was created before this time. Once we have found the one, we use the
        // first key after this one, and continue scanning from there.
        for r in self.storage.rev()? {
            let ts = match r.timestamp {
                Some(ts) => ts,
                None => {
                    bail!("rebuild from storage is not supported for this storage implementation")
                }
            };

            if ts < from {
                break;
            }

            start = Some(r.key);
        }

        if start.is_none() {
            return Ok(());
        }

        let mut key = start.unwrap();

        loop {
            let data = match self.storage.get(key)? {
                Some(data) => data,
                None => {
                    break; // we hit the end
                }
            };

            let obj = serde_json::from_slice::<ZdbMetaDe>(&data)?;
            self.inner.set(obj.key, Meta::new(obj.tags)).await?;
            key += 1;
        }

        Ok(())
    }

    async fn rebuild_all(&mut self) -> Result<()> {
        for k in self.storage.keys()? {
            let data = match self.storage.get(k.key)? {
                Some(data) => data,
                None => {
                    warn!("metadata with key '{}' not found", k.key);
                    continue;
                }
            };

            let obj = serde_json::from_slice::<ZdbMetaDe>(&data)?;

            self.inner.set(obj.key, Meta::new(obj.tags)).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl<I, S> Index for MetaInterceptor<I, S>
where
    I: Index,
    S: Storage + Send + Sync + 'static,
{
    async fn set(&self, key: Key, meta: Meta) -> Result<()> {
        let m = ZdbMetaSer {
            key: key,
            tags: &meta.0,
        };

        let bytes = serde_json::to_vec(&m)?;
        let db = self.storage.clone();
        spawn_blocking(move || db.set(None, &bytes))
            .await
            .context("failed to run blocking task")?
            .context("failed to set metadata")?;

        self.inner.set(key, meta).await
    }

    async fn get(&self, key: Key) -> Result<Meta> {
        self.inner.get(key).await
    }

    async fn find(&self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>> {
        self.inner.find(meta).await
    }
}

#[cfg(test)]
pub mod memory {
    use crate::database::{Index, Meta};
    use crate::storage::Key;
    use anyhow::Result;
    use async_trait::async_trait;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio::sync::Mutex;

    #[derive(Clone)]
    pub struct MemoryIndex {
        data: Arc<Mutex<HashMap<(String, String), HashSet<u32>>>>,
    }

    impl MemoryIndex {
        pub fn new() -> Self {
            MemoryIndex {
                data: Arc::new(Mutex::new(HashMap::default())),
            }
        }
    }

    #[async_trait]
    impl Index for MemoryIndex {
        async fn set(&self, key: Key, meta: Meta) -> Result<()> {
            let mut data = self.data.lock().await;
            for pair in meta {
                let set = data.get_mut(&pair);
                match set {
                    Some(set) => {
                        set.insert(key);
                    }
                    None => {
                        let mut set = HashSet::default();
                        set.insert(key);
                        data.insert(pair, set);
                    }
                };
            }
            Ok(())
        }

        async fn get(&self, key: Key) -> Result<Meta> {
            let data = self.data.lock().await;
            let mut meta = Meta::default();
            for ((k, v), s) in data.iter() {
                if !s.get(&key).is_none() {
                    meta.insert::<String, String>(k.into(), v.into())
                }
            }

            Ok(meta)
        }

        async fn find(&self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>> {
            let data = self.data.lock().await;
            let mut results: Option<HashSet<u32>> = None;
            for pair in meta {
                let set = data.get(&pair);
                match set {
                    None => break,
                    Some(data) => {
                        results = match results {
                            None => Some(data.clone()),
                            Some(results) => {
                                results.intersection(data);
                                Some(results)
                            }
                        }
                    }
                }
            }

            let (mut tx, rx) = mpsc::channel(10);
            tokio::spawn(async move {
                let results = match results {
                    None => return,
                    Some(results) => results,
                };

                for result in results {
                    tx.send(Ok(result)).await.unwrap();
                }
            });
            Ok(rx)
        }
    }

    #[derive(Clone)]
    pub struct NullIndex;

    #[async_trait]
    impl Index for NullIndex {
        async fn set(&self, key: Key, meta: Meta) -> Result<()> {
            Ok(())
        }
        async fn get(&self, key: Key) -> Result<Meta> {
            bail!("not supported");
        }
        async fn find(&self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>> {
            bail!("not supported");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::memory::MemoryIndex;
    use super::*;
    use crate::database::Meta;

    #[tokio::test]
    async fn memory_index() {
        let index = MemoryIndex::new();
        let mut meta1 = Meta::default();
        meta1.insert("name", "user1");
        meta1.insert("age", "38");
        let results = index.set(1, meta1).await;

        assert_eq!(results.is_ok(), true);

        let mut meta2 = Meta::default();
        meta2.insert("name", "user2");
        meta2.insert("age", "38");
        let results = index.set(2, meta2).await;

        assert_eq!(results.is_ok(), true);

        let loaded = index.get(1).await.unwrap();
        assert_eq!(loaded.count(), 2);
        assert_eq!(loaded.get("name").unwrap(), "user1");
        assert_eq!(loaded.get("age").unwrap(), "38");

        let loaded = index.get(2).await.unwrap();
        assert_eq!(loaded.count(), 2);
        assert_eq!(loaded.get("name").unwrap(), "user2");
        assert_eq!(loaded.get("age").unwrap(), "38");

        let mut find = Meta::default();
        find.insert("age", "38");

        use tokio::stream::StreamExt;
        let found = index.find(find).await.unwrap();
        let results: Vec<Result<Key>> = found.collect().await;

        assert_eq!(results.len(), 2);

        let mut find = Meta::default();
        find.insert("name", "user1");

        let found = index.find(find).await.unwrap();
        let results: Vec<Result<Key>> = found.collect().await;

        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn schema() {
        let db = "/tmp/testing.sqlite3";
        if std::path::Path::new(db).exists() {
            std::fs::remove_file(db).expect("failed to clean up file");
        }

        let constr = format!("sqlite://{}", db);
        let c = SqlitePool::new(&constr).await.expect("failed to connect");
        let schema = Schema::new(c);
        schema.setup().await.expect("failed to create table");
        let mut meta = Meta::default();
        meta.insert("name", "filename");
        meta.insert("type", "file");
        meta.insert("parent", "/root/dir");

        schema
            .insert(100, meta)
            .await
            .expect("failed to insert object");

        let getter = schema.clone();
        let mut filter = Meta::default();
        filter.insert("name", "filename");
        let mut cur = schema.find(filter).await.expect("failed to do fine");
        loop {
            let key = match cur.recv().await {
                Some(key) => key,
                None => break,
            };
            let tags = getter.get(key.unwrap()).await.expect("object not found");
            for (k, v) in tags {
                println!("{}: {}", k, v);
            }
        }
    }

    #[tokio::test]
    async fn schema_safety() {
        let db = "/tmp/testing_safety.sqlite3";
        if std::path::Path::new(db).exists() {
            std::fs::remove_file(db).expect("failed to clean up file");
        }

        let constr = format!("sqlite://{}", db);
        let c = SqlitePool::new(&constr).await.expect("failed to connect");
        let schema = Schema::new(c);
        schema.setup().await.expect("failed to create table");
        let mut meta = Meta::default();
        meta.insert("name", "filename");
        meta.insert("type", "file");
        meta.insert("parent", "/root/dir");

        let mut handles = vec![];
        for i in 0..100 {
            let schema = schema.clone();
            let meta = meta.clone();
            let handle = tokio::spawn(async move {
                schema.insert(i, meta).await.expect("failed to insert data");
            });

            handles.push(handle);
        }

        assert_eq!(handles.len(), 100);
        for handle in handles {
            assert_eq!(handle.await.is_ok(), true);
        }

        let mut results = schema.find(Meta::default()).await.expect("find failed");

        let mut keys = vec![];
        while let Some(item) = results.recv().await {
            let key = item.expect("expecting a key");
            keys.push(key);
        }

        assert_eq!(keys.len(), 100);
    }

    #[tokio::test]
    async fn sqlite_index() {
        const DIR: &str = "/tmp/sqlite-index.test";
        let _ = std::fs::remove_dir_all(DIR);
        let builder = SqliteIndexBuilder::new(DIR).unwrap();

        let index = builder.build("metadata").await.unwrap();
        let mut meta1 = Meta::default();
        meta1.insert("name", "user1");
        meta1.insert("age", "38");
        let results = index.set(1, meta1).await;

        assert_eq!(results.is_ok(), true);

        let mut meta2 = Meta::default();
        meta2.insert("name", "user2");
        meta2.insert("age", "38");
        let results = index.set(2, meta2).await;

        assert_eq!(results.is_ok(), true);

        let loaded = index.get(1).await.unwrap();
        assert_eq!(loaded.count(), 2);
        assert_eq!(loaded.get("name").unwrap(), "user1");
        assert_eq!(loaded.get("age").unwrap(), "38");

        let loaded = index.get(2).await.unwrap();
        assert_eq!(loaded.count(), 2);
        assert_eq!(loaded.get("name").unwrap(), "user2");
        assert_eq!(loaded.get("age").unwrap(), "38");

        //update
        let mut meta2 = Meta::default();
        meta2.insert("name", "updated");
        let results = index.set(2, meta2).await;

        assert_eq!(results.is_ok(), true);
        let loaded = index.get(2).await.unwrap();
        assert_eq!(loaded.count(), 2);
        assert_eq!(loaded.get("name").unwrap(), "updated");
        assert_eq!(loaded.get("age").unwrap(), "38");

        let mut find = Meta::default();
        find.insert("age", "38");

        use tokio::stream::StreamExt;
        let found = index.find(find).await.unwrap();
        let results: Vec<Result<Key>> = found.collect().await;

        assert_eq!(results.len(), 2);

        let mut find = Meta::default();
        find.insert("name", "updated");

        let found = index.find(find).await.unwrap();
        let results: Vec<Result<Key>> = found.collect().await;

        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn sqlite_delete() {
        const DIR: &str = "/tmp/sqlite-delete.test";
        let _ = std::fs::remove_dir_all(DIR);
        let builder = SqliteIndexBuilder::new(DIR).unwrap();

        let index = builder.build("metadata").await.unwrap();
        let mut meta1 = Meta::default();
        meta1.insert("name", "user1");
        meta1.insert("age", "38");
        let results = index.set(1, meta1).await;

        assert_eq!(results.is_ok(), true);

        let mut meta2 = Meta::default();
        meta2.insert("name", "user2");
        meta2.insert("age", "38");
        let results = index.set(2, meta2).await;

        assert_eq!(results.is_ok(), true);

        let loaded = index.get(1).await.unwrap();
        assert_eq!(loaded.count(), 2);
        assert_eq!(loaded.get("name").unwrap(), "user1");
        assert_eq!(loaded.get("age").unwrap(), "38");

        let loaded = index.get(2).await.unwrap();
        assert_eq!(loaded.count(), 2);
        assert_eq!(loaded.get("name").unwrap(), "user2");
        assert_eq!(loaded.get("age").unwrap(), "38");

        index
            .set(1, Meta::default().with_deleted(true))
            .await
            .unwrap();

        let loaded = index.get(1).await.unwrap();
        assert_eq!(loaded.count(), 0);

        let loaded = index.get(2).await.unwrap();
        assert_eq!(loaded.count(), 2);
        assert_eq!(loaded.get("name").unwrap(), "user2");
        assert_eq!(loaded.get("age").unwrap(), "38");
    }

    #[tokio::test]
    async fn sqlite_perf() {
        // this should probably be replaced by a benchmark test
    }
}
