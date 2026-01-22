//! WebSocket API handlers
//!
//! Request/response types and handler functions for quality analysis.

use crate::config::Config;
use crate::stats::{
    bin_edges, chi_square_test, conf_int, expected_freq, frequencies,
    method_of_moments, pdf, scale_data, survival_cdf, ChiSquareResult, DistributionType,
};
use crate::xgb;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// =============================================================================
// API Request/Response Types
// =============================================================================

/// Incoming WebSocket request
#[derive(Debug, Deserialize)]
pub struct ApiRequest {
    /// Command name: "analyze", "update_bins", "about"
    pub command: String,

    /// Distribution type: 0 = Beta, 1 = Normal
    #[serde(default)]
    pub distribution: u8,

    /// Raw sample data
    #[serde(default)]
    pub data: Vec<f64>,

    /// Minimum value for scaling (domain lower bound)
    #[serde(default)]
    pub min_value: Option<f64>,

    /// Maximum value for scaling (domain upper bound)
    #[serde(default)]
    pub max_value: Option<f64>,

    /// Population size for hypergeometric confidence intervals
    #[serde(default)]
    pub population_size: Option<usize>,

    /// Number of histogram bins
    #[serde(default)]
    pub bins: Option<usize>,

    /// Test mode flag (generates synthetic data)
    #[serde(default)]
    pub test_mode: bool,

    // Fields for update_bins command (when params already computed)
    #[serde(default)]
    pub params_min: Option<[f64; 2]>,
    #[serde(default)]
    pub params_max: Option<[f64; 2]>,
    #[serde(default)]
    pub predicted_params: Option<[f64; 2]>,
}

/// API response
#[derive(Debug, Serialize, Default)]
pub struct ApiResponse {
    /// Echo of command name
    pub command: String,

    /// Status: true = success
    pub success: bool,

    /// Error or info message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Server version (for "about" command)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    // === Input echo ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,

    // === Scaled data ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scaled_data: Option<Vec<f64>>,

    // === Domain for plotting ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Vec<f64>>,

    // === Confidence intervals ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdf_min: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdf_max: Option<Vec<f64>>,

    // === Fitted parameters (from CDF fitting) ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params_min: Option<[f64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params_max: Option<[f64; 2]>,

    // === Predicted parameters (from XGBoost) ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predicted_params: Option<[f64; 2]>,

    // === Method of moments estimate ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_params: Option<[f64; 2]>,

    // === CDF curves for plotting ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitted_cdf_min: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitted_cdf_max: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predicted_cdf: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_cdf: Option<Vec<f64>>,

    // === PDF curves for plotting ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitted_pdf_min: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitted_pdf_max: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predicted_pdf: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_pdf: Option<Vec<f64>>,

    // === Histogram ===
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

    // === Chi-square test results ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chi2_min: Option<ChiSquareResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chi2_max: Option<ChiSquareResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chi2_pred: Option<ChiSquareResult>,
}

// =============================================================================
// Application State
// =============================================================================

/// Shared application state
pub struct AppState {
    pub config: Config,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Find best model path for given sample size
    pub fn find_model(&self, kind: DistributionType, sample_size: usize) -> Option<String> {
        // Find nearest available sample size
        let nearest = self.config.models.sample_sizes
            .iter()
            .min_by_key(|&&s| (s as i64 - sample_size as i64).abs())?;

        let dist_name = match kind {
            DistributionType::Beta => "Beta",
            DistributionType::Normal => "Normal",
        };

        // Look for model file (try common patterns)
        let base = format!("{}/xgb_{}_{}", self.config.models.models_dir, dist_name, nearest);
        
        // Try to find the latest model file
        if let Ok(entries) = std::fs::read_dir(&self.config.models.models_dir) {
            let prefix = format!("xgb_{}_{}_", dist_name, nearest);
            let mut matches: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
                .collect();
            
            matches.sort_by_key(|e| std::cmp::Reverse(e.file_name())); // Latest first
            
            if let Some(entry) = matches.first() {
                return Some(entry.path().to_string_lossy().into_owned());
            }
        }

        // Fallback: try exact name
        for ext in &[".ubj", ".json"] {
            let path = format!("{}{}", base, ext);
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }

        None
    }
}

