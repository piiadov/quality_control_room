#[test]
use lib::wrapper::*;

#[test]
const CONFIG_PATH: &str = "/home/vp/GitHub/quality_control_room/data/config.json";
#[test]
fn test_read_config() {
    let config = read_config(CONFIG_PATH.to_string());
    println!("conf: {:?}", config);
}

#[test]
fn test_shuffle() {
    let indices: Vec<i32> = vec![0,0,0,0,0];
    c_shuffle(&indices);
    println!("Shuffle vector: {:?}", indices);
}

#[test]
fn test_split_data() {
    let x: Vec<[f64; 4]> = (0..10*4).step_by(4)
        .map(|i| [i as f64, (i + 1) as f64, (i + 2) as f64, (i + 3) as f64]).collect();
    let y: Vec<[f64; 2]> = (0..10*2).step_by(2)
        .map(|i| [i as f64, (i + 1) as f64]).collect();

    println!("x: {:?}", x);
    println!("y: {:?}", y);

    let x = flat_vector::<4>(x);
    let y = flat_vector::<2>(y);

    println!("flat x: {:?}", x);
    println!("flat y: {:?}", y);

    let (_x_train, _y_train,
        x_test, y_test) = c_split_data(x, y, 10,
                                       4, 2, 0.75);
    println!("flat x_test: {:?}", x_test);
    println!("flat y_test: {:?}", y_test);

    let x_test = shape_vector::<4>(x_test);
    let y_test = shape_vector::<2>(y_test);

    println!("x_test: {:?}", x_test);
    println!("y_test: {:?}", y_test);
}

#[test]
fn test_xgb() {
    let rows: usize = 10000;
    let x_cols: usize = 4;
    let y_cols: usize = 2;
    let (x, y) = c_generate_data_2cols(rows, x_cols);
    let config = read_config(CONFIG_PATH.to_string());

    let split_ratio: f32 = 0.8;
    let (x_train, y_train, x_test, y_test)
        = c_split_data(x, y, rows, x_cols, y_cols, split_ratio);

    let config_clone = config.clone();
    let rows_train = (split_ratio * rows as f32) as usize;
    xgb_train(x_train, y_train, rows_train, x_cols, y_cols, config);

    let rows_test = rows - rows_train;
    let y_pred = xgb_predict(x_test, rows_test, x_cols, y_cols, config_clone);

    let rmse = c_calculate_rmse(y_test, y_pred, rows_test, y_cols);
    c_print_rmse(&rmse);
}

#[test]
fn test_calculate_rmse() {
    let y_true = vec![[1.0, 2.0], [3.0, 4.0]];
    let y_pred = vec![[2.0, 4.0], [4.0, 6.0]];

    let rows = 2;
    let cols = 2;

    let y_true = flat_vector::<2>(y_true);
    let y_pred = flat_vector::<2>(y_pred);

    println!("y_true: {:?}", y_true);
    println!("y_pred: {:?}", y_pred);

    let rmse = c_calculate_rmse(y_true, y_pred, rows, cols);
    println!("RMSE: {:?}", rmse);
    assert!(rmse[0] > 0.0);
}
