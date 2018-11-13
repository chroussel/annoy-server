extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .cpp(true)
        .include("src_cpp")
        .file("src_cpp/annoyrust.cpp")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-deprecated")
        .compile("annoy");

    let bindings = bindgen::Builder::default()
        .rustfmt_bindings(true)
        .header("src_cpp/annoyrust.h")
        .opaque_type("rust_annoy_index_t")
        .opaque_type(".*_vector")
        .whitelist_recursively(false)
        .whitelist_type("rust_annoy_index_t")
        .whitelist_type("f_vector")
        .whitelist_type("i_vector")
        .whitelist_function(".*_vector_.*")
        .whitelist_function("rust_annoy_.*")
        .clang_arg(r"-xc++")
        .clang_arg(r"-lstdc++")
        .layout_tests(false)
        .derive_copy(false)
        .clang_arg(r"-Iinclude")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