// =============================================================================
// Handler Functions
// =============================================================================

/// Handle "about" command
pub fn handle_about() -> ApiResponse {
    ApiResponse {
        command: "about".into(),
        success: true,
        version: Some(env!("CARGO_PKG_VERSION").into()),
        message: Some("Quality Control Room Server".into()),
        ..Default::default()
    }
}

/// Handle "analyze" command - full analysis pipeline
pub fn handle_analyze(req: &ApiRequest, state: &Arc<AppState>) -> ApiResponse {
    let mut resp = ApiResponse {
        command: "analyze".into(),
        ..Default::default()
    };

    // Parse distribution type
    let kind = match DistributionType::from_u8(req.distribution) {
        Some(k) => k,
        None => {
            resp.message = Some(format!("Invalid distribution type: {}", req.distribution));
            return resp;
        }
    };

    // Validate data
    if req.data.is_empty() {
        resp.message = Some("Data is empty".into());
        return resp;
    }

    if req.data.iter().any(|x| x.is_nan() || x.is_infinite()) {
        resp.message = Some("Data contains NaN or infinite values".into());
        return resp;
    }

    // Get bounds
    let min_val = req.min_value.unwrap_or_else(|| {
        req.data.iter().cloned().fold(f64::INFINITY, f64::min)
    });
    let max_val = req.max_value.unwrap_or_else(|| {
        req.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    });

    if min_val >= max_val {
        resp.message = Some("min_value must be less than max_value".into());
        return resp;
    }

    // Scale data to [0, 1]
    let mut scaled = scale_data(&req.data, min_val, max_val);
    scaled.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let sample_size = scaled.len();
    let population_size = req.population_size
        .unwrap_or(state.config.statistics.default_population_size);
    let num_bins = req.bins.unwrap_or(state.config.statistics.default_bins);

    // Domain for plotting
    let domain = kind.domain();

    // Confidence intervals
    let (cdf_min, cdf_max) = conf_int(
        population_size,
        sample_size,
        state.config.statistics.prob_threshold_factor,
    );

    // Method of moments estimate
    let sampling_params = method_of_moments(kind, &scaled);

    // TODO: CDF fitting via Nelder-Mead (for now, use method of moments as placeholder)
    // This should be implemented properly - for now we'll use sampling_params
    let params_min = sampling_params;
    let params_max = sampling_params;

    // Find and load XGBoost model for prediction
    let predicted_params = if let Some(model_path) = state.find_model(kind, sample_size) {
        let features = [
            params_min[0] as f32,
            params_min[1] as f32,
            params_max[0] as f32,
            params_max[1] as f32,
        ];
        match xgb::predict(features, &model_path) {
            Ok(pred) => Some([pred[0] as f64, pred[1] as f64]),
            Err(e) => {
                resp.message = Some(format!("Prediction failed: {}", e));
                None
            }
        }
    } else {
        resp.message = Some("No model found for sample size".into());
        None
    };

    // Compute CDF/PDF curves
    let fitted_cdf_min = survival_cdf(kind, &domain, params_min);
    let fitted_cdf_max = survival_cdf(kind, &domain, params_max);
    let fitted_pdf_min = pdf(kind, &domain, params_min);
    let fitted_pdf_max = pdf(kind, &domain, params_max);
    let sampling_cdf = survival_cdf(kind, &domain, sampling_params);
    let sampling_pdf = pdf(kind, &domain, sampling_params);

    // Histogram
    let bins = bin_edges(domain[0], *domain.last().unwrap(), num_bins);
    let observed = frequencies(&bins, &scaled);
    let exp_min = expected_freq(kind, params_min, &bins, sample_size);
    let exp_max = expected_freq(kind, params_max, &bins, sample_size);

    // Chi-square tests
    let chi2_min = chi_square_test(&observed, &exp_min, state.config.statistics.alpha);
    let chi2_max = chi_square_test(&observed, &exp_max, state.config.statistics.alpha);

    // Build response
    resp.success = true;
    resp.population_size = Some(population_size);
    resp.sample_size = Some(sample_size);
    resp.min_value = Some(min_val);
    resp.max_value = Some(max_val);
    resp.scaled_data = Some(scaled);
    resp.domain = Some(domain);
    resp.cdf_min = Some(cdf_min);
    resp.cdf_max = Some(cdf_max);
    resp.params_min = Some(params_min);
    resp.params_max = Some(params_max);
    resp.sampling_params = Some(sampling_params);
    resp.fitted_cdf_min = Some(fitted_cdf_min);
    resp.fitted_cdf_max = Some(fitted_cdf_max);
    resp.fitted_pdf_min = Some(fitted_pdf_min);
    resp.fitted_pdf_max = Some(fitted_pdf_max);
    resp.sampling_cdf = Some(sampling_cdf);
    resp.sampling_pdf = Some(sampling_pdf);
    resp.bin_edges = Some(bins);
    resp.observed_freq = Some(observed);
    resp.expected_freq_min = Some(exp_min);
    resp.expected_freq_max = Some(exp_max);
    resp.chi2_min = Some(chi2_min);
    resp.chi2_max = Some(chi2_max);

    // Add predicted results if available
    if let Some(pred) = predicted_params {
        resp.predicted_params = Some(pred);
        resp.predicted_cdf = Some(survival_cdf(kind, resp.domain.as_ref().unwrap(), pred));
        resp.predicted_pdf = Some(pdf(kind, resp.domain.as_ref().unwrap(), pred));
        
        let exp_pred = expected_freq(kind, pred, resp.bin_edges.as_ref().unwrap(), sample_size);
        resp.chi2_pred = Some(chi_square_test(
            resp.observed_freq.as_ref().unwrap(),
            &exp_pred,
            state.config.statistics.alpha,
        ));
        resp.expected_freq_pred = Some(exp_pred);
    }

    resp
}

