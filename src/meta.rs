use crate::storage::Key;
pub mod sqlite;

pub enum Value {
    String(String),
    Double(f64),
    Number(i64),
    Unsigned(u64),
}

pub struct Tag {
    pub key: String,
    pub value: Value,
}

pub struct Meta {
    pub tags: Vec<Tag>,
    pub blob: Option<Vec<u8>>,
    pub text: Option<String>,
}

pub trait Storage {
    fn set(&mut self, key: Key, meta: Meta) -> Result<(), Error>;
    fn get(&mut self, key: Key) -> Result<Meta, Error>;
}

pub enum Error {}
