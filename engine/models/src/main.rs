use libmodels::train::*;
use libmodels::wrapper::*;
use std::time::Instant;
use std::fs::{OpenOptions, metadata};
use csv::WriterBuilder;

const CONFIG_PATH: &str = "/home/vp/GitHub/quality_control_room/data/config.json";

fn main() {
    let config = read_config(CONFIG_PATH.to_string());
    let folder_path = config.paths.data_folder;
    let main_params = config.main_params;

    let alpha_res: usize = 100;
    let beta_res: usize = 100;
    let dist_train_size: usize = 100; // Number of examples for each pair (a,b)
    let rows = alpha_res * beta_res * dist_train_size;
    println!("Data size: {} rows", rows);

    let population_size: usize = 3000; // Large number in comparison with samples number
    let alpha_bounds = [0.1, 10.0];
    let beta_bounds = [0.1, 10.0];
    let init_params = [0.1, 0.1];

    let (y, dist) = target_prepare(alpha_bounds, alpha_res,
                                   beta_bounds, beta_res, dist_train_size);
    let y = flat_vector::<2>(y); // XGBoost works with f32 (c_float)

    let file_path = format!("{}/rmse.csv", folder_path);
    let file_exists = metadata(&file_path).is_ok();

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .expect("Failed to open or create rmse.csv");
    let mut wtr = WriterBuilder::new()
        .has_headers(false) // Avoid writing headers again
        .from_writer(file);
    if !file_exists {
        wtr.write_record(&["sample_num", "a_rmse", "b_rmse"])
            .expect("Failed to write headers to rmse.csv");
        wtr.flush().expect("Failed to flush csv writer");
    }

    for sample_size in [100] {
        println!("Sample size: {}", sample_size);
        let start = Instant::now();

        let (cdf_min, cdf_max) = conf_int(population_size, sample_size);

        let x = features_prepare_nm(sample_size, cdf_min, cdf_max,
                                    dist.clone(), alpha_bounds, beta_bounds, init_params);
        let x = flat_vector::<4>(x);

        // Split data for models/test
        let split_ratio = 0.70;
        let (x_train, y_train, x_test, y_test)
            = c_split_data(x, y.clone(), rows, 4, 2, split_ratio);
        assert_eq!(((x_train.len() + x_test.len()) as f64/ 4.0) as usize, rows,
                   "x_train.len() + x_test.len() != rows");

        // Train model
        let rows_train = (split_ratio * rows as f32) as usize;
        assert_eq!(rows_train, (x_train.len() as f64 / 4.0) as usize, "Wrong rows_train value");

        xgb_train(x_train, y_train, rows_train, 4, 2, main_params.clone(),
                  folder_path.clone(), format!("xgb_{}.json", &sample_size));

        // Test model
        let rows_test = rows - rows_train;
        let y_pred = xgb_predict(x_test, rows_test, 4, 2,
                                 format!("{}/xgb_{}.json", folder_path, &sample_size));
        let rmse = c_calculate_rmse(y_test, y_pred, rows_test, 2);
        c_print_rmse(&rmse);

        wtr.write_record(&[
            sample_size.to_string(),
            rmse[0].to_string(),
            rmse[1].to_string(),
        ])
        .expect("Failed to write RMSE values to rmse.csv");
        wtr.flush().expect("Failed to flush csv writer");

        println!("Elapsed time: {:.3} s", start.elapsed().as_secs_f64());
    }
}
