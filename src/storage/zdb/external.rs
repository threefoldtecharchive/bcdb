//! This crate provides a wrapper for a client to a 0-db running as a separate process on the same
//! system. The 0-db must be running in sequential mode

use crate::storage::{Error as StorageError, Key, Storage};

use redis::ConnectionLike;
use redis::RedisError;
use scheduled_thread_pool::ScheduledThreadPool;

use std::convert::TryInto;
use std::sync::Arc;

#[derive(Clone)]
pub struct Zdb {
    client: redis::Client,
    // Thread pool used by r2d2 to spawn collections. We share one to avoid every connection pool
    // in every namespace allocating one.
    spawn_pool: Arc<ScheduledThreadPool>,
    // upon connection we are always connected to the default namespace
    default_namespace: Collection,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pool: r2d2::Pool<ZdbConnectionManager>,
}

/// Zdb connection manager to be used by the r2d2 crate. Because namespaces in zdb are tied to the
/// actual connection, we need to manage connection pools on namespace lvl.
#[derive(Debug, Clone)]
struct ZdbConnectionManager {
    namespace: Option<String>,
    client: redis::Client,
}

impl Zdb {
    pub fn new(port: u16) -> Zdb {
        let client = redis::Client::open(format!("redis://localhost:{}", port))
            .expect("Could not connect to zdb");
        let spawn_pool = Arc::new(ScheduledThreadPool::new(2));
        let default_namespace = Collection::new(client.clone(), None, spawn_pool.clone());
        Zdb {
            client,
            spawn_pool,
            default_namespace,
        }
    }
}

impl Storage for Zdb {
    fn set(&mut self, key: Option<Key>, data: &[u8]) -> Result<Key, StorageError> {
        self.default_namespace.set(key, data)
    }

    fn get(&mut self, key: Key) -> Result<Option<Vec<u8>>, StorageError> {
        self.default_namespace.get(key)
    }
}

impl Default for Zdb {
    fn default() -> Self {
        // default port 9900
        Self::new(9900)
    }
}

impl Collection {
    fn new(
        client: redis::Client,
        namespace: Option<String>,
        spawn_pool: Arc<ScheduledThreadPool>,
    ) -> Collection {
        let manager = ZdbConnectionManager::new(client, namespace);
        // All these options should be verified and refined once we have some load estimate
        let pool = r2d2::Builder::new()
            .max_size(10)
            .min_idle(Some(1))
            .thread_pool(spawn_pool)
            // TODO: set connection lifetimes
            .build_unchecked(manager);
        Collection { pool }
    }
}

impl Storage for Collection {
    fn set(&mut self, key: Option<Key>, data: &[u8]) -> Result<Key, StorageError> {
        let raw_key: Vec<u8> = redis::cmd("SET")
            .arg(if let Some(key) = key {
                Vec::from(&key.to_le_bytes()[..])
            } else {
                Vec::new()
            })
            .arg(data)
            .query(&mut *self.pool.get()?)?;
        debug_assert!(raw_key.len() == std::mem::size_of::<Key>());
        Ok(read_le_key(&raw_key))
    }

    fn get(&mut self, key: Key) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(redis::cmd("GET")
            .arg(&key.to_le_bytes())
            .query(&mut *self.pool.get()?)?)
    }
}

fn read_le_key(input: &[u8]) -> Key {
    let (int_bytes, _) = input.split_at(std::mem::size_of::<Key>());
    Key::from_le_bytes(
        int_bytes
            .try_into()
            .expect("could not convert bytes to key"),
    )
}

impl ZdbConnectionManager {
    fn new(client: redis::Client, namespace: Option<String>) -> ZdbConnectionManager {
        ZdbConnectionManager { client, namespace }
    }
}

// This implementation is mainly taken from the r2d2 redis implementation, with the difference
// being that we set the namespace on the connections
impl r2d2::ManageConnection for ZdbConnectionManager {
    type Connection = redis::Connection;
    type Error = redis::RedisError;

    fn connect(&self) -> Result<redis::Connection, redis::RedisError> {
        let conn = self.client.get_connection()?;
        if let Some(ref ns) = self.namespace {
            // TODO: set namespace
        }
        Ok(conn)
    }

    fn is_valid(&self, conn: &mut redis::Connection) -> Result<(), redis::RedisError> {
        redis::cmd("PING").query(conn)
    }

    fn has_broken(&self, conn: &mut redis::Connection) -> bool {
        !conn.is_open()
    }
}

impl From<RedisError> for StorageError {
    fn from(e: RedisError) -> Self {
        if e.is_io_error() {
            return StorageError::IO(None);
        }
        StorageError::Protocol(e.category().to_owned())
    }
}

impl From<r2d2::Error> for StorageError {
    fn from(e: r2d2::Error) -> Self {
        StorageError::IO(None)
    }
}
