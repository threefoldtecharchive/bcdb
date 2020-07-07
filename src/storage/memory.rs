use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::{Error, Key, Record, Storage};

#[derive(Debug, Clone)]
pub struct MemoryStorage {
    internal: Arc<RwLock<Internal>>,
}

#[derive(Debug)]
struct Internal {
    key_counter: Key,
    backend: HashMap<Key, Vec<u8>>,
}

impl MemoryStorage {
    pub fn new() -> MemoryStorage {
        let key_counter = 0;
        let backend = HashMap::new();
        MemoryStorage {
            internal: Arc::new(RwLock::new(Internal {
                key_counter,
                backend,
            })),
        }
    }
}

impl Storage for MemoryStorage {
    fn set(&mut self, key: Option<Key>, data: &[u8]) -> Result<Key, Error> {
        let mut handle = self.internal.write().unwrap();
        let key = match key {
            Some(key) => key,
            None => {
                let key = handle.key_counter;
                handle.key_counter += 1;
                key
            }
        };
        handle.backend.insert(key, Vec::from(data));
        Ok(key)
    }

    fn get(&mut self, key: Key) -> Result<Option<Vec<u8>>, Error> {
        let handle = self.internal.read().unwrap();
        match handle.backend.get(&key) {
            Some(data) => Ok(Some(data.clone())),
            None => Ok(None),
        }
    }

    fn keys(&mut self) -> Result<Box<dyn Iterator<Item = Record> + Send>, Error> {
        let handle = self.internal.read().unwrap();
        Ok(Box::new(
            handle
                .backend
                .keys()
                .copied()
                .collect::<Vec<Key>>()
                .into_iter()
                .map(|v| Record {
                    key: v,
                    timestamp: None,
                    size: None,
                }),
        ))
    }

    fn rev(&mut self) -> Result<Box<dyn Iterator<Item = Record> + Send>, Error> {
        let handle = self.internal.read().unwrap();
        Ok(Box::new(
            handle
                .backend
                .keys()
                .copied()
                .collect::<Vec<Key>>()
                .into_iter()
                .map(|v| Record {
                    key: v,
                    timestamp: None,
                    size: None,
                })
                .rev(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_storage() {
        let mut storage = MemoryStorage::new();

        let key1 = storage.set(None, &[0, 1, 2, 3]).unwrap();
        let key2 = storage.set(None, &[1, 2, 3, 4]).unwrap();
        let key3 = storage.set(None, &[2, 3, 4, 5]).unwrap();

        assert_eq!(key1, 0);
        assert_eq!(key2, 1);
        assert_eq!(key3, 2);

        assert_eq!(Some(vec![0, 1, 2, 3]), storage.get(key1).unwrap());
        assert_eq!(Some(vec![1, 2, 3, 4]), storage.get(key2).unwrap());
        assert_eq!(Some(vec![2, 3, 4, 5]), storage.get(key3).unwrap());

        // overwrite key
        let key4 = storage.set(Some(key1), &[3, 4, 5, 6]).unwrap();
        assert_eq!(key1, key4);
        assert_eq!(Some(vec![3, 4, 5, 6]), storage.get(key1).unwrap());

        // check for nonexisting keys
        assert_eq!(None, storage.get(3).unwrap());
        assert_eq!(None, storage.get(32_413_214).unwrap());
        assert_eq!(None, storage.get(17).unwrap());

        // key equality
        let mut keys = storage.keys().unwrap().collect::<Vec<_>>();
        keys.sort();
        assert_eq!(keys, vec![key1, key2, key3]);
    }
}
