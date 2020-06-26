use super::*;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::prelude::*;
use sqlx::SqlitePool;
use tokio::sync::mpsc;

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
            .await?;
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

                tx.send(res).await.expect("failed to send results");
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
