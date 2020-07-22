pub mod encrypted;
pub mod zdb;

#[cfg(test)]
pub mod memory;

use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::{fmt, io};

/// Key as they are expected by the Storage interface. For now, the key is expect to be 4 bytes
pub type Key = u32;

/// Iteration record
#[derive(Eq)]
pub struct Record {
    pub key: Key,
    pub timestamp: Option<u32>,
    pub size: Option<u32>,
}

impl Ord for Record {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

/// The generic set op instructions that are suported by storage implementations
pub trait Storage: Clone {
    /// Set some data, returning a generated key which can later be used to retrieve the data
    /// The caller can optionally provide a previously returned key. If such a key is provided,
    /// the data previously attached to this key will be replaced by the new data.
    fn set(&self, key: Option<Key>, data: &[u8]) -> Result<Key, Error>;
    /// Get data which has been set previously.
    fn get(&self, key: Key) -> Result<Option<Vec<u8>>, Error>;
    /// Get an iterator over all keys in a collection
    fn keys(&self) -> Result<Box<dyn Iterator<Item = Record> + Send>, Error>;
    /// Get an iterator over all keys in a collection, in reverse order
    fn rev(&self) -> Result<Box<dyn Iterator<Item = Record> + Send>, Error>;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An IO error has occured - an error on the connection to the remote Zdb
    IO(Option<io::Error>),
    /// An error in the redis protocol
    Protocol(String),
    /// A cryptographic error occurred, this can only happen if the storage supports encryption
    Crypto,
    /// A generic error
    Other,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(e) => match e {
                Some(e) => write!(f, "{}", e),
                None => write!(f, "Zdb IO Error"),
            },
            Error::Protocol(e) => write!(f, "protocol error: {}", e),
            Error::Crypto => write!(f, "cryptographic error"),
            Error::Other => write!(f, "Unknown error"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(Some(e))
    }
}

impl std::error::Error for Error {}
