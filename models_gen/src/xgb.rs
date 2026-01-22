//! XGBoost FFI bindings for models_gen
//!
//! Minimal FFI wrapper for xgbw_train_eval - the only function needed for model generation.

use std::ffi::{c_char, c_float, c_int, CStr, CString};
use std::os::raw::c_ulong;

/// Key-value pair for XGBoost configuration (matches C struct)
#[repr(C)]
struct KVPair {
    key: *const c_char,
    value: *const c_char,
}

extern "C" {
    fn xgbw_init() -> c_int;
    fn xgbw_cleanup();
    fn xgbw_get_last_error() -> *const c_char;
    
    fn xgbw_train_eval(
        x: *const c_float,
        y: *const c_float,
        rows: c_int,
        x_cols: c_int,
        y_cols: c_int,
        train_ratio: c_float,
        config: *const KVPair,
        len_config: c_int,
        output_dir: *const c_char,
        model_name: *const c_char,
        actual_path_out: *mut c_char,
        actual_path_size: c_ulong,
        rmse_out: *mut c_float,
    ) -> c_int;
}

fn get_last_error() -> String {
    unsafe {
        let ptr = xgbw_get_last_error();
        if ptr.is_null() {
            "Unknown error".into()
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }
}

/// Initialize the xgbwrapper library.
pub fn init() -> Result<(), String> {
    unsafe {
        if xgbw_init() == 0 {
            Ok(())
        } else {
            Err(get_last_error())
        }
    }
}

/// Clean up library resources.
pub fn cleanup() {
    unsafe { xgbw_cleanup() }
}

/// Result of training with evaluation
pub struct TrainResult {
    pub model_path: String,
    pub rmse: Vec<f32>,
}

/// Train model with auto split and evaluation.
/// 
/// Calls xgbw_train_eval which handles: split → train → predict → RMSE → save.
#[allow(clippy::too_many_arguments)] // Mirrors xgbwrapper C API
pub fn train_eval(
    x: &[f32],
    y: &[f32],
    rows: usize,
    x_cols: usize,
    y_cols: usize,
    train_ratio: f32,
    params: &[(&str, String)],
    output_dir: &str,
    model_name: &str,
) -> Result<TrainResult, String> {
    assert_eq!(x.len(), rows * x_cols, "x size mismatch");
    assert_eq!(y.len(), rows * y_cols, "y size mismatch");
    
    // Build KVPair array with CStrings kept alive
    let keys: Vec<CString> = params.iter()
        .map(|(k, _)| CString::new(*k).unwrap())
        .collect();
    let values: Vec<CString> = params.iter()
        .map(|(_, v)| CString::new(v.as_str()).unwrap())
        .collect();
    let kv_pairs: Vec<KVPair> = keys.iter()
        .zip(values.iter())
        .map(|(k, v)| KVPair { key: k.as_ptr(), value: v.as_ptr() })
        .collect();
    
    let output_dir_c = CString::new(output_dir).unwrap();
    let model_name_c = CString::new(model_name).unwrap();
    
    let mut path_buf = vec![0u8; 512];
    let mut rmse = vec![0.0f32; y_cols];
    
    unsafe {
        let status = xgbw_train_eval(
            x.as_ptr(),
            y.as_ptr(),
            rows as c_int,
            x_cols as c_int,
            y_cols as c_int,
            train_ratio,
            kv_pairs.as_ptr(),
            kv_pairs.len() as c_int,
            output_dir_c.as_ptr(),
            model_name_c.as_ptr(),
            path_buf.as_mut_ptr() as *mut c_char,
            path_buf.len() as c_ulong,
            rmse.as_mut_ptr(),
        );
        
        if status == 0 {
            let null_pos = path_buf.iter().position(|&b| b == 0).unwrap_or(path_buf.len());
            Ok(TrainResult {
                model_path: String::from_utf8_lossy(&path_buf[..null_pos]).into_owned(),
                rmse,
            })
        } else {
            Err(get_last_error())
        }
    }
}
