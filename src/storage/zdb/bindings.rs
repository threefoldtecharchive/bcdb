#![allow(dead_code)] // TODO: see if we can reduce generated binding size
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

include!(concat!(env!("OUT_DIR"), "/libzdb_bindings.rs"));
