//! Configuration module for models_gen
//!
//! Handles YAML configuration parsing for training hyperparameters.

use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Root configuration structure
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub training: TrainingConfig,
    pub xgboost: XGBoostConfig,
    pub output: OutputConfig,
}

/// Training pipeline configuration
#[derive(Debug, Deserialize, Clone)]
pub struct TrainingConfig {
    /// Sample sizes to train models for
    pub sample_sizes: Vec<usize>,
    
    /// Distribution types to generate models for
    pub distributions: Vec<String>,
    
    /// Train/test split ratio (fraction for training set, e.g. 0.7 = 70% train)
    pub train_ratio: f64,
    
    /// Number of examples per parameter pair
    pub dist_train_size: usize,
    
    /// Parameter grid resolution [p1_resolution, p2_resolution]
    pub params_resolution: [usize; 2],
    
    /// Population size for hypergeometric confidence intervals
    pub population_size: usize,
}

/// XGBoost hyperparameters
#[derive(Debug, Deserialize, Clone)]
pub struct XGBoostConfig {
    pub booster: String,
    pub objective: String,
    pub eval_metric: String,
    pub max_depth: String,
    pub gamma: String,
    pub eta: String,
    pub num_round: String,
    pub subsample: String,
    pub colsample_bytree: String,
    pub reg_alpha: String,
    pub reg_lambda: String,
    pub nthread: String,
    pub seed: String,
}

impl XGBoostConfig {
    /// Convert to key-value pairs for FFI
    pub fn to_kv_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("booster", self.booster.clone()),
            ("objective", self.objective.clone()),
            ("eval_metric", self.eval_metric.clone()),
            ("max_depth", self.max_depth.clone()),
            ("gamma", self.gamma.clone()),
            ("eta", self.eta.clone()),
            ("n_estimators", self.num_round.clone()),  // XGBoost C API uses n_estimators
            ("subsample", self.subsample.clone()),
            ("colsample_bytree", self.colsample_bytree.clone()),
            ("reg_alpha", self.reg_alpha.clone()),
            ("reg_lambda", self.reg_lambda.clone()),
            ("nthread", self.nthread.clone()),
            ("seed", self.seed.clone()),
        ]
    }
}

/// Output configuration
#[derive(Debug, Deserialize, Clone)]
pub struct OutputConfig {
    /// Output directory for trained models
    pub models_dir: String,
    
    /// Metrics CSV file path
    pub metrics_file: String,
}

impl Config {
    /// Load configuration from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let mut file = File::open(path.as_ref())
            .map_err(|e| ConfigError::FileOpen(e.to_string()))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| ConfigError::FileRead(e.to_string()))?;
        
        serde_yaml::from_str(&contents)
            .map_err(|e| ConfigError::Parse(e.to_string()))
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.training.sample_sizes.is_empty() {
            return Err(ConfigError::Validation("sample_sizes cannot be empty".into()));
        }
        
        if self.training.distributions.is_empty() {
            return Err(ConfigError::Validation("distributions cannot be empty".into()));
        }
        
        if self.training.train_ratio <= 0.0 || self.training.train_ratio >= 1.0 {
            return Err(ConfigError::Validation("train_ratio must be between 0 and 1".into()));
        }
        
        if self.training.dist_train_size == 0 {
            return Err(ConfigError::Validation("dist_train_size must be > 0".into()));
        }
        
        for dist in &self.training.distributions {
            if dist != "Beta" && dist != "Normal" {
                return Err(ConfigError::Validation(
                    format!("Unknown distribution type: {}", dist)
                ));
            }
        }
        
        Ok(())
    }
}

/// Configuration error types
#[derive(Debug)]
pub enum ConfigError {
    FileOpen(String),
    FileRead(String),
    Parse(String),
    Validation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileOpen(e) => write!(f, "Failed to open config file: {}", e),
            ConfigError::FileRead(e) => write!(f, "Failed to read config file: {}", e),
            ConfigError::Parse(e) => write!(f, "Failed to parse config: {}", e),
            ConfigError::Validation(e) => write!(f, "Config validation error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}
