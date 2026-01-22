//! Configuration module for server
//!
//! Handles YAML configuration parsing.

use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Root configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub models: ModelsConfig,
    pub statistics: StatisticsConfig,
}

/// Server network configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub ws_path: String,
    pub tls: Option<TlsConfig>,
}

/// TLS configuration
#[derive(Debug, Deserialize, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

/// Models configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ModelsConfig {
    pub models_dir: String,
    pub sample_sizes: Vec<usize>,
}

/// Statistics configuration
#[derive(Debug, Deserialize, Clone)]
pub struct StatisticsConfig {
    pub default_population_size: usize,
    pub alpha: f64,
    pub default_bins: usize,
    pub prob_threshold_factor: f64,
}

impl Config {
    /// Load configuration from YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let mut file = File::open(path.as_ref())
            .map_err(|e| ConfigError::FileOpen(e.to_string()))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| ConfigError::FileRead(e.to_string()))?;
        
        serde_yaml::from_str(&contents)
            .map_err(|e| ConfigError::Parse(e.to_string()))
    }
}

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    FileOpen(String),
    FileRead(String),
    Parse(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileOpen(e) => write!(f, "Failed to open config file: {}", e),
            ConfigError::FileRead(e) => write!(f, "Failed to read config file: {}", e),
            ConfigError::Parse(e) => write!(f, "Failed to parse config: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}
