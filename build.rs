extern crate protoc_rust_grpc;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// arguments to pass to `clang`, these are taken from the libzdb makefile
const CLANG_ARGS: [&str; 9] = [
    "-g",
    "-fPIC",
    "-std=gnu11",
    "-O0",
    "-W",
    "-Wall",
    "-Wextra",
    "-msse4.2",
    "-Wno-implicit-fallthrough",
];

fn main() {
    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src/api",
        includes: &[],
        input: &["api.proto"],
        rust_protobuf: true,
        ..Default::default()
    })
    .expect("protoc-rust-grpc");

    // invalidate build if something in the libzdb dir changes
    println!("cargo:rerun-if-changed=libzdb,api.proto");

    // generate static library

    // collect all .c files
    let c_files: Vec<PathBuf> = fs::read_dir("libzdb")
        .expect("Failed to read files in libzdbd")
        .filter(|res| {
            res.as_ref()
                .expect("Failed to get path entry")
                .path()
                .as_path()
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                == "c"
        })
        .map(|entry| entry.unwrap().file_name().to_str().unwrap().to_owned())
        .map(|path| Path::new("libzdb").join(path))
        .collect();

    // build static lib
    let mut cc = cc::Build::new();
    cc.include("libzdb").files(c_files);

    for flag in &CLANG_ARGS {
        cc.flag(flag);
    }
    cc.static_flag(true);
    cc.no_default_flags(true);
    cc.compile("libzdb.a");

    // now generate bindings
    let bindings = bindgen::Builder::default()
        // use manual wrapper to include <sys/types.h>, adding this as a separate header does
        // not seem to work
        .header("libzdb/wrapper.h")
        .clang_args(&CLANG_ARGS)
        .rustified_enum("zdb_api_type_t")
        // rerun when headers change
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate libzdb bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("libzdb_bindings.rs"))
        .expect("Couldn't write libzdb bindings");
}
