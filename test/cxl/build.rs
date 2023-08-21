use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=../../../../build_gcc_debug/bin");
    println!("cargo:rustc-link-lib=ralloc_cxl");
    println!("cargo:rustc-link-lib=cxl_driver_api");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=atomic");
    println!("cargo:rustc-link-lib=asan");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_function("RP_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out.join("bindings.rs"))
        .expect("Failed to write bindings");
}
