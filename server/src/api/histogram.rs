//! Handler: get_histogram

use super::state::AppState;
use super::types::{ApiRequest, ApiResponse};
use crate::stats::{bin_edges, chi_square_test, expected_freq, frequencies, DistributionType};
use std::sync::Arc;

/// Handle "get_histogram" - histogram with expected frequencies
pub fn handle_get_histogram(req: &ApiRequest, state: &Arc<AppState>) -> ApiResponse {
    let mut resp = ApiResponse {
        command: "get_histogram".into(),
        ..Default::default()
    };

    let kind = match DistributionType::from_u8(req.distribution) {
        Some(k) => k,
        None => {
            resp.message = Some(format!("Invalid distribution type: {}", req.distribution));
            return resp;
        }
    };

    let scaled = match &req.scaled_data {
        Some(d) if !d.is_empty() => d,
        _ => {
            resp.message = Some("scaled_data required".into());
            return resp;
        }
    };

    let sample_size = scaled.len();
    let num_bins = req.bins.unwrap_or(state.config.statistics.default_bins);

    let domain = kind.domain();
    let bins = bin_edges(domain[0], *domain.last().unwrap(), num_bins);
    let observed = frequencies(&bins, scaled);

    resp.bin_edges = Some(bins.clone());
    resp.observed_freq = Some(observed.clone());

    if let Some(params) = req.params_min {
        let exp = expected_freq(kind, params, &bins, sample_size);
        resp.chi2_min = Some(chi_square_test(
            &observed,
            &exp,
            state.config.statistics.alpha,
        ));
        resp.expected_freq_min = Some(exp);
    }
    if let Some(params) = req.params_max {
        let exp = expected_freq(kind, params, &bins, sample_size);
        resp.chi2_max = Some(chi_square_test(
            &observed,
            &exp,
            state.config.statistics.alpha,
        ));
        resp.expected_freq_max = Some(exp);
    }
    if let Some(params) = req.predicted_params {
        let exp = expected_freq(kind, params, &bins, sample_size);
        resp.chi2_pred = Some(chi_square_test(
            &observed,
            &exp,
            state.config.statistics.alpha,
        ));
        resp.expected_freq_pred = Some(exp);
    }

    resp.success = true;
    resp
}
