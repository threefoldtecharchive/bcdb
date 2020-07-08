use crate::storage::{Key, Storage};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

const READ: u32 = 0x4;
const WRITE: u32 = 0x2;
const DELETE: u32 = 0x1;
const SYNTAX: &str = "rwd";

/// Permissions bits. stores the value of current
/// permissiones set associated with an acl
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Permissions(u32);

impl Permissions {
    /// set the read permissions bit
    #[allow(dead_code)]
    pub fn set_read(self, t: bool) -> Self {
        self.set(READ, t)
    }

    /// set the write permissions bit
    #[allow(dead_code)]
    pub fn set_write(self, t: bool) -> Self {
        self.set(WRITE, t)
    }

    /// set the delete permissions bit
    #[allow(dead_code)]
    pub fn set_delete(self, t: bool) -> Self {
        self.set(DELETE, t)
    }

    /// checks the read permissions bit
    pub fn is_read(&self) -> bool {
        self.get(READ)
    }

    /// checks the write permissions bit
    pub fn is_write(&self) -> bool {
        self.get(WRITE)
    }

    /// checks the delete permissions bit
    pub fn is_delete(&self) -> bool {
        self.get(DELETE)
    }

    pub fn grants(&self, p: Permissions) -> bool {
        (self.0 & p.0) == p.0
    }

    fn set(self, bit: u32, t: bool) -> Self {
        let v = match t {
            true => self.0 | bit,
            false => self.0 & !bit,
        };

        Self(v)
    }

    fn get(&self, bit: u32) -> bool {
        self.0 & bit > 0
    }
}

impl std::str::FromStr for Permissions {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 3 {
            bail!(
                "invalid format expecting format '{}' replace empty perm with a dash '-'",
                SYNTAX
            );
        }

        let p = Ok(Permissions::default());

        let p = s.chars().zip(SYNTAX.chars()).fold(p, |p, (v, c)| match p {
            Ok(p) => {
                let p = if v == c {
                    p.0 << 1 | 1
                } else if v == '-' {
                    p.0 << 1
                } else {
                    bail!("invalid char '{}'", v);
                };
                Ok(Permissions(p))
            }
            Err(err) => Err(err),
        });

        p
    }
}

impl std::fmt::Display for Permissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            match self.is_read() {
                true => "r",
                false => "-",
            },
            match self.is_write() {
                true => "w",
                false => "-",
            },
            match self.is_delete() {
                true => "d",
                false => "-",
            }
        )
    }
}
// ACL object structure
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ACL {
    pub perm: Permissions,
    pub users: Vec<u64>,
}

impl From<Permissions> for ACL {
    fn from(p: Permissions) -> ACL {
        ACL {
            perm: p,
            users: vec![],
        }
    }
}

#[derive(Clone)]
pub struct ACLStorage<S>
where
    S: Storage,
{
    inner: S,
}

/**
 * Storage is a wrapper around a raw Storage to easily set, get, and list
 * ACL objects
 */
