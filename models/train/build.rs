use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-search=native=/home/vp/GitHub/quality_control_room/lib");
    println!("cargo:rustc-link-lib=dylib=xgbwrapper");
    println!("cargo:rustc-link-search=native=/home/vp/GitHub/quality_control_room/lib");
    println!("cargo:rustc-link-lib=dylib=xgboost");

    // Define the source and destination paths for the .so files
    let lib_dir = PathBuf::from("/home/vp/GitHub/quality_control_room/lib");
    let xgbwrapper_src = lib_dir.join("libxgbwrapper.so");
    let xgboost_src = lib_dir.join("libxgboost.so");

    let bin_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("target")
        .join(env::var("PROFILE").unwrap());

    let xgbwrapper_dst = bin_dir.join("libxgbwrapper.so");
    let xgboost_dst = bin_dir.join("libxgboost.so");

    fs::copy(xgbwrapper_src, xgbwrapper_dst).expect("Failed to copy libxgbwrapper.so");
    fs::copy(xgboost_src, xgboost_dst).expect("Failed to copy libxgboost.so");

}