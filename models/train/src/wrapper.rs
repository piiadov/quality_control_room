use std::ffi::{c_float, c_int};

extern "C" {
    fn shuffle(array: *const c_int, length: c_int);
    fn split_data(x: *const c_float, y: *const c_float,
                  x_train: *const c_float, y_train: *const c_float,
                  x_test: *const c_float, y_test: *const c_float,
                  x_cols: c_int, y_cols: c_int, rows: c_int, rows_train: c_int);
    fn calculate_rmse(pred: *const c_float, test: *const c_float,
                      rows: c_int, cols: c_int, rmse: *const c_float);

    fn xgb_train(x: *const c_float, y: *const c_float, rows: c_int,
                 x_cols: c_int, y_cols: c_int);
    fn xgb_predict(data: *const c_float, rows: c_int, cols: c_int, pred: *const c_float);
    fn print_rmse(rmse: *const c_float, cols: c_int);
}

pub fn c_shuffle(array: &Vec<i32>) {
    unsafe {
        shuffle(array.as_ptr() as *const c_int, array.len() as c_int);
    }
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

    (x_train, y_train, x_test, y_test)
}
