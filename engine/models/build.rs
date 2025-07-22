fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let lib_path = "/home/vp/quality_control_room/lib";
    println!("cargo:rustc-link-search=native={}", lib_path);
    println!("cargo:rustc-link-lib=dylib=xgbwrapper");
    println!("cargo:rustc-link-lib=dylib=xgboost");
    // Embed library path in binary
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path);
}
