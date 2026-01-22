//! Handlers: about, analyze

use super::state::AppState;
use super::types::{ApiRequest, ApiResponse};
use crate::stats::{
    bin_edges, cdf, chi_square_test, expected_freq, fit_ci_curves, frequencies, generate_sample,
    method_of_moments, pdf, scale_data, DistributionType,
};
use crate::xgb;
use std::sync::Arc;

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

/// Handle "analyze" - core analysis, returns params and chi2 only
pub fn handle_analyze(req: &ApiRequest, state: &Arc<AppState>) -> ApiResponse {
    let mut resp = ApiResponse {
        command: "analyze".into(),
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

    if req.data.iter().any(|x| x.is_nan() || x.is_infinite()) {
        resp.message = Some("Data contains NaN or infinite values".into());
        return resp;
    }

    let min_val = req
        .min_value
        .unwrap_or_else(|| req.data.iter().cloned().fold(f64::INFINITY, f64::min));
    let max_val = req
        .max_value
        .unwrap_or_else(|| req.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max));

    if min_val >= max_val {
        resp.message = Some("min_value must be less than max_value".into());
        return resp;
    }

    // Scale data to [0, 1]
    let mut scaled = scale_data(&req.data, min_val, max_val);
    scaled.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let sample_size = scaled.len();
    let population_size = req
        .population_size
        .unwrap_or(state.config.statistics.default_population_size);

    // Method of moments estimate
    let sampling_params = method_of_moments(kind, &scaled);

    // Fit CDF curves to confidence interval bounds
    let (params_min, params_max) = fit_ci_curves(
        kind,
        &scaled,
        population_size,
        state.config.statistics.prob_threshold_factor,
    );

    // XGBoost prediction
    let predicted_params = if let Some(model_path) = state.find_model(kind, sample_size) {
        tracing::info!("Using model: {} for sample_size={}", model_path, sample_size);
        let features = [
            params_min[0] as f32,
            params_min[1] as f32,
            params_max[0] as f32,
            params_max[1] as f32,
        ];
        tracing::debug!("Prediction features: {:?}", features);
        match xgb::predict(features, &model_path) {
            Ok(pred) => {
                tracing::info!("Prediction result: {:?}", pred);
                Some([pred[0] as f64, pred[1] as f64])
            },
            Err(e) => {
                resp.message = Some(format!("Prediction failed: {}", e));
                None
            }
        }
    } else {
        resp.message = Some("No model found for sample size".into());
        None
    };

    // Chi-square tests (quick, using default bins)
    let domain = kind.domain();
    let num_bins = state.config.statistics.default_bins;
    let bins = bin_edges(domain[0], *domain.last().unwrap(), num_bins);
    let observed = frequencies(&bins, &scaled);

    let exp_min = expected_freq(kind, params_min, &bins, sample_size);
    let chi2_min = chi_square_test(&observed, &exp_min, state.config.statistics.alpha);

    let exp_max = expected_freq(kind, params_max, &bins, sample_size);
    let chi2_max = chi_square_test(&observed, &exp_max, state.config.statistics.alpha);

    let chi2_pred = predicted_params.map(|pred| {
        let exp = expected_freq(kind, pred, &bins, sample_size);
        chi_square_test(&observed, &exp, state.config.statistics.alpha)
    });

    // Build minimal response
    resp.success = true;
    resp.sample_size = Some(sample_size);
    resp.population_size = Some(population_size);
    resp.min_value = Some(min_val);
    resp.max_value = Some(max_val);
    resp.scaled_data = Some(scaled);
    resp.params_min = Some(params_min);
    resp.params_max = Some(params_max);
    resp.predicted_params = predicted_params;
    resp.sampling_params = Some(sampling_params);
    resp.chi2_min = Some(chi2_min);
    resp.chi2_max = Some(chi2_max);
    resp.chi2_pred = chi2_pred;

    resp
}

/// Handle "generate_test_data" - generate random samples from a distribution
pub fn handle_generate_test_data(req: &ApiRequest) -> ApiResponse {
    let mut resp = ApiResponse {
        command: "generate_test_data".into(),
        ..Default::default()
    };

    let kind = match DistributionType::from_u8(req.distribution) {
        Some(k) => k,
        None => {
            resp.message = Some(format!("Invalid distribution type: {}", req.distribution));
            return resp;
        }
    };

    let params = match req.params {
        Some(p) => p,
        None => {
            resp.message = Some("Missing params parameter".into());
            return resp;
        }
    };

    let sample_size = req.sample_size.unwrap_or(50);
    let min_value = req.min_value.unwrap_or(0.0);
    let max_value = req.max_value.unwrap_or(100.0);

    if min_value >= max_value {
        resp.message = Some("min_value must be less than max_value".into());
        return resp;
    }

    match generate_sample(kind, params, sample_size, min_value, max_value) {
        Ok(samples) => {
            tracing::info!(
                "Generated {} samples from {:?} with params {:?}",
                sample_size,
                kind,
                params
            );
            
            // Compute true CDF and PDF curves
            let domain = kind.domain();
            let true_cdf: Vec<f64> = cdf(kind, &domain, params)
                .into_iter()
                .map(|x| 1.0 - x)  // survival CDF (1 - CDF)
                .collect();
            let true_pdf = pdf(kind, &domain, params);
            
            resp.success = true;
            resp.test_data = Some(samples);
            resp.min_value = Some(min_value);
            resp.max_value = Some(max_value);
            resp.sample_size = Some(sample_size);
            resp.test_params = Some(params);
            resp.test_cdf = Some(true_cdf);
            resp.test_pdf = Some(true_pdf);
            resp.domain = Some(domain);
        }
        Err(e) => {
            resp.message = Some(e);
        }
    }

    resp
}
