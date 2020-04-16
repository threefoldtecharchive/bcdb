use crate::storage::Key;
use async_trait::async_trait;
use failure::Error;
use tokio::stream::Stream;

pub mod sqlite;

pub struct Tag {
    pub key: String,
    pub value: String,
}

impl Tag {
    pub fn new<S>(key: S, value: S) -> Tag
    where
        S: Into<String>,
    {
        Tag {
            key: key.into(),
            value: value.into(),
        }
    }
}

pub struct Meta {
    pub tags: Vec<Tag>,
}

#[async_trait]
pub trait Storage: Send + Sync + 'static {
    async fn set(&mut self, key: Key, meta: Meta) -> Result<(), Error>;
    async fn get(&mut self, key: Key) -> Result<Meta, Error>;
    async fn find<'a>(
        &'a mut self,
        tags: Vec<Tag>,
    ) -> Result<Box<dyn Stream<Item = Result<Key, Error>> + 'a>, Error>;
}

#[async_trait]
pub trait StorageFactory: Send + Sync + 'static {
    type Storage: Storage + Clone;

    async fn new(&self, typ: &str) -> Result<Self::Storage, Error>;
}
