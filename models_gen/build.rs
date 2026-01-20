fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Link only to xgbwrapper - it internally handles xgboost linkage
    let lib_path = "/home/vp/quality_control_room/lib";
    println!("cargo:rustc-link-search=native={}", lib_path);
    println!("cargo:rustc-link-lib=dylib=xgbwrapper");
    
    // Set RPATH so the binary can find shared libraries at runtime
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path);
}
