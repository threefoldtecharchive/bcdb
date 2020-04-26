use crate::storage::Key;
use async_trait::async_trait;
use failure::Error;
use tokio::sync::mpsc;

pub mod sqlite;

#[derive(Debug)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

impl Tag {
    pub fn new<K, S>(key: K, value: S) -> Tag
    where
        K: Into<String>,
        S: Into<String>,
    {
        Tag {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn is_reserved(&self) -> bool {
        self.key.starts_with(":")
    }
}

pub struct Meta {
    pub tags: Vec<Tag>,
}

impl Meta {
    pub fn add<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.tags.push(Tag::new(key, value))
    }
}

#[async_trait]
pub trait Storage: Send + Sync + 'static {
    async fn set(&mut self, key: Key, meta: Meta) -> Result<(), Error>;
    async fn get(&mut self, key: Key) -> Result<Meta, Error>;
    async fn find(&mut self, tags: Vec<Tag>) -> Result<mpsc::Receiver<Result<Key, Error>>, Error>;
}

#[async_trait]
pub trait StorageFactory: Send + Sync + 'static {
    type Storage: Storage + Clone;

    async fn new(&self, typ: &str) -> Result<Self::Storage, Error>;
}
