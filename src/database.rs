use crate::storage::Key;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub mod data;
pub mod index;

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

//#[derive(Default)]
pub type Meta = HashMap<String, String>;

#[derive(Default)]
pub struct Object {
    pub key: Key,
    pub meta: Meta,
    pub data: Vec<u8>,
}

#[async_trait]
pub trait Index: Send + Sync + 'static {
    async fn set(&mut self, key: Key, meta: Meta) -> Result<()>;
    async fn get(&mut self, key: Key) -> Result<Meta>;
    async fn find(&mut self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>>;
}

pub enum Context {
    Owner,
    User(u64),
    Unknown,
}

impl Context {
    pub fn is_authenticated(&self) -> bool {
        match self {
            Self::Unknown => false,
            _ => true,
        }
    }

    pub fn is_owner(&self) -> bool {
        match self {
            Self::Owner => true,
            _ => false,
        }
    }
}

#[async_trait]
pub trait Database: Send + Sync + 'static {
    async fn set(
        &mut self,
        ctx: Context,
        collection: String,
        data: Vec<u8>,
        meta: Meta,
        acl: Option<u64>,
    ) -> Result<Key>;
    async fn get(&mut self, ctx: Context, key: Key) -> Result<Object>;
    // update
    // delete
    async fn find(
        &mut self,
        ctx: Context,
        meta: Meta,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Key>>>;
}
