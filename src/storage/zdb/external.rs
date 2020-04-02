use crate::storage::{Error as StorageError, Key, Storage};

use redis::RedisError;

use std::convert::TryInto;
use std::io;

pub struct Zdb {
    client: redis::Client,
    // upon connection we are always connected to the default namespace
    default_namespace: Collection,
}

struct Collection {
    conn: redis::Connection,
}

impl Zdb {
    pub fn new(port: u16) -> Zdb {
        let client = redis::Client::open(format!("redis://localhost:{}", port))
            .expect("Could not connect to zdb");
        let default_namespace = Collection {
            conn: client.get_connection().expect("could not get connection"),
        };
        Zdb {
            client,
            default_namespace,
        }
    }
}

impl Storage for Zdb {
    fn set(&mut self, data: &[u8]) -> Result<Key, StorageError> {
        self.default_namespace.set(data)
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

impl Clone for Zdb {
    /// create a new instance of an existing zdb. This opens a new connection to the given instance.
    fn clone(&self) -> Zdb {
        Zdb {
            client: self.client.clone(),
            default_namespace: Collection {
                conn: self
                    .client
                    .get_connection()
                    .expect("failed to open new zdb connection"),
            },
        }
    }
}

impl Storage for Collection {
    fn set(&mut self, data: &[u8]) -> Result<Key, StorageError> {
        let raw_key: Vec<u8> = redis::cmd("SET").arg("").arg(data).query(&mut self.conn)?;
        debug_assert!(raw_key.len() == std::mem::size_of::<Key>());
        Ok(read_le_key(&raw_key))
    }

    fn get(&mut self, key: Key) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(redis::cmd("GET")
            .arg(&key.to_le_bytes())
            .query(&mut self.conn)?)
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

impl From<RedisError> for StorageError {
    fn from(e: RedisError) -> Self {
        if e.is_io_error() {
            return StorageError::IO(None);
        }
        StorageError::Protocol(e.category().to_owned())
    }
}
