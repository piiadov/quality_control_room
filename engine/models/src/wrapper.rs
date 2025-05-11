use std::ffi::{c_char, c_float, c_int, CString};
use std::fs::File;
use std::io::Read;
use serde::Deserialize;

extern "C" {
    fn generate_data_2cols(x: *const c_float, y: *const c_float, rows: c_int, x_cols: c_int);
    fn shuffle(array: *const c_int, length: c_int);
    fn split_data(x: *const c_float, y: *const c_float,
                  x_train: *const c_float, y_train: *const c_float,
                  x_test: *const c_float, y_test: *const c_float,
                  x_cols: c_int, y_cols: c_int, rows: c_int, rows_train: c_int);
    fn train(x: *const c_float, y: *const c_float, rows: c_int,
             x_cols: c_int, y_cols: c_int,
             config: *const KVPair, len_config: c_int, inference_path: *const c_char);
    fn predict(x: *const c_float, rows: c_int, x_cols: c_int, y_cols: c_int,
               inference_path: *const c_char, pred: *const c_float);
    fn calculate_rmse(pred: *const c_float, test: *const c_float,
                      rows: c_int, cols: c_int, rmse: *const c_float);
    fn print_rmse(rmse: *const c_float, len: c_int);
}

#[repr(C)]
struct KVPair {
    key: *const c_char,
    value: *const c_char,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub main_params: XGBParams,
    pub test_params: XGBParams,
    pub paths: Paths,
}

