use crate::storage::Key;
use async_trait::async_trait;
use failure::Error;

pub mod sqlite;

// pub enum Value {
//     String(String),
//     Double(f64),
//     Number(i64),
//     Unsigned(u64),
// }

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
    pub blob: Option<Vec<u8>>,
    pub text: Option<String>,
}

#[async_trait]
pub trait Storage {
    async fn set(&mut self, key: Key, meta: Meta) -> Result<(), Error>;
    async fn get(&mut self, key: Key) -> Result<Meta, Error>;
}
