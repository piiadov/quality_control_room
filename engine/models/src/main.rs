use libmodels::train::*;
use libmodels::wrapper::*;
use std::time::Instant;
use std::fs::{OpenOptions, metadata};
use csv::WriterBuilder;

const DATA_PATH: &str = "/home/vp/quality_control_room/data";

fn main() {
    let kind = DistributionType::Normal;
    println!("Distribution: {}", kind);

    let config = read_config(format!("{}/config.json", DATA_PATH));
    let main_params = config.main_params;
    let inferences_folder = format!("{}/inferences", DATA_PATH);

    let params_res = [200, 200];
    let dist_train_size: usize = 200; // Number of examples for each pair (a,b)

    let rows = params_res[0] * params_res[1] * dist_train_size;
    println!("Data size: {} rows", rows);

    let (y, dist) = target_prepare(
        kind.clone(), params_res, dist_train_size);
    let y = flat_vector::<2>(y); // XGBoost works with f32 (c_float)

    let metrics_path = format!("{}/metrics.csv", DATA_PATH);
    let metrics_file_exists = metadata(&metrics_path).is_ok();

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&metrics_path)
        .expect("Failed to open or create metrics.csv");
    let mut wtr = WriterBuilder::new()
        .has_headers(false) // Avoid writing headers again
        .from_writer(file);
    if !metrics_file_exists {
        wtr.write_record(&["Sampling size", "Type", "RMSE1", "RMSE2", "MSE1", "MSE2", "Dist train size",
            "Param1 Res", "Param2 Res", "Data size", "Elapsed time", "Sampling RMSE1", "Sampling RMSE2", "Sampling MSE1", "Sampling MSE2"])
            .expect("Failed to write headers to metrics.csv");
        wtr.flush().expect("Failed to flush csv writer");
    }

    for sample_size in [50] {
        let population_size: usize = sample_size * 100; // Large number in comparison with sampling
        println!("Sample size: {}", sample_size);
        let start = Instant::now();

        let (cdf_min, cdf_max) = conf_int(population_size, sample_size);

        let (x, sampling_y) = features_prepare_nm(sample_size, cdf_min, cdf_max,
                                    dist.clone(), kind.clone());
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
        inferences_folder.clone(), format!("xgb_{}_{}.json", &kind, &sample_size));

        // Test model
        let rows_test = rows - rows_train;
        let y_pred = xgb_predict(x_test, rows_test, 4, 2,
                                 format!("{}/xgb_{}_{}.json", inferences_folder, &kind, &sample_size));
        let rmse = c_calculate_rmse(y_test, y_pred, rows_test, 2);
        c_print_rmse(&rmse);

        wtr.write_record(&[
            sample_size.to_string(),
            kind.to_string(),
            format!("{:.6}", rmse[0]),
            format!("{:.6}", rmse[1]),
            format!("{:.6}", rmse[0] * rmse[0]),
            format!("{:.6}", rmse[1] * rmse[1]),
            dist_train_size.to_string(),
            params_res[0].to_string(),
            params_res[1].to_string(),
            rows.to_string(),
            format!("{:.2}", start.elapsed().as_secs_f64() / 60.0),
        ])
        .expect("Failed to write values to metrics.csv");
        wtr.flush().expect("Failed to flush csv writer");

        println!("Elapsed time: {:.3} s", start.elapsed().as_secs_f64());
    }
}
