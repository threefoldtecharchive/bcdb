pub use crate::storage::Key;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::IntoIterator;
use thiserror::Error;
use tokio::sync::mpsc;
use tonic::metadata::MetadataMap;

pub mod data;
pub mod index;

pub use data::BcdbDatabase;
pub use index::SqliteIndexBuilder;

const TAG_COLLECTION: &str = ":collection";
const TAG_ACL: &str = ":acl";
const TAG_CREATED: &str = ":created";
const TAG_UPDATED: &str = ":updated";
const TAG_DELETED: &str = ":deleted";
const TAG_SIZE: &str = ":size";

#[derive(Error, Debug, Clone, PartialEq)]
pub enum Reason {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Object with not found")]
    NotFound,

    #[error("operation not supported")]
    NotSupported,

    #[error("invalid tag")]
    InvalidTag,

    #[error("Cannot get peer: {0}")]
    CannotGetPeer(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<tonic::Status> for Reason {
    fn from(s: tonic::Status) -> Self {
        use tonic::Code;
        match s.code() {
            Code::Unauthenticated => Reason::Unauthorized,
            Code::NotFound => Reason::NotFound,
            Code::Unavailable => Reason::CannotGetPeer(s.message().into()),
            Code::InvalidArgument => Reason::InvalidTag,
            _ => Reason::Unknown(s.message().into()),
        }
    }
}

impl From<&anyhow::Error> for Reason {
    fn from(err: &anyhow::Error) -> Self {
        match err.downcast_ref::<Reason>() {
            Some(reason) => reason.clone(),
            None => Reason::Unknown(format!("{}", err)),
        }
    }
}

pub fn is_reserved(tag: &str) -> bool {
    tag.starts_with(":")
}

#[derive(Default, Debug, Clone)]
pub struct Meta(HashMap<String, String>);

impl Meta {
    pub fn new(tags: HashMap<String, String>) -> Self {
        Meta(tags)
    }

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

    pub fn is_collection<S: AsRef<str>>(&self, collection: S) -> bool {
        match self.collection() {
            Some(col) if col == collection.as_ref() => true,
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
        self.get_u64(TAG_UPDATED)
    }

    pub fn deleted(&self) -> bool {
        self.get_u64(TAG_DELETED).map(|v| v >= 1).unwrap_or(false)
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

    pub fn with_deleted(self, deleted: bool) -> Self {
        self.with_u64(TAG_DELETED, if deleted { 1 } else { 0 })
    }
}

impl Into<HashMap<String, String>> for Meta {
    fn into(self) -> HashMap<String, String> {
        self.0
    }
}

impl IntoIterator for Meta {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl TryFrom<HashMap<String, String>> for Meta {
    type Error = anyhow::Error;

    /// try_from should be used when converting user input to meta object
    /// it makes sure that no internal tags are used by the user.
    fn try_from(m: HashMap<String, String>) -> Result<Self> {
        for (k, _) in m.iter() {
            if is_reserved(k) {
                bail!(Reason::InvalidTag);
            }
        }

        Ok(Meta(m))
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
    /// set operation is used to associate meta data to key
    /// the operation will do an update (merge) of key metadata
    /// if metadata already exists for that key.
    ///
    /// If the meta.deleted flag is set, the entire key is deleted
    /// We don't have a separate "delete" operation so metadata interceptors
    /// that needs to keep a transaction log of metadata change can reply
    /// the metadata changes associated with a key.
    async fn set(&self, key: Key, meta: Meta) -> Result<()>;
    async fn get(&self, key: Key) -> Result<Meta>;
    async fn find(&self, meta: Meta) -> Result<mpsc::Receiver<Result<Key>>>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Route {
    Local,
    Remote(u32),
}

impl Default for Route {
    fn default() -> Self {
        Route::Local
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Authorization {
    Invalid,
    Owner,
    User(u32),
}

impl Default for Authorization {
    fn default() -> Self {
        Authorization::Invalid
    }
}

#[derive(Default, Debug, Clone)]
pub struct Context {
    pub route: Route,
    pub authorization: Authorization,
}

impl Context {
    pub fn with_route(mut self, route: Option<u32>) -> Self {
        match route {
            Some(r) => self.route = Route::Remote(r),
            None => self.route = Route::Local,
        };
        self
    }

    pub fn with_auth(mut self, auth: Authorization) -> Self {
        self.authorization = auth;
        self
    }

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

    pub fn into_metadata(self, meta: &mut MetadataMap) {
        use tonic::metadata::AsciiMetadataValue;
        match self.route {
            Route::Local => {}
            Route::Remote(r) => {
                meta.insert(
                    "remote",
                    AsciiMetadataValue::from_str(&format!("{}", r)).unwrap(),
                );
            }
        };

        match self.authorization {
            Authorization::Owner => {
                meta.insert("owner", AsciiMetadataValue::from_static("true"));
            }
            Authorization::User(u) => {
                meta.insert(
                    "key-id",
                    AsciiMetadataValue::from_str(&format!("{}", u)).unwrap(),
                );
            }
            _ => {}
        };
    }
}

#[async_trait]
pub trait Database: Send + Sync + 'static {
    async fn set(
        &mut self,
        ctx: Context,
        collection: String,
        data: Vec<u8>,
        meta: HashMap<String, String>,
        acl: Option<u64>,
    ) -> Result<Key>;

    async fn fetch(&mut self, ctx: Context, key: Key) -> Result<Object>;

    async fn get(&mut self, ctx: Context, key: Key, collection: String) -> Result<Object>;

    async fn head(&mut self, ctx: Context, key: Key, collection: String) -> Result<Object>;

    async fn delete(&mut self, ctx: Context, key: Key, collection: String) -> Result<()>;

    async fn update(
        &mut self,
        ctx: Context,
        key: Key,
        collection: String,
        data: Option<Vec<u8>>,
        tags: HashMap<String, String>,
        acl: Option<u64>,
    ) -> Result<()>;

    async fn list(
        &mut self,
        ctx: Context,
        tags: HashMap<String, String>,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Key>>>;

    async fn find(
        &mut self,
        ctx: Context,
        tags: HashMap<String, String>,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Object>>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn meta_try_from_ok() {
        let mut tags: HashMap<String, String> = HashMap::new();
        tags.insert("name".into(), "some name".into());
        tags.insert("parent".into(), "some other value".into());
        let meta = Meta::try_from(tags);

        assert_eq!(meta.is_ok(), true);
    }

    #[test]
    fn meta_try_from_reserved() {
        let mut tags: HashMap<String, String> = HashMap::new();
        tags.insert(":reserved".into(), "some name".into());
        let meta = Meta::try_from(tags);

        assert_eq!(meta.is_err(), true);
    }

    #[test]
    fn meta_with_fns() {
        let meta = Meta::default()
            .with_collection("collection")
            .with_size(200)
            .with_acl(10)
            .with_created(2000)
            .with_updated(3000)
            .with_deleted(true);

        assert_eq!(meta.collection(), Some("collection".into()));
        assert_eq!(meta.size(), Some(200));
        assert_eq!(meta.acl(), Some(10));
        assert_eq!(meta.created(), Some(2000));
        assert_eq!(meta.updated(), Some(3000));
        assert_eq!(meta.deleted(), true);
    }

    #[test]
    fn meta_deleted() {
        let meta = Meta::default();
        assert_eq!(meta.deleted(), false);

        let meta = meta.with_deleted(true);
        assert_eq!(meta.deleted(), true);

        let meta = meta.with_deleted(false);
        assert_eq!(meta.deleted(), false);
    }

    #[test]
    fn context_default() {
        let ctx = Context::default();
        assert_eq!(ctx.authorization, Authorization::Invalid);
        assert_eq!(ctx.route, Route::Local);
    }
}