#[derive(Debug, Deserialize, Clone)]
pub struct XGBParams {
    pub booster: String,
    pub objective: String,
    pub eval_metric: String,
    pub n_thread: String,
    pub subsample: String,
    pub reg_alpha: String,
    pub reg_lambda: String,
    pub max_depth: String,
    pub gamma: String,
    pub learning_rate: String,
    pub colsample_bytree: String,
    pub eta: String,
    pub n_estimators: String,
    pub random_state: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct Paths {
    pub data_folder: String,
}

pub fn read_config(path: String) -> Config {
    let mut file = File::open(path).expect("Unable to open config file");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read config file");
    serde_json::from_str(&data).expect("Unable to parse JSON from config file")
}

pub fn shape_vector<const N: usize>(vec: Vec<f32>) -> Vec<[f64;N]>
where [f64;N]: Sized {
    assert!(N == 2 || N == 4);
    vec
        .into_iter()
        .map(|x| x as f64)
        .collect::<Vec<f64>>()
        .chunks_exact(N)
        .map(|chunk| {
            let mut arr = [0.0; N];
            for (i, &val) in chunk.iter().enumerate() {
                arr[i] = val;
            }
            arr
        })
        .collect()
}

pub fn flat_vector<const N: usize>(vec: Vec<[f64;N]>) -> Vec<f32>
where
    [f64; N]: Sized {
    assert!(N == 2 || N == 4);
    vec.into_iter().flatten().map(|x| x as f32).collect()
}

pub fn c_shuffle(array: &Vec<i32>) {
    unsafe {
        shuffle(array.as_ptr() as *const c_int, array.len() as c_int);
    }
}

pub fn c_split_data(x: Vec<f32>, y: Vec<f32>, rows: usize,
                    x_cols: usize, y_cols: usize, ratio: f32)
    -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {

    assert_eq!(x.len(), rows * x_cols, "x.len() != rows * x_cols");
    assert_eq!(y.len(), rows * y_cols, "y.len() != rows * y_cols");
    assert!(ratio >= 0.0 && ratio <= 1.0, "ratio must be between 0 and 1");

    let rows_train = (ratio * rows as f32) as usize;
    let rows_test = rows- rows_train;

    let mut x_train: Vec<f32> = Vec::with_capacity(rows_train * x_cols);
    x_train.resize(rows_train * x_cols, 0.0);
    let mut y_train: Vec<f32> = Vec::with_capacity(rows_train * y_cols);
    y_train.resize(rows_train * y_cols, 0.0);

    let mut x_test: Vec<f32> = Vec::with_capacity(rows_test * x_cols);
    x_test.resize(rows_test * x_cols, 0.0);
    let mut y_test: Vec<f32> = Vec::with_capacity(rows_test * y_cols);
    y_test.resize(rows_test * y_cols, 0.0);

    unsafe {
        split_data(x.as_ptr() as *const c_float, y.as_ptr() as *const c_float,
                   x_train.as_ptr() as *const c_float, y_train.as_ptr() as *const c_float,
                   x_test.as_ptr() as *const c_float, y_test.as_ptr() as *const c_float,
                   x_cols as c_int, y_cols as c_int, rows as c_int, rows_train as c_int);
    }

    (x_train, y_train, x_test, y_test) // Add y_sampling
}

pub fn c_generate_data_2cols(rows: usize, x_cols: usize) -> (Vec<f32>, Vec<f32>) {
    let y_cols: usize = 2;
    let mut x: Vec<f32> = Vec::with_capacity(x_cols * rows);
    x.resize(x_cols * rows, 0.0);
    let mut y: Vec<f32> = Vec::with_capacity(y_cols * rows);
    y.resize(y_cols * rows, 0.0);
    unsafe {
        generate_data_2cols(x.as_ptr() as *const c_float, y.as_ptr() as *const c_float,
                            rows as c_int, x_cols as c_int);
    }
    (x, y)
}

pub fn xgb_train(x: Vec<f32>, y: Vec<f32>, rows: usize,
                 x_cols: usize, y_cols: usize,
                 xgb_params: XGBParams, folder_path: String, filename: String) {
    let filename = CString::new(format!("{}/{}", folder_path, filename)).unwrap();
    let params = vec![
        ("booster", xgb_params.booster),
        ("objective", xgb_params.objective),
        ("eval_metric", xgb_params.eval_metric),
        ("n_thread", xgb_params.n_thread),
        ("subsample", xgb_params.subsample),
        ("reg_alpha", xgb_params.reg_alpha),
        ("reg_lambda", xgb_params.reg_lambda),
        ("max_depth", xgb_params.max_depth),
        ("gamma", xgb_params.gamma),
        ("learning_rate", xgb_params.learning_rate),
        ("colsample_bytree", xgb_params.colsample_bytree),
        ("eta", xgb_params.eta),
        ("n_estimators", xgb_params.n_estimators),
        ("random_state", xgb_params.random_state),
    ];

    let kv_pairs: Vec<KVPair> = params.into_iter().map(|(key, value)| {
        KVPair {
            key: CString::new(key).unwrap().into_raw(),
            value: CString::new(value).unwrap().into_raw(),
        }
    }).collect();
    let len_config = kv_pairs.len() as c_int;
    unsafe {
        train(x.as_ptr() as *const c_float, y.as_ptr() as *const c_float,
              rows as c_int, x_cols as c_int, y_cols as c_int,
              kv_pairs.as_ptr(), len_config, filename.as_ptr());
    }
}

pub fn xgb_predict(x: Vec<f32>, rows: usize, x_cols: usize, y_cols: usize,
                   inference_path: String) -> Vec<f32> {
    assert_eq!(x.len(), rows * x_cols, "x.len() != rows * x_cols");
    let inference_path = CString::new(inference_path).unwrap();
    let mut pred: Vec<f32> = Vec::with_capacity(y_cols * rows);
    pred.resize(y_cols * rows, 0.0);
    unsafe {
        predict(x.as_ptr() as *const c_float, rows as c_int, x_cols as c_int, y_cols as c_int,
                inference_path.as_ptr(), pred.as_ptr())
    }
    pred
}

pub fn c_calculate_rmse(y_test: Vec<f32>, y_pred: Vec<f32>, rows: usize, cols: usize) -> Vec<f32> {
    assert_eq!(y_test.len(), rows * cols, "y_test.len() != rows * cols");
    assert_eq!(y_pred.len(), y_test.len(), "y_pred.len() != y_test.len()");
    let mut rmse: Vec<f32> = Vec::with_capacity(cols);
    rmse.resize(cols, 0.0);
    unsafe {
        calculate_rmse(y_pred.as_ptr() as *const c_float, y_test.as_ptr() as *const c_float,
                       rows as c_int, cols as c_int, rmse.as_ptr());
    }
    rmse
}

pub fn c_print_rmse(rmse: &Vec<f32>) {
    unsafe {
        print_rmse(rmse.as_ptr(), rmse.len() as c_int);
    }
}
