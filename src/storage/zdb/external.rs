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

/// An iterator over all keys in a collection.
/// Leaking this value will cause the underlying connection to leak.
pub struct CollectionKeys {
    conn: r2d2::PooledConnection<ZdbConnectionManager>,
    cursor: Option<Vec<u8>>,
    /// since the scan method returns more than 1 key, yet does not specify exactly how many,
    /// we keep track of an internal buffer
    buffer: Vec<ScanEntry>,
    /// keep track of which entry in the buffer we are at
    buffer_idx: usize,
}

// Yes, this is a vec, and yes, this only represents a single element. It is what it is.
type ScanEntry = Vec<(Vec<u8>, u32, u32)>;

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

    /// Resets a collection then return a reference to a new *empty* collection
    /// usually used for testing.
    pub fn reset(self, name: &str) -> Collection {
        let mut client = self.client.clone();
        redis::cmd("NSDEL").arg(name).query::<()>(&mut client);
        self.collection(name)
    }

    /// Get a reference to a `Collection`.
    pub fn collection(self, name: &str) -> Collection {
        Collection::new(self.client.clone(), Some(name.into()), self.spawn_pool)
    }
}

impl Storage for Zdb {
    fn set(&mut self, key: Option<Key>, data: &[u8]) -> Result<Key, StorageError> {
        self.default_namespace.set(key, data)
    }

    fn get(&mut self, key: Key) -> Result<Option<Vec<u8>>, StorageError> {
        self.default_namespace.get(key)
    }

    fn keys(&mut self) -> Result<Box<dyn Iterator<Item = Key>>, StorageError> {
        self.default_namespace.keys()
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
            // TODO: other configuration
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

    fn keys(&mut self) -> Result<Box<dyn Iterator<Item = Key>>, StorageError> {
        Ok(Box::new(CollectionKeys {
            conn: self.pool.get()?,
            cursor: None,
            buffer: Vec::new(),
            buffer_idx: 0,
        }))
    }
}

impl Iterator for CollectionKeys {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        // Note that entries are actually single element vectors of tuples, not regular tuples.
        // If we have a buffer from a previous cmd execution, check for an entry there first
        if self.buffer_idx < self.buffer.len() {
            self.buffer_idx += 1;
            return Some(read_le_key(&self.buffer[self.buffer_idx - 1][0].0));
        }
        // No more keys in buffer, fetch a new buffer from the remote
        let mut scan_cmd = redis::cmd("SCAN");
        // If we have a cursor (from a previous execution), set it
        // Annoyingly, the redis lib does not allow to set a reference to a vec as argument, so
        // we take here, but that's fine, since a result will update the cursor anyway.
        if let Some(cur) = self.cursor.take() {
            scan_cmd.arg(cur);
        }
        let res: (Vec<u8>, Vec<ScanEntry>) = match scan_cmd.query(&mut *self.conn) {
            Ok(r) => Some(r),
            Err(_) => None,
        }?;

        // set the new cursor
        self.cursor = Some(res.0);
        // set the buffer
        self.buffer = res.1;
        // reset buffer pointer, set it to one as we will return the first element
        self.buffer_idx = 1;

        Some(read_le_key(&self.buffer[0][0].0))
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
        let mut conn = self.client.get_connection()?;
        if let Some(ref ns) = self.namespace {
            let namespaces: Vec<String> = redis::cmd("NSLIST").query(&mut conn)?;
            let mut exists = false;
            for existing_ns in namespaces {
                if &existing_ns == ns {
                    exists = true;
                    break;
                }
            }
            if !exists {
                redis::cmd("NSNEW").arg(ns).query(&mut conn)?;
            }
            redis::cmd("SELECT").arg(ns).query(&mut conn)?;
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
    fn from(_: r2d2::Error) -> Self {
        StorageError::IO(None)
    }
}
