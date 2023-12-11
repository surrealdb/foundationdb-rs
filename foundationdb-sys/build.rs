extern crate bindgen;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[cfg(all(not(feature = "embedded-fdb-include"), target_os = "linux"))]
const INCLUDE_PATH: &str = "-I/usr/include/foundationdb/";

#[cfg(all(not(feature = "embedded-fdb-include"), target_os = "macos"))]
const INCLUDE_PATH: &str = "-I/usr/local/include/foundationdb/";

#[cfg(all(not(feature = "embedded-fdb-include"), target_os = "windows"))]
const INCLUDE_PATH: &str = "-IC:/Program Files/foundationdb/include/foundationdb";

#[cfg(all(feature = "embedded-fdb-include", feature = "fdb-5_1"))]
const INCLUDE_PATH: &str = "-I./include/510";
#[cfg(all(feature = "embedded-fdb-include", feature = "fdb-5_2"))]
const INCLUDE_PATH: &str = "-I./include/520";
#[cfg(all(feature = "embedded-fdb-include", feature = "fdb-6_0"))]
const INCLUDE_PATH: &str = "-I./include/600";
#[cfg(all(feature = "embedded-fdb-include", feature = "fdb-6_1"))]
const INCLUDE_PATH: &str = "-I./include/610";
#[cfg(all(feature = "embedded-fdb-include", feature = "fdb-6_2"))]
const INCLUDE_PATH: &str = "-I./include/620";
#[cfg(all(feature = "embedded-fdb-include", feature = "fdb-6_3"))]
const INCLUDE_PATH: &str = "-I./include/630";
#[cfg(all(feature = "embedded-fdb-include", feature = "fdb-7_0"))]
const INCLUDE_PATH: &str = "-I./include/700";
#[cfg(all(feature = "embedded-fdb-include", feature = "fdb-7_1"))]
const INCLUDE_PATH: &str = "-I./include/710";

fn main() {
    println!("cargo:rerun-if-env-changed=FDB_CLIENT_LIB_PATHS");
    println!("cargo:rerun-if-env-changed=FDB_CLIENT_STATIC");


    if let Ok(lib_path) = env::var("FDB_CLIENT_LIB_PATHS") {
        lib_path.split(":").for_each(|path| {
            println!("cargo:rustc-link-search=native={}", path);
        });
    }

    // Link against fdb_c
    if env::var_os(format!("FDB_CLIENT_STATIC")).is_some() {
        println!("cargo:rustc-link-lib=static=crypto");
        println!("cargo:rustc-link-lib=static=ssl");
        println!("cargo:rustc-link-lib=static=eio");
        println!("cargo:rustc-link-lib=static=fdb_c");
        println!("cargo:rustc-link-lib=static=fdbclient");
        println!("cargo:rustc-link-lib=static=fdbrpc");
        println!("cargo:rustc-link-lib=static=flow");
        println!("cargo:rustc-link-lib=static=fmt");
        println!("cargo:rustc-link-lib=static=stacktrace");
    } else {
        println!("cargo:rustc-link-lib=dylib=fdb_c");
    }


    // Include the link directory for the .lib file on windows (which will resolve to
    // the shared library, at runtime)
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-search=C:/Program Files/foundationdb/lib/foundationdb");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not defined!"));

    // We need to have FDB_API_VERSION set to a constant so that bindgen will
    // generate a const value for it. We could try to pass -DFDB_TRICKY_VERSION=510
    // to the driver and then '#define FDB_API_VERSION FDB_TRICKY_VERSION', but
    // bindgen isn't smart enough to resolve that from the arguments. Instead, write
    // out a src/wrapper.h file with the chosen version instead.
    let api_version;

    #[cfg(feature = "fdb-5_1")]
    {
        api_version = 510;
    }
    #[cfg(feature = "fdb-5_2")]
    {
        api_version = 520;
    }
    #[cfg(feature = "fdb-6_0")]
    {
        api_version = 600;
    }
    #[cfg(feature = "fdb-6_1")]
    {
        api_version = 610;
    }
    #[cfg(feature = "fdb-6_2")]
    {
        api_version = 620;
    }
    #[cfg(feature = "fdb-6_3")]
    {
        api_version = 630;
    }
    #[cfg(feature = "fdb-7_0")]
    {
        api_version = 700;
    }
    #[cfg(feature = "fdb-7_1")]
    {
        api_version = 710;
    }

    // Sigh, bindgen only takes a String for its header path, but that's UTF-8 while
    // PathBuf is OS-native...
    let wpath = out_path.join("wrapper.h");
    let wrapper_path = wpath
        .to_str()
        .expect("couldn't convert wrapper PathBuf to String!");

    let mut wrapper = File::create(wrapper_path).expect("couldn't create wrapper.h!");
    wrapper
        .write_all(format!("#define FDB_API_VERSION {}\n", api_version).as_bytes())
        .expect("couldn't write wrapper.h!");
    wrapper
        .write_all(b"#include <fdb_c.h>\n")
        .expect("couldn't write wrapper.h!");
    drop(wrapper);

    // Finish up by writing the actual bindings
    let bindings = bindgen::Builder::default()
        // TODO: there must be a way to get foundationdb from pkg-config...
        .clang_arg(INCLUDE_PATH)
        .header(wrapper_path)
        .generate_comments(true)
        .layout_tests(false)
        .generate()
        .expect("Unable to generate FoundationDB bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
