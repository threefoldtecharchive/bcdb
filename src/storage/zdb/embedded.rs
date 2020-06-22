mod bindings;

use bindings::*;

use std::ffi::{c_void, CStr, CString};
use std::io;
use std::mem;
use std::os::raw::c_char;
use std::ptr;

/// A wrapper around libzdb, `Zdb` exposes a safe interface to the database
#[derive(Debug)]
pub struct Zdb {
    settings: *mut zdb_settings_t,
    namespace: *mut namespace_t, // TODO: create separate namespace structs
}

impl Zdb {
    /// Create a new zdb with default settings.
    /// This will later be refactored to make settings a separate struct
    pub fn new() -> Zdb {
        let mut settings;
        let namespace;
        // this is safe as it takes no parameters, and always returns a valid structure
        unsafe {
            settings = zdb_initialize();
            (*settings).datapath = "./tmp/zdb/data".as_ptr() as *mut c_char;
            (*settings).indexpath = "./tmp/zdb/index".as_ptr() as *mut c_char;
            // TODO: set sequential mode
            settings = zdb_open(settings);
            namespace = namespace_get_default();
        }

        Zdb {
            settings,
            namespace,
        }
    }
}

impl super::Storage for Zdb {
    fn set(&self, data: &[u8]) -> io::Result<super::Key> {
        let reply;
        unsafe {
            reply = zdb_api_set(
                self.namespace,
                0 as *mut c_void, // TODO: can this be a null pointer, or should it be??
                0,
                data.as_ptr() as *mut c_void, // the data slice must outlive this pointer.
                data.len() as u64,
            );
        }
        let key_bytes: [u8; 4];
        let key: super::Key;
        // TODO: check for errors
        unsafe {
            match (*reply).status {
                zdb_api_type_t::ZDB_API_BUFFER => {
                    let entry = (*reply).payload as *mut zdb_api_buffer_t;
                    let key_size = (*entry).size;
                    // TODO: double check key size 32 vs 64
                    assert!((*entry).size == mem::size_of::<[u8; 4]>() as u64);
                    // we need to make sure not to take ownership of this data
                    // TODO: quadruple check this cast
                    // TODO: do we need to convert endiannes??
                    // // TODO: can we use ptr::copy?
                    // let mut data = Vec::with_capacity(4);
                    // data.sen_len(4)
                    // ptr::copy_nonoverlapping((*entry).payload, data.as_mut_ptr(), 4);
                    // key = u32::from_le_bytes(data);
                    key = ptr::read((*entry).payload as *const u32);
                }
                _ => unimplemented!(),
            }
        }
        // make sure we clean up
        unsafe {
            zdb_api_reply_free(reply);
        }

        Ok(key)
    }

    fn get(&self, key: super::Key) -> io::Result<Option<Vec<u8>>> {
        unimplemented!();
    }
}

impl Drop for Zdb {
    fn drop(&mut self) {
        // this is safe because settings is a field which is not exported, and we initialize
        // it when we create the struct.
        unsafe {
            zdb_close(self.settings);
        }
    }
}

// TODO: verify this
// impl !std::marker::Sync for Zdb {}
// impl !std::marker::Send for Zdb {}

fn do_something() {
    unsafe {
        let settings = zdb_initialize();

        let mut id = String::from("testing\0");
        zdb_id_set(id.as_mut_ptr() as *mut c_char);

        println!("[+] ZDB instance ID: {}", zdb_instanceid_get());

        (*settings).datapath = CString::new("/tmp/zdb-example/data").unwrap().into_raw();
        (*settings).indexpath = CString::new("/tmp/zdb-example/index").unwrap().into_raw();

        zdb_open(settings);

        let ns = namespace_get_default();
        // write stoof
        // let key = CString::new("testkey").unwrap().into_raw();
        let key = "testkey\0".as_ptr();
        //let data = CString::new("Hello from bcdb").unwrap().into_raw();
        let data = "Hello from bcdb\0".as_ptr();
        let reply = zdb_api_set(ns, key as *mut c_void, 7, data as *mut c_void, 20);
        dump_type((*reply).status);
        zdb_api_reply_free(reply);

        // ensure key exists
        println!("Check keys existence");
        let reply = zdb_api_exists(ns, key as *mut c_void, 7);
        dump_type((*reply).status);
        zdb_api_reply_free(reply);

        // now read stoof
        let reply = zdb_api_get(ns, key as *mut c_void, 7);
        dump_type((*reply).status);

        if (*reply).status == zdb_api_type_t::ZDB_API_ENTRY {
            let entry = (*reply).payload as *mut zdb_api_entry_t;
            println!("entry: key {:?}", (*entry).key.payload);
            println!("entry: enry {:?}", (*entry).payload.payload);
            let actual_value = CStr::from_ptr((*entry).payload.payload as *mut c_char);
            println!("entry: entry: {}", actual_value.to_str().unwrap());
        }
        // this line here triggers a double free error, no clue why
        zdb_api_reply_free(reply);

        zdb_close(settings);
    }
}

unsafe fn dump_type(status: zdb_api_type_t) {
    let msg = zdb_api_debug_type(status);
    println!("Example: {:?}", msg);
}
