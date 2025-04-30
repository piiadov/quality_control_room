fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-search=native=/home/vp/quality_control_room/lib");
    println!("cargo:rustc-link-lib=dylib=xgbwrapper");
    println!("cargo:rustc-link-search=native=/home/vp/quality_control_room/lib");
    println!("cargo:rustc-link-lib=dylib=xgboost");
}
