//! Build script for server
//!
//! Links against xgbwrapper library.

fn main() {
    // Library search path
    let lib_path = std::env::var("CARGO_MANIFEST_DIR")
        .map(|p| format!("{}/../lib", p))
        .unwrap_or_else(|_| "../lib".into());

    println!("cargo:rustc-link-search=native={}", lib_path);
    println!("cargo:rustc-link-lib=dylib=xgbwrapper");
    
    // Rebuild if library changes
    println!("cargo:rerun-if-changed=../lib/libxgbwrapper.so");
}
