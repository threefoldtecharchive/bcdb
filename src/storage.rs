pub mod zdb;

use std::{fmt, io};

/// Key as they are expected by the Storage interface. For now, the key is expecteto be 4 bytes
pub type Key = u32;

/// The generic set op instructions that are suported by storage implementations
pub trait Storage {
    /// Set some data, returning a generated key which can later be used to retrieve the data
    fn set(&mut self, data: &[u8]) -> Result<Key, Error>;
    /// Get data which has been set previously.
    fn get(&mut self, key: Key) -> Result<Option<Vec<u8>>, Error>;
}

#[derive(Debug)]
pub enum Error {
    /// An IO error has occured - an error on the connection to the remote Zdb
    IO(Option<io::Error>),
    /// An error in the redis protocol
    Protocol(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(e) => match e {
                Some(e) => write!(f, "{}", e),
                None => write!(f, "Zdb IO Error"),
            },
            Error::Protocol(e) => write!(f, "protocol error: {}", e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(Some(e))
    }
}

impl std::error::Error for Error {}
