use super::*;
use crate::storage::Storage;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::prelude::*;
use sqlx::SqlitePool;
use tokio::sync::mpsc;
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
        let mut schema = Schema::new(pool);
        schema.setup().await?;

        Ok(SqliteIndex { schema })
    }
}

#[async_trait]
impl Index for SqliteIndex {
    async fn set(&mut self, key: Key, meta: Meta) -> Result<()> {
        self.schema.insert(key, meta).await
    }

    async fn get(&mut self, key: Key) -> Result<Meta> {
        self.schema.get(key).await
    }

    async fn find(&mut self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>> {
        self.schema.find(meta).await
    }
}

#[derive(Clone)]
struct Schema {
    c: SqlitePool,
}

impl Schema {
    fn new(c: SqlitePool) -> Schema {
        Schema { c }
    }

    async fn setup(&mut self) -> Result<()> {
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
        .execute(&self.c)
        .await?;

        Ok(())
    }

    async fn insert(&mut self, key: Key, tags: Meta) -> Result<()> {
        let mut tx: sqlx::Transaction<_> = self.c.begin().await?;
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
            .execute(&mut tx)
            .await
            .context("failed to insert data to index")?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get(&mut self, key: Key) -> Result<Meta> {
        let mut cur = sqlx::query("SELECT tag, value FROM metadata WHERE key = ?")
            .bind(key as f64)
            .fetch(&self.c);

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

    async fn find<'a>(&'a mut self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>> {
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

            let mut cur = query.fetch(&pool);

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
    async fn set(&mut self, key: Key, meta: Meta) -> Result<()> {
        let m = ZdbMetaSer {
            key: key,
            tags: &meta.0,
        };

        let bytes = serde_json::to_vec(&m)?;
        let mut db = self.storage.clone();
        spawn_blocking(move || db.set(None, &bytes).expect("failed to set metadata"))
            .await
            .context("failed to run blocking task")?;

        self.inner.set(key, meta).await
    }

    async fn get(&mut self, key: Key) -> Result<Meta> {
        self.inner.get(key).await
    }

    async fn find(&mut self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>> {
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
        async fn set(&mut self, key: Key, meta: Meta) -> Result<()> {
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

        async fn get(&mut self, key: Key) -> Result<Meta> {
            let data = self.data.lock().await;
            let mut meta = Meta::default();
            for ((k, v), s) in data.iter() {
                if !s.get(&key).is_none() {
                    meta.insert::<String, String>(k.into(), v.into())
                }
            }

            Ok(meta)
        }

        async fn find(&mut self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>> {
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
}

#[cfg(test)]
mod tests {
    use super::memory::MemoryIndex;
    use super::*;
    use crate::database::Meta;

    #[tokio::test]
    async fn memory_index() {
        let mut index = MemoryIndex::new();
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
        let mut schema = Schema::new(c);
        schema.setup().await.expect("failed to create table");
        let mut meta = Meta::default();
        meta.insert("name", "filename");
        meta.insert("type", "file");
        meta.insert("parent", "/root/dir");

        schema
            .insert(100, meta)
            .await
            .expect("failed to insert object");

        let mut getter = schema.clone();
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
    async fn sqlite_index() {
        const dir: &str = "/tmp/sqlite-index.test";
        let _ = std::fs::remove_dir_all(dir);
        let builder = SqliteIndexBuilder::new(dir).unwrap();

        let mut index = builder.build("metadata").await.unwrap();
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
}
