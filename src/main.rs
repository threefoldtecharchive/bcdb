#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;

fn main() {
    println!("Hello, world!");

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
        //let data = CString::new("Hello from dbreboot").unwrap().into_raw();
        let data = "Hello from dbreboot\0".as_ptr();
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

include!(concat!(env!("OUT_DIR"), "/libzdb_bindings.rs"));
