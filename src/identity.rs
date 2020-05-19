mod ed25519;

pub use ed25519::*;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An error during signature validation
    InvalidSignature,
    /// A private key which is not of the correct length
    MalformedPrivateKey,
    /// A public key which is not of the correct length
    MalformedPublicKey,
    /// A signature which is not of the correct length
    MalformedSignature,
    /// IOError
    IOError(std::io::Error),

    Other(failure::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidSignature => write!(f, "Invalid signature"),
            Error::MalformedPrivateKey => write!(f, "Private key has the wrong length"),
            Error::MalformedPublicKey => write!(f, "Public key has the wrong length"),
            Error::MalformedSignature => write!(f, "Signature has the wrong length"),
            Error::IOError(err) => write!(f, "Input Output Error: {}", err),
            Error::Other(err) => write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}
impl From<failure::Error> for Error {
    fn from(err: failure::Error) -> Error {
        Error::Other(err)
    }
}
impl std::error::Error for Error {}