impl<S> ACLStorage<S>
where
    S: Storage,
{
    pub fn new(storage: S) -> ACLStorage<S> {
        ACLStorage { inner: storage }
    }

    /// Creates a new ACL and return the key
    pub fn create(&mut self, acl: &ACL) -> Result<Key> {
        let bytes = serde_json::to_vec(acl)?;
        let key = self.inner.set(None, &bytes)?;
        Ok(key)
    }

    /// Get an ACL with key
    pub fn get(&mut self, key: Key) -> Result<Option<ACL>> {
        let bytes = self.inner.get(key)?;
        match bytes {
            None => Ok(None),
            Some(bytes) => {
                let acl: ACL = serde_json::from_slice(&bytes)?;
                Ok(Some(acl))
            }
        }
    }

    /// Overrides a value of an ACL
    pub fn update(&mut self, key: Key, acl: &ACL) -> Result<()> {
        let bytes = serde_json::to_vec(acl)?;
        self.inner.set(Some(key), &bytes)?;
        Ok(())
    }

    /// iterates over all configured ACLs
    pub fn list<'a>(&'a mut self) -> Result<impl Iterator<Item = Result<(Key, ACL)>> + 'a> {
        Ok(self
            .inner
            .keys()?
            .filter_map(move |r| match self.get(r.key) {
                Ok(acl) => match acl {
                    Some(acl) => Some(Ok((r.key, acl))),
                    None => None,
                },
                Err(err) => Some(Err(err)),
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn permissions() {
        let p = Permissions::default()
            .set_read(true)
            .set_write(true)
            .set_delete(false);

        assert_eq!(true, p.is_read());
        assert_eq!(true, p.is_write());
        assert_eq!(false, p.is_delete());
        assert_eq!("rw-", p.to_string());
    }

    #[test]
    fn grants() {
        let p: Permissions = "rw-".parse().unwrap();

        assert_eq!(true, p.grants("r--".parse().unwrap()));
        assert_eq!(true, p.grants("-w-".parse().unwrap()));
        assert_eq!(true, p.grants("rw-".parse().unwrap()));
        assert_eq!(false, p.grants("r-d".parse().unwrap()));
        assert_eq!(false, p.grants("--d".parse().unwrap()));
    }

    #[test]
    fn permissions_parse() {
        let p: Permissions = "rwd".parse().expect("failed to parse");

        assert_eq!(p.is_read(), true);
        assert_eq!(p.is_write(), true);
        assert_eq!(p.is_delete(), true);

        let p: Permissions = "r-d".parse().expect("failed to parse");

        assert_eq!(p.is_read(), true);
        assert_eq!(p.is_write(), false);
        assert_eq!(p.is_delete(), true);

        let p: Permissions = "r-d".parse().expect("failed to parse");

        assert_eq!(p.is_read(), true);
        assert_eq!(p.is_write(), false);
        assert_eq!(p.is_delete(), true);

        let p: Permissions = "--d".parse().expect("failed to parse");

        assert_eq!(p.is_read(), false);
        assert_eq!(p.is_write(), false);
        assert_eq!(p.is_delete(), true);

        let p: Permissions = "rw-".parse().expect("failed to parse");

        assert_eq!(p.is_read(), true);
        assert_eq!(p.is_write(), true);
        assert_eq!(p.is_delete(), false);
    }

    #[test]
    fn storage_default() {
        use crate::storage::memory::MemoryStorage;
        let db = MemoryStorage::new();
        let mut storage = ACLStorage::new(db);

        let key = storage
            .create(&ACL::default())
            .expect("failed to create acl object");
        let acl = storage
            .get(key)
            .expect("failed to get acl")
            .expect("got nil value");

        assert_eq!(acl.perm, Permissions::default());
        assert_eq!(acl.users.len(), 0);
    }

    #[test]
    fn storage_custom() {
        use crate::storage::memory::MemoryStorage;
        let db = MemoryStorage::new();
        let mut storage = ACLStorage::new(db);
        let mut acl = ACL::from(Permissions::default().set_read(true));
        acl.users.push(100);

        let key = storage.create(&acl).expect("failed to create acl object");
        let acl = storage
            .get(key)
            .expect("failed to get acl")
            .expect("got nil value");

        assert_eq!(acl.perm, Permissions::default().set_read(true));
        assert_eq!(acl.users.len(), 1);
        assert_eq!(acl.users[0], 100);
    }

    #[test]
    fn storage_list() {
        use crate::storage::memory::MemoryStorage;
        let db = MemoryStorage::new();
        let mut storage = ACLStorage::new(db);

        let mut acl = ACL::from(Permissions::default().set_read(true));
        acl.users.push(100);

        let key = storage.create(&acl).expect("failed to create acl object");
        let (k, v) = storage
            .list()
            .expect("failed to list")
            .next()
            .expect("failed to get next value")
            .unwrap();

        assert_eq!(k, key);

        assert_eq!(v.perm, acl.perm);
        assert_eq!(v.users.len(), 1);
        assert_eq!(v.users[0], 100);
    }
}
