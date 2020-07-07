use aead::{generic_array::GenericArray, Aead, NewAead};
use aes_gcm::{aead, Aes256Gcm};
use rand::prelude::*;
use rand::rngs::OsRng;

use super::{Error as StorageError, Key, Record, Storage};

const ENCRYPTION_KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;

#[derive(Clone)]
pub struct EncryptedStorage<S> {
    cipher: Aes256Gcm,
    nonce_source: OsRng,
    backend: S,
}

impl<S> EncryptedStorage<S>
where
    S: Storage,
{
    /// Create a new encrypted storage instance from an existing storage instance, with the given
    /// key. All data written to the storage backend will be encrypted with the provided key.
    /// Likewise, all data comming from the storage backend will be decrypted with the provided
    /// key. The cryptographic algorithm used is AES in GCM mode, with a 256 bit key.
    ///
    /// # panics
    ///
    /// This function will panic if the provided key is not 32 bytes long.
    pub fn new(key: &[u8], backend: S) -> Self {
        assert_eq!(key.len(), ENCRYPTION_KEY_SIZE);
        let key = GenericArray::clone_from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce_source = OsRng;
        EncryptedStorage {
            cipher,
            nonce_source,
            backend,
        }
    }
}

impl<S> Storage for EncryptedStorage<S>
where
    S: Storage,
{
    fn set(&mut self, key: Option<Key>, data: &[u8]) -> Result<Key, StorageError> {
        let mut nonce_slice = vec![0u8; NONCE_SIZE];
        self.nonce_source.fill_bytes(&mut nonce_slice);
        let nonce = GenericArray::clone_from_slice(&nonce_slice);
        let mut ciphertext = self.cipher.encrypt(&nonce, data)?;
        let mut final_data = nonce.to_vec();
        final_data.append(&mut ciphertext);
        self.backend.set(key, &final_data)
    }

    fn get(&mut self, key: Key) -> Result<Option<Vec<u8>>, StorageError> {
        let data = match self.backend.get(key)? {
            Some(data) => data,
            None => return Ok(None),
        };
        let nonce = GenericArray::clone_from_slice(&data[..NONCE_SIZE]);
        let plaintext = self.cipher.decrypt(&nonce, &data[NONCE_SIZE..])?;
        Ok(Some(plaintext))
    }

    fn keys(&mut self) -> Result<Box<dyn Iterator<Item = Record> + Send>, StorageError> {
        self.backend.keys()
    }

    fn rev(&mut self) -> Result<Box<dyn Iterator<Item = Record> + Send>, StorageError> {
        self.backend.rev()
    }
}

impl From<aead::Error> for StorageError {
    fn from(_: aead::Error) -> StorageError {
        StorageError::Crypto
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let storage = crate::storage::memory::MemoryStorage::new();

        let mut encryption_key = vec![0; 32];
        rand::thread_rng().fill_bytes(&mut encryption_key);

        let mut crypt = EncryptedStorage::new(&encryption_key, storage);

        let data1 = b"First piece of data";
        let data2 = b"Second piece of data";
        let data3 = b"Some super secret data nobody should read";

        let key1 = crypt.set(None, data1).unwrap();
        let key2 = crypt.set(None, data2).unwrap();
        let key3 = crypt.set(None, data3).unwrap();

        let recovered_data_1 = crypt.get(key1).unwrap();
        let recovered_data_2 = crypt.get(key2).unwrap();
        let recovered_data_3 = crypt.get(key3).unwrap();

        assert_eq!(Some(Vec::from(&data1[..])), recovered_data_1);
        assert_eq!(Some(Vec::from(&data2[..])), recovered_data_2);
        assert_eq!(Some(Vec::from(&data3[..])), recovered_data_3);

        assert_eq!(None, crypt.get(3).unwrap());
        assert_eq!(None, crypt.get(17).unwrap());
        assert_eq!(None, crypt.get(17_343_525).unwrap());
    }
}
