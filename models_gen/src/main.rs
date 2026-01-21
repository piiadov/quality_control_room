//! XGBoost Model Generator
//!
//! A standalone tool for training XGBoost models to predict distribution
//! parameters from quality control sampling data.
//!
//! # Usage
//!
//! ```bash
//! models_gen [config.yaml]
//! ```
//!
//! If no config file is specified, looks for `config.yaml` in the current directory.

mod config;
mod datagen;
mod metrics;
mod xgb;

use config::Config;
use datagen::{conf_int, features_prepare_nm, flat_vector, target_prepare, DistributionType};
use metrics::{print_metrics_summary, MetricsWriter, TrainingMetrics};
use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Number of feature columns (fitted params: min_p1, min_p2, max_p1, max_p2)
const X_COLS: usize = 4;
/// Number of target columns (distribution params: p1, p2)
const Y_COLS: usize = 2;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 { &args[1] } else { "config.yaml" };

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           XGBoost Model Generator for Quality Control         ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // Load and validate configuration
    println!("Loading configuration from: {}", config_path);
    let config = match Config::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    // Initialize xgbwrapper library
    println!("Initializing XGBoost wrapper...");
    if let Err(e) = xgb::init() {
        eprintln!("Failed to initialize xgbwrapper: {}", e);
        std::process::exit(1);
    }

    // Ensure output directory exists
    let models_dir = &config.output.models_dir;
    if !Path::new(models_dir).exists() {
        println!("Creating output directory: {}", models_dir);
        if let Err(e) = fs::create_dir_all(models_dir) {
            eprintln!("Failed to create output directory: {}", e);
            std::process::exit(1);
        }
    }

    // Setup metrics writer
    let metrics_writer = MetricsWriter::new(&config.output.metrics_file);
    if let Err(e) = metrics_writer.ensure_header() {
        eprintln!("Warning: Failed to write metrics header: {}", e);
    }

    // Get XGBoost parameters
    let xgb_params: Vec<(&str, String)> = config.xgboost.to_kv_pairs();

    // Training parameters
    let total_start = Instant::now();
    let params_res = config.training.params_resolution;
    let dist_train_size = config.training.dist_train_size;
    let train_ratio = config.training.train_ratio as f32;

    for dist_name in &config.training.distributions {
        let kind = match DistributionType::from_str(dist_name) {
            Some(k) => k,
            None => {
                eprintln!("Unknown distribution type: {}", dist_name);
                continue;
            }
        };

        println!();
        println!("══════════════════════════════════════════════════════════════");
        println!("Distribution: {}", kind);
        println!("══════════════════════════════════════════════════════════════");

        // Prepare target data (distribution parameters)
        let rows = params_res[0] * params_res[1] * dist_train_size;
        println!("Preparing target data ({} rows)...", rows);
        let (y, dist) = target_prepare(&kind, params_res, dist_train_size);

        for &sample_size in &config.training.sample_sizes {
            println!();
            println!("┌─ Sample size: {}", sample_size);
            let start = Instant::now();

            let population_size = config.training.population_size;

            // Calculate confidence intervals
            let (cdf_min, cdf_max) = conf_int(population_size, sample_size);

            // Prepare features using CDF curve fitting
            println!("├─ Fitting CDF curves...");
            let x = features_prepare_nm(sample_size, cdf_min, cdf_max, dist.clone(), &kind);

            // Flatten data for FFI
            let x_flat = flat_vector(&x);
            let y_flat = flat_vector(&y);

            // Train model with evaluation (xgbwrapper handles split, train, predict, RMSE)
            let model_name = format!("xgb_{}_{}", kind, sample_size);
            println!("├─ Training model: {} (train ratio: {:.0}%)...", model_name, train_ratio * 100.0);

            let result = match xgb::train_eval(
                &x_flat,
                &y_flat,
                rows,
                X_COLS,
                Y_COLS,
                train_ratio,
                &xgb_params,
                models_dir,
                &model_name,
            ) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("  └─ Error training model: {}", e);
                    continue;
                }
            };

            let elapsed = start.elapsed().as_secs_f64() / 60.0;

            let training_metrics = TrainingMetrics {
                sample_size,
                distribution_type: kind.to_string(),
                params_res,
                dist_train_size,
                data_size: rows,
                elapsed_time_min: elapsed,
                rmse: [result.rmse[0] as f64, result.rmse[1] as f64],
                model_path: result.model_path.clone(),
            };

            print_metrics_summary(&training_metrics);

            if let Err(e) = metrics_writer.write_record(&training_metrics) {
                eprintln!("  Warning: Failed to write metrics: {}", e);
            }
        }
    }

    xgb::cleanup();

    let total_elapsed = total_start.elapsed().as_secs_f64() / 60.0;
    println!();
    println!("══════════════════════════════════════════════════════════════");
    println!("Training complete! Total time: {:.2} minutes", total_elapsed);
    println!("Models saved to: {}", models_dir);
    println!("══════════════════════════════════════════════════════════════");
}
