use crate::storage::{Error, Key, Storage};

type Result<T> = std::result::Result<T, Error>;

const read: u32 = 0x4;
const write: u32 = 0x2;
const delete: u32 = 0x1;

/// Permissions bits. stores the value of current
/// permissiones set associated with an acl
#[derive(Default)]
pub struct Permissions(u32);

impl Permissions {
    /// set the read permissions bit
    pub fn set_read(self, t: bool) -> Self {
        self.set(read, t)
    }
    /// set the write permissions bit
    pub fn set_write(self, t: bool) -> Self {
        self.set(write, t)
    }
    /// set the delete permissions bit
    pub fn set_delete(self, t: bool) -> Self {
        self.set(delete, t)
    }

    /// checks the read permissions bit
    pub fn is_read(&self) -> bool {
        self.get(read)
    }

    /// checks the write permissions bit
    pub fn is_write(&self) -> bool {
        self.get(write)
    }

    /// checks the delete permissions bit
    pub fn is_delete(&self) -> bool {
        self.get(delete)
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

// ACL object structure
pub struct ACL {
    perm: Permissions,
    users: Vec<u64>,
}

struct ACLStorage<S>
where
    S: Storage,
{
    inner: S,
}

impl<S: Storage> ACLStorage<S> {
    fn create(&mut self, acl: ACL) -> Result<Key> {
        Ok(0)
    }

    fn update(&mut self, key: Key, acl: ACL) -> Result<()> {
        Ok(())
    }

    fn list(&self) {
        // not sure yet what to return
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
    }
}
