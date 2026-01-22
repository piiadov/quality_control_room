//! Handlers: get_intervals, get_cdf, get_pdf

use super::state::AppState;
use super::types::{ApiRequest, ApiResponse};
use crate::stats::{conf_int, pdf, survival_cdf, DistributionType};
use std::sync::Arc;

/// Handle "get_intervals" - confidence interval curves
pub fn handle_get_intervals(req: &ApiRequest, state: &Arc<AppState>) -> ApiResponse {
    let mut resp = ApiResponse {
        command: "get_intervals".into(),
        ..Default::default()
    };

    let kind = match DistributionType::from_u8(req.distribution) {
        Some(k) => k,
        None => {
            resp.message = Some(format!("Invalid distribution type: {}", req.distribution));
            return resp;
        }
    };

    let sample_size = req.scaled_data.as_ref().map(|d| d.len()).unwrap_or(0);
    if sample_size == 0 {
        resp.message = Some("scaled_data required".into());
        return resp;
    }

    let population_size = req
        .population_size
        .unwrap_or(state.config.statistics.default_population_size);

    let domain = kind.domain();
    let (cdf_min, cdf_max) = conf_int(
        population_size,
        sample_size,
        state.config.statistics.prob_threshold_factor,
    );

    resp.success = true;
    resp.domain = Some(domain);
    resp.cdf_min = Some(cdf_min);
    resp.cdf_max = Some(cdf_max);

    resp
}

/// Handle "get_cdf" - CDF curves for fitted/predicted params
pub fn handle_get_cdf(req: &ApiRequest) -> ApiResponse {
    let mut resp = ApiResponse {
        command: "get_cdf".into(),
        ..Default::default()
    };

    let kind = match DistributionType::from_u8(req.distribution) {
        Some(k) => k,
        None => {
            resp.message = Some(format!("Invalid distribution type: {}", req.distribution));
            return resp;
        }
    };

    let domain = kind.domain();

    if let Some(params) = req.params_min {
        resp.fitted_cdf_min = Some(survival_cdf(kind, &domain, params));
    }
    if let Some(params) = req.params_max {
        resp.fitted_cdf_max = Some(survival_cdf(kind, &domain, params));
    }
    if let Some(params) = req.predicted_params {
        resp.predicted_cdf = Some(survival_cdf(kind, &domain, params));
    }
    if let Some(params) = req.sampling_params {
        resp.sampling_cdf = Some(survival_cdf(kind, &domain, params));
    }

    resp.success = true;
    resp.domain = Some(domain);
    resp
}

/// Handle "get_pdf" - PDF curves for fitted/predicted params
pub fn handle_get_pdf(req: &ApiRequest) -> ApiResponse {
    let mut resp = ApiResponse {
        command: "get_pdf".into(),
        ..Default::default()
    };

    let kind = match DistributionType::from_u8(req.distribution) {
        Some(k) => k,
        None => {
            resp.message = Some(format!("Invalid distribution type: {}", req.distribution));
            return resp;
        }
    };

    let domain = kind.domain();

    if let Some(params) = req.params_min {
        resp.fitted_pdf_min = Some(pdf(kind, &domain, params));
    }
    if let Some(params) = req.params_max {
        resp.fitted_pdf_max = Some(pdf(kind, &domain, params));
    }
    if let Some(params) = req.predicted_params {
        resp.predicted_pdf = Some(pdf(kind, &domain, params));
    }
    if let Some(params) = req.sampling_params {
        resp.sampling_pdf = Some(pdf(kind, &domain, params));
    }

    resp.success = true;
    resp.domain = Some(domain);
    resp
}