/// Handle "update_bins" command - recalculate histograms with new bin count
pub fn handle_update_bins(req: &ApiRequest, state: &Arc<AppState>) -> ApiResponse {
    let mut resp = ApiResponse {
        command: "update_bins".into(),
        ..Default::default()
    };

    let kind = match DistributionType::from_u8(req.distribution) {
        Some(k) => k,
        None => {
            resp.message = Some(format!("Invalid distribution type: {}", req.distribution));
            return resp;
        }
    };

    if req.data.is_empty() {
        resp.message = Some("Data is empty".into());
        return resp;
    }

    let min_val = req.min_value.unwrap_or(0.0);
    let max_val = req.max_value.unwrap_or(1.0);
    let scaled = scale_data(&req.data, min_val, max_val);
    let sample_size = scaled.len();
    let num_bins = req.bins.unwrap_or(state.config.statistics.default_bins);

    let domain = kind.domain();
    let bins = bin_edges(domain[0], *domain.last().unwrap(), num_bins);
    let observed = frequencies(&bins, &scaled);

    resp.bin_edges = Some(bins.clone());
    resp.observed_freq = Some(observed.clone());

    // Recalculate expected frequencies if params provided
    if let Some(params) = req.params_min {
        let exp = expected_freq(kind, params, &bins, sample_size);
        resp.chi2_min = Some(chi_square_test(&observed, &exp, state.config.statistics.alpha));
        resp.expected_freq_min = Some(exp);
    }

    if let Some(params) = req.params_max {
        let exp = expected_freq(kind, params, &bins, sample_size);
        resp.chi2_max = Some(chi_square_test(&observed, &exp, state.config.statistics.alpha));
        resp.expected_freq_max = Some(exp);
    }

    if let Some(params) = req.predicted_params {
        let exp = expected_freq(kind, params, &bins, sample_size);
        resp.chi2_pred = Some(chi_square_test(&observed, &exp, state.config.statistics.alpha));
        resp.expected_freq_pred = Some(exp);
    }

    resp.success = true;
    resp
}

/// Route request to appropriate handler
pub fn handle_request(req: &ApiRequest, state: &Arc<AppState>) -> ApiResponse {
    match req.command.as_str() {
        "about" => handle_about(),
        "analyze" => handle_analyze(req, state),
        "update_bins" => handle_update_bins(req, state),
        _ => ApiResponse {
            command: req.command.clone(),
            success: false,
            message: Some(format!("Unknown command: {}", req.command)),
            ..Default::default()
        },
    }
}
