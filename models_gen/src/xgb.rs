//! XGBoost FFI bindings module
//!
//! Provides safe Rust wrappers around the xgbwrapper C library functions.

use std::ffi::{c_char, c_float, c_int, CString};
use std::os::raw::c_ulong;

/// Key-value pair for XGBoost configuration (matches C struct)
#[repr(C)]
pub struct KVPair {
    pub key: *const c_char,
    pub value: *const c_char,
}

/// XGBoost wrapper status codes (matches C enum)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XGBWrapperStatus {
    Success = 0,
    ErrorInvalidParam = 1,
    ErrorMemory = 2,
    ErrorFileIo = 3,
    ErrorXgboost = 4,
    ErrorNotInitialized = 5,
    ErrorSizeMismatch = 6,
}

impl XGBWrapperStatus {
    pub fn is_success(&self) -> bool {
        *self == XGBWrapperStatus::Success
    }
}

extern "C" {
    fn xgbw_init() -> XGBWrapperStatus;
    fn xgbw_cleanup();
    fn xgbw_get_last_error() -> *const c_char;
    
    fn xgbw_train_eval(
        x: *const c_float, y: *const c_float,
        rows: c_int, x_cols: c_int, y_cols: c_int,
        train_ratio: c_float,
        config: *const KVPair, len_config: c_int,
        output_dir: *const c_char,
        model_name: *const c_char,
        actual_path_out: *mut c_char,
        actual_path_size: c_ulong,
        rmse_out: *mut c_float
    ) -> XGBWrapperStatus;
}

/// Initialize the xgbwrapper library. Must be called before any other functions.
pub fn init() -> Result<(), String> {
    unsafe {
        let status = xgbw_init();
        if status.is_success() {
            Ok(())
        } else {
            Err(get_last_error())
        }
    }
}

/// Clean up library resources. Call when done using the library.
pub fn cleanup() {
    unsafe {
        xgbw_cleanup();
    }
}

/// Get the last error message from the library.
pub fn get_last_error() -> String {
    unsafe {
        let ptr = xgbw_get_last_error();
        if ptr.is_null() {
            String::from("Unknown error")
        } else {
            std::ffi::CStr::from_ptr(ptr)
                .to_string_lossy()
                .into_owned()
        }
    }
}

/// Holds CStrings to keep them alive during FFI call
struct KVPairHolder {
    _keys: Vec<CString>,
    _values: Vec<CString>,
    pairs: Vec<KVPair>,
}

impl KVPairHolder {
    fn new(params: &[(&str, String)]) -> Self {
        let keys: Vec<CString> = params.iter()
            .map(|(k, _)| CString::new(*k).unwrap())
            .collect();
        let values: Vec<CString> = params.iter()
            .map(|(_, v)| CString::new(v.as_str()).unwrap())
            .collect();
        
        let pairs: Vec<KVPair> = keys.iter().zip(values.iter())
            .map(|(k, v)| KVPair {
                key: k.as_ptr(),
                value: v.as_ptr(),
            })
            .collect();
        
        KVPairHolder {
            _keys: keys,
            _values: values,
            pairs,
        }
    }
    
    fn as_ptr(&self) -> *const KVPair {
        self.pairs.as_ptr()
    }
    
    fn len(&self) -> c_int {
        self.pairs.len() as c_int
    }
}

/// Result of training with evaluation
pub struct TrainResult {
    pub model_path: String,
    pub rmse: Vec<f32>,
}

/// Train an XGBoost model with train/test split and evaluation.
/// Returns the model path and RMSE metrics.
pub fn train_eval(
    x: &[f32], y: &[f32],
    rows: usize, x_cols: usize, y_cols: usize,
    train_ratio: f32,
    params: &[(&str, String)],
    output_dir: &str,
    model_name: &str
) -> Result<TrainResult, String> {
    assert_eq!(x.len(), rows * x_cols, "x.len() != rows * x_cols");
    assert_eq!(y.len(), rows * y_cols, "y.len() != rows * y_cols");
    
    let kv_holder = KVPairHolder::new(params);
    let output_dir_c = CString::new(output_dir).unwrap();
    let model_name_c = CString::new(model_name).unwrap();
    
    let mut actual_path = vec![0u8; 512];
    let mut rmse = vec![0.0f32; y_cols];
    
    unsafe {
        let status = xgbw_train_eval(
            x.as_ptr(),
            y.as_ptr(),
            rows as c_int,
            x_cols as c_int,
            y_cols as c_int,
            train_ratio,
            kv_holder.as_ptr(),
            kv_holder.len(),
            output_dir_c.as_ptr(),
            model_name_c.as_ptr(),
            actual_path.as_mut_ptr() as *mut c_char,
            actual_path.len() as c_ulong,
            rmse.as_mut_ptr()
        );
        
        if status.is_success() {
            let null_pos = actual_path.iter().position(|&b| b == 0).unwrap_or(actual_path.len());
            let path_str = String::from_utf8_lossy(&actual_path[..null_pos]).into_owned();
            Ok(TrainResult {
                model_path: path_str,
                rmse,
            })
        } else {
            Err(get_last_error())
        }
    }
}
