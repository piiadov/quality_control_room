use libmodels::train::*;
use libmodels::wrapper::*;
use std::time::Instant;
use std::fs::{OpenOptions, metadata};
use csv::WriterBuilder;

const DATA_PATH: &str = "/home/vp/quality_control_room/data";

fn main() {
    let kind = DistributionType::Beta;
    println!("Distribution: {}", kind);

    let config = read_config(format!("{}/config.json", DATA_PATH));
    let main_params = config.main_params;
    let inferences_folder = format!("{}/inferences", DATA_PATH);

    let params_res = [100, 100];
    let dist_train_size: usize = 100; // Number of examples for each pair (a,b)

    let rows = params_res[0] * params_res[1] * dist_train_size;
    println!("Data size: {} rows", rows);

    let (y, dist) = target_prepare(
        kind.clone(), params_res, dist_train_size);

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
        wtr.write_record(&[
            "Sampling size", "Type", "Res1", "Res2", "Data size", "Dist train size", "Elapsed time (min)",
            "RMSE1", "RMSE2", "Sampling RMSE1", "Sampling RMSE2",
            "MAE1", "MAE2", "Sampling MAE1", "Sampling MAE2",
            "RMSE1 Ratio", "RMSE2 Ratio", "MAE1 Ratio", "MAE2 Ratio",
            ])
            .expect("Failed to write headers to metrics.csv");
        wtr.flush().expect("Failed to flush csv writer");
    }

    //for sample_size in [5, 10, 20, 30, 40, 50] {
    for sample_size in [100] {
        let population_size: usize = sample_size * 100; // Large number in comparison with sampling
        println!("Sample size: {}", sample_size);
        let start = Instant::now();

        let (cdf_min, cdf_max) = conf_int(population_size, sample_size);

        let (x, sampling_y) = features_prepare_nm(sample_size, cdf_min, cdf_max,
                                    dist.clone(), kind.clone());

        // Split data for models/test
        let (train_indices, test_indices) = split_data(0.70, rows);
        let x_train: Vec<[f64;4]> = train_indices.iter().map(|&i| x[i]).collect();
        let y_train: Vec<[f64;2]> = train_indices.iter().map(|&i| y[i]).collect();
        let x_test: Vec<[f64;4]> = test_indices.iter().map(|&i| x[i]).collect();
        let y_test: Vec<[f64;2]> = test_indices.iter().map(|&i| y[i]).collect();
        let sampling_y_pred: Vec<[f64;2]> = test_indices.iter().map(|&i| sampling_y[i]).collect();

        // Flat vectors: XGBoost works with f32 (c_float)
        let x_train_flat = flat_vector::<4>(x_train.clone());
        let y_train_flat = flat_vector::<2>(y_train.clone());

        // Train model
        xgb_train(x_train_flat, y_train_flat, x_train.len(), 4, 2, main_params.clone(),
        inferences_folder.clone(), format!("xgb_{}_{}.json", &kind, &sample_size));

        // Test model
        let x_test_flat = flat_vector::<4>(x_test.clone());
        let y_pred_flat = xgb_predict(x_test_flat, x_test.len(), 4, 2,
                                 format!("{}/xgb_{}_{}.json", inferences_folder, &kind, &sample_size));
        let y_pred = shape_vector::<2>(y_pred_flat);


        // Calculate metrics
        let rmse = calculate_rmse(&y_test, &y_pred);
        let sampling_rmse = calculate_rmse(&y_test, &sampling_y_pred);
        let mae = calculate_mae(&y_test, &y_pred);
        let sampling_mae = calculate_mae(&y_test, &sampling_y_pred);

        wtr.write_record(&[
            sample_size.to_string(),
            kind.to_string(),
            params_res[0].to_string(),
            params_res[1].to_string(),
            dist_train_size.to_string(),
            rows.to_string(),
            format!("{:.2}", start.elapsed().as_secs_f64() / 60.0),

            format!("{:.6}", rmse[0]),
            format!("{:.6}", rmse[1]),
            format!("{:.6}", sampling_rmse[0]),
            format!("{:.6}", sampling_rmse[1]),

            format!("{:.6}", mae[0]),
            format!("{:.6}", mae[1]),
            format!("{:.6}", sampling_mae[0]),
            format!("{:.6}", sampling_mae[1]),

            format!("{:.6}", rmse[0] / sampling_rmse[0]),
            format!("{:.6}", rmse[1] / sampling_rmse[1]),
            format!("{:.6}", mae[0] / sampling_mae[0]),
            format!("{:.6}", mae[1] / sampling_mae[1]),

        ])
        .expect("Failed to write values to metrics.csv");
        wtr.flush().expect("Failed to flush csv writer");

        println!("Elapsed time: {:.3} s", start.elapsed().as_secs_f64());
    }
}
