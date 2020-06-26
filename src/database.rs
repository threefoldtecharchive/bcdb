use crate::storage::Key;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::iter::{FromIterator, IntoIterator, Iterator};
use tokio::sync::mpsc;

pub mod data;
pub mod index;

const TAG_COLLECTION: &str = ":collection";
const TAG_ACL: &str = ":acl";
const TAG_CREATED: &str = ":created";
const TAG_UPDATED: &str = ":updated";
const TAG_DELETE: &str = ":deleted";
const TAG_SIZE: &str = ":size";

pub fn is_reserved(tag: &str) -> bool {
    tag.starts_with(":")
}

#[derive(Default, Debug)]
pub struct Meta(HashMap<String, String>);

impl Meta {
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.0.insert(key.into(), value.into());
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&String> {
        self.0.get(key.as_ref())
    }

    pub fn collection(&self) -> Option<String> {
        self.get(TAG_COLLECTION).map(|v| v.clone())
    }

    pub fn is_collection(&self, collection: &str) -> bool {
        match self.collection() {
            Some(col) if col == collection => true,
            _ => false,
        }
    }

    fn get_u64<K: AsRef<str>>(&self, key: K) -> Option<u64> {
        match self.get(key) {
            Some(v) => v.parse().ok(),
            None => None,
        }
    }

    pub fn size(&self) -> Option<u64> {
        self.get_u64(TAG_SIZE)
    }

    pub fn acl(&self) -> Option<u64> {
        self.get_u64(TAG_ACL)
    }

    pub fn created(&self) -> Option<u64> {
        self.get_u64(TAG_CREATED)
    }

    pub fn updated(&self) -> Option<u64> {
        self.get_u64(TAG_CREATED)
    }

    pub fn with_collection<V: Into<String>>(mut self, collection: V) -> Self {
        self.0.insert(TAG_COLLECTION.into(), collection.into());
        self
    }

    fn with_u64<K: Into<String>>(mut self, key: K, v: u64) -> Self {
        self.0.insert(key.into(), format!("{}", v));
        self
    }

    pub fn with_acl(self, acl: u64) -> Self {
        self.with_u64(TAG_ACL, acl)
    }

    pub fn with_size(self, size: u64) -> Self {
        self.with_u64(TAG_SIZE, size)
    }

    pub fn with_created(self, created: u64) -> Self {
        self.with_u64(TAG_CREATED, created)
    }

    pub fn with_updated(self, updated: u64) -> Self {
        self.with_u64(TAG_UPDATED, updated)
    }
}

impl IntoIterator for Meta {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(String, String)> for Meta {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let mut map = HashMap::default();
        for (k, v) in iter.into_iter() {
            // if is_reserved(&k) {
            //     continue;
            // }
            map.insert(k, v);
        }

        Meta(map)
    }
}

#[derive(Default, Debug)]
pub struct Object {
    pub key: Key,
    pub meta: Meta,
    pub data: Option<Vec<u8>>,
}

#[async_trait]
pub trait Index: Send + Sync + 'static {
    async fn set(&mut self, key: Key, meta: Meta) -> Result<()>;
    async fn get(&mut self, key: Key) -> Result<Meta>;
    async fn find(&mut self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>>;
}

pub enum Route {
    Local,
    Remote(u64),
}

pub enum Authorization {
    Owner,
    User(u64),
}

pub struct Context {
    pub route: Route,
    pub authorization: Authorization,
}

impl Context {
    pub fn is_owner(&self) -> bool {
        match self.authorization {
            Authorization::Owner => true,
            _ => false,
        }
    }

    pub fn is_local(&self) -> bool {
        match self.route {
            Route::Local => true,
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

    async fn update(
        &mut self,
        ctx: Context,
        key: Key,
        collection: String,
        data: Option<Vec<u8>>,
        meta: Meta,
        acl: Option<u64>,
    ) -> Result<()>;
    // update
    // delete
    async fn list(
        &mut self,
        ctx: Context,
        meta: Meta,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Key>>>;

    async fn find(
        &mut self,
        ctx: Context,
        meta: Meta,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Object>>>;
}
