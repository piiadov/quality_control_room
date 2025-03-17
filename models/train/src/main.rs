use lib::train::*;
use lib::wrapper::*;
use std::time::Instant;
use rand_distr::{Beta as RandBeta};

fn main() {
    let start = Instant::now();

    // Parameters
    let (population_size, sample_size): (usize, usize) = (1000, 50); // Batch and sampling sizes
    let alpha_bounds: [f64; 2] = [1.0, 5.0];
    let beta_bounds: [f64; 2] = [1.0, 5.0];
    
    let alpha_res: usize = 2;
    let beta_res: usize = 2;
    let dist_train_size: usize = 3; // Number of examples for each pair (a,b)

    // How much memory we need
    let iter_num = alpha_res * beta_res * dist_train_size;
    println!("Elements num: {}", iter_num);
    let dist_mem = iter_num * size_of::<RandBeta<f64>>() + size_of::<Vec<RandBeta<f64>>>();
    println!("Memory for Beta-distributions: {} MB", dist_mem/1024/1024);
    let y_mem = 2 * iter_num * size_of::<f64>() + size_of::<Vec<f64>>();
    println!("Memory for y: {} MB", y_mem/1024/1024);


    let (cdf_min, cdf_max) = conf_int(population_size, sample_size);
    let (y, dist) = target_prepare(alpha_bounds, alpha_res,
                                   beta_bounds, beta_res, dist_train_size);

    let x = features_prepare (sample_size, cdf_min, cdf_max,
                                          dist, alpha_bounds, beta_bounds);

    // XGBoost part
    let x_flat: Vec<f64> = x.iter().flat_map(|&arr| arr).collect();
    let y_flat: Vec<f64> = y.iter().flat_map(|&arr| arr).collect();

    xgb(x_flat, y_flat);

    println!("Elapsed time: {:.3} s", start.elapsed().as_secs_f64());
}
