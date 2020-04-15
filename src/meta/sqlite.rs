use crate::meta::{Key, Meta, Storage, Tag};
use async_trait::async_trait;
use failure::Error;
use sqlx::prelude::*;
use sqlx::sqlite::{SqliteCursor, SqliteRow};
use sqlx::Error as SqlError;
use sqlx::SqlitePool;
use tokio::prelude::*;
use tokio::stream::Stream;

type Result<T> = std::result::Result<T, Error>;

struct SqliteMetaStore {
    pool: SqlitePool,
}

impl SqliteMetaStore {
    async fn new() -> Result<SqliteMetaStore> {
        let pool = SqlitePool::new(":memory:").await?;
        //Self::schame(&pool);

        Ok(SqliteMetaStore { pool: pool })
    }
}

#[async_trait]
impl Storage for SqliteMetaStore {
    async fn set(&mut self, key: Key, meta: Meta) -> Result<()> {
        Ok(())
    }

    async fn get(&mut self, key: Key) -> Result<Meta> {
        bail!("not implemented")
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

        CREATE INDEX IF NOT EXISTS metadata_key ON metadata (
            key
        );

        CREATE INDEX IF NOT EXISTS metadata_tag ON metadata (
            tag
        );

        CREATE INDEX IF NOT EXISTS metadata_value ON metadata (
            value
        );
        ",
        )
        .execute(&self.c)
        .await?;

        Ok(())
    }

    async fn insert(&mut self, key: Key, tags: Vec<Tag>) -> Result<()> {
        let mut tx: sqlx::Transaction<_> = self.c.begin().await?;
        for tag in tags {
            sqlx::query(
                "INSERT INTO metadata (key, tag, value) VALUES
                (?, ?, ?);",
            )
            .bind(key as f64) // only f64 is supported by sqlite.
            .bind(tag.key)
            .bind(tag.value)
            .execute(&mut tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get(&mut self, key: Key) -> Result<Vec<Tag>> {
        let mut cur = sqlx::query("SELECT tag, value FROM metadata WHERE key = ?")
            .bind(key as f64)
            .fetch(&self.c);

        #[derive(sqlx::FromRow, Debug)]
        struct Row {
            tag: String,
            value: String,
        }

        let mut tags: Vec<Tag> = vec![];
        while let Some(row) = cur.next().await? {
            let row = Row::from_row(&row)?;
            tags.push(Tag::new(row.tag, row.value));
        }

        Ok(tags)
    }

    async fn find<'a>(
        &'a mut self,
        tags: Vec<Tag>,
    ) -> Result<impl Stream<Item = std::result::Result<f64, SqlError>> + 'a> {
        let q = "SELECT key FROM metadata WHERE tag = ? AND value = ?";
        let mut query_str = String::new();

        for _ in 0..tags.len() {
            if query_str.len() > 0 {
                query_str.push_str(" intersect ");
            }
            query_str.push_str(q);
        }

        let mut query = sqlx::query(q);
        for tag in tags {
            query = query.bind(tag.key).bind(tag.value);
        }
        #[derive(sqlx::FromRow, Debug)]
        struct Row {
            key: f64,
        }
        let cur = query
            .map(|r: SqliteRow| Row::from_row(&r).expect("failed to load item").key)
            .fetch(&self.c);
        Ok(cur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn schema() {
        let db = "/tmp/testing.sqlite3";
        std::fs::remove_file(db).expect("failed to clean up file");

        let constr = format!("sqlite://{}", db);
        let c = SqlitePool::new(&constr).await.expect("failed to connect");
        let mut schema = Schema::new(c);
        schema.setup().await.expect("failed to create table");
        schema
            .insert(
                100,
                vec![
                    Tag::new("name", "filename"),
                    Tag::new("type", "file"),
                    Tag::new("parent", "/root/dir"),
                ],
            )
            .await
            .expect("failed to insert object");

        let mut getter = schema.clone();
        let cur = schema
            .find(vec![Tag::new("name", "filename")])
            .await
            .expect("failed to do fine");
        use tokio::stream::StreamExt;
        tokio::pin!(cur);
        while let Some(row) = cur.next().await {
            let tags = getter
                .get(row.expect("invalid row") as Key)
                .await
                .expect("object not found");
            for t in tags {
                println!("{}: {}", t.key, t.value);
            }
        }
    }
}
