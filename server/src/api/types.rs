//! API request and response types

use crate::stats::ChiSquareResult;
use serde::{Deserialize, Serialize};

/// Incoming WebSocket request
#[derive(Debug, Deserialize)]
pub struct ApiRequest {
    /// Command: "about", "analyze", "get_intervals", "get_cdf", "get_pdf", "get_histogram", "generate_test_data"
    pub command: String,

    /// Distribution type: 0 = Beta, 1 = Normal
    #[serde(default)]
    pub distribution: u8,

    // === For "analyze" ===
    /// Raw sample data (for analyze)
    #[serde(default)]
    pub data: Vec<f64>,

    /// Minimum value for scaling
    #[serde(default)]
    pub min_value: Option<f64>,

    /// Maximum value for scaling
    #[serde(default)]
    pub max_value: Option<f64>,

    /// Population size for hypergeometric CI
    #[serde(default)]
    pub population_size: Option<usize>,

    // === For "generate_test_data" ===
    /// Distribution parameters [alpha, beta] or [mean, std]
    #[serde(default)]
    pub params: Option<[f64; 2]>,

    /// Sample size for test data generation
    #[serde(default)]
    pub sample_size: Option<usize>,

    // === For curve/histogram requests (client sends params back) ===
    /// Params from CI lower bound fit
    #[serde(default)]
    pub params_min: Option<[f64; 2]>,

    /// Params from CI upper bound fit
    #[serde(default)]
    pub params_max: Option<[f64; 2]>,

    /// Predicted params from XGBoost
    #[serde(default)]
    pub predicted_params: Option<[f64; 2]>,

    /// Method of moments params
    #[serde(default)]
    pub sampling_params: Option<[f64; 2]>,

    // === For "get_histogram" ===
    /// Number of bins
    #[serde(default)]
    pub bins: Option<usize>,

    /// Scaled data [0,1] (client stores after analyze)
    #[serde(default)]
    pub scaled_data: Option<Vec<f64>>,
}

/// API response - fields populated based on command
#[derive(Debug, Serialize, Default)]
pub struct ApiResponse {
    /// Echo of command name
    pub command: String,

    /// Status: true = success
    pub success: bool,

    /// Error or info message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    // === "about" ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    // === "analyze" core results ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scaled_data: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params_min: Option<[f64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params_max: Option<[f64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predicted_params: Option<[f64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_params: Option<[f64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chi2_min: Option<ChiSquareResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chi2_max: Option<ChiSquareResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chi2_pred: Option<ChiSquareResult>,

    // === "get_intervals" ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdf_min: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdf_max: Option<Vec<f64>>,

    // === "get_cdf" ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitted_cdf_min: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitted_cdf_max: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predicted_cdf: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_cdf: Option<Vec<f64>>,

    // === "get_pdf" ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitted_pdf_min: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitted_pdf_max: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predicted_pdf: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_pdf: Option<Vec<f64>>,

    // === "get_histogram" ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_edges: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_freq: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_freq_min: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_freq_max: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_freq_pred: Option<Vec<f64>>,

    // === "generate_test_data" ===
    /// Generated test samples
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_data: Option<Vec<f64>>,
}
