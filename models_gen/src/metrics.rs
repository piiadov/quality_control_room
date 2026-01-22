//! Metrics module for models_gen
//!
//! Handles metrics calculation and CSV logging.

use csv::WriterBuilder;
use std::fs::{metadata, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Metrics record for a single training run
#[derive(Debug, Clone)]
pub struct TrainingMetrics {
    pub sample_size: usize,
    pub distribution_type: String,
    pub params_res: [usize; 2],
    pub dist_train_size: usize,
    pub data_size: usize,
    pub elapsed_time_min: f64,
    pub rmse: [f64; 2],
    pub model_path: String,
}

/// CSV metrics writer
pub struct MetricsWriter {
    path: String,
}

impl MetricsWriter {
    /// Create a new metrics writer
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        MetricsWriter {
            path: path.as_ref().to_string_lossy().into_owned(),
        }
    }
    
    /// Write metrics header if file doesn't exist
    pub fn ensure_header(&self) -> std::io::Result<()> {
        if metadata(&self.path).is_err() {
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&self.path)?;
            
            writeln!(
                file,
                "Sample Size,Distribution,Res1,Res2,Dist Train Size,Data Size,Elapsed (min),RMSE1,RMSE2,Model Path"
            )?;
        }
        Ok(())
    }
    
    /// Append a metrics record to the CSV file
    pub fn write_record(&self, metrics: &TrainingMetrics) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        
        let mut wtr = WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);
        
        wtr.write_record(&[
            metrics.sample_size.to_string(),
            metrics.distribution_type.clone(),
            metrics.params_res[0].to_string(),
            metrics.params_res[1].to_string(),
            metrics.dist_train_size.to_string(),
            metrics.data_size.to_string(),
            format!("{:.2}", metrics.elapsed_time_min),
            format!("{:.6}", metrics.rmse[0]),
            format!("{:.6}", metrics.rmse[1]),
            metrics.model_path.clone(),
        ])?;
        
        wtr.flush()?;
        Ok(())
    }
}

/// Print metrics summary to console
pub fn print_metrics_summary(metrics: &TrainingMetrics) {
    println!("├─ Data size: {} rows", metrics.data_size);
    println!("├─ Elapsed: {:.2} min", metrics.elapsed_time_min);
    println!("├─ RMSE: [{:.6}, {:.6}]", metrics.rmse[0], metrics.rmse[1]);
    println!("└─ Model: {}", metrics.model_path);
}
