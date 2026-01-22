//! XGBoost FFI bindings for server
//!
//! Minimal FFI wrapper for xgbw_predict - inference only.

use std::ffi::{c_char, c_float, c_int, CStr, CString};

extern "C" {
    fn xgbw_init() -> c_int;
    fn xgbw_cleanup();
    fn xgbw_get_last_error() -> *const c_char;

    fn xgbw_predict(
        data: *const c_float,
        rows: c_int,
        x_cols: c_int,
        y_cols: c_int,
        inference_path: *const c_char,
        pred: *mut c_float,
    ) -> c_int;
}

/// Number of feature columns (fitted params: min_p1, min_p2, max_p1, max_p2)
pub const X_COLS: usize = 4;

/// Number of target columns (distribution params: p1, p2)
pub const Y_COLS: usize = 2;

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

/// Initialize the xgbwrapper library
pub fn init() -> Result<(), String> {
    unsafe {
        if xgbw_init() == 0 {
            Ok(())
        } else {
            Err(get_last_error())
        }
    }
}

/// Clean up library resources
pub fn cleanup() {
    unsafe { xgbw_cleanup() }
}

/// Predict distribution parameters from fitted CDF features
///
/// # Arguments
/// * `features` - [min_p1, min_p2, max_p1, max_p2] from CDF fitting
/// * `model_path` - Path to trained XGBoost model (.ubj or .json)
///
/// # Returns
/// * `Ok([p1, p2])` - Predicted distribution parameters
/// * `Err(msg)` - Error message on failure
pub fn predict(features: [f32; X_COLS], model_path: &str) -> Result<[f32; Y_COLS], String> {
    let model_path_c = CString::new(model_path)
        .map_err(|_| "Invalid model path")?;

    let mut pred = [0.0f32; Y_COLS];

    unsafe {
        let status = xgbw_predict(
            features.as_ptr(),
            1, // single row
            X_COLS as c_int,
            Y_COLS as c_int,
            model_path_c.as_ptr(),
            pred.as_mut_ptr(),
        );

        if status == 0 {
            Ok(pred)
        } else {
            Err(get_last_error())
        }
    }
}
