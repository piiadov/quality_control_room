//! Statistical functions module
//!
//! Confidence intervals, CDF/PDF computations, chi-square tests,
//! histogram generation, and method of moments estimation.

use serde::Serialize;
use statrs::distribution::{Beta, ChiSquared, ContinuousCDF, Continuous, Discrete, Hypergeometric, Normal};
use statrs::statistics::Statistics;

// =============================================================================
// Constants
// =============================================================================

/// Number of anchor points for CDF interpolation
const NUM_ANCHORS: usize = 2;

/// Domain grid resolution for Beta distribution
const BETA_DOMAIN_POINTS: usize = 101;

/// Domain grid resolution for Normal distribution  
const NORMAL_DOMAIN_POINTS: usize = 201;

/// Normal distribution domain margin beyond [0,1]
const NORMAL_DOMAIN_MARGIN: f64 = 0.5;

// =============================================================================
// Distribution Types
// =============================================================================

/// Supported distribution types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistributionType {
    Beta = 0,
    Normal = 1,
}

impl DistributionType {
    /// Create from integer (for API compatibility)
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(DistributionType::Beta),
            1 => Some(DistributionType::Normal),
            _ => None,
        }
    }

    /// Get domain for interpolation
    pub fn domain(&self) -> Vec<f64> {
        match self {
            DistributionType::Beta => linspace(0.0, 1.0, BETA_DOMAIN_POINTS),
            DistributionType::Normal => linspace(
                -NORMAL_DOMAIN_MARGIN,
                1.0 + NORMAL_DOMAIN_MARGIN,
                NORMAL_DOMAIN_POINTS,
            ),
        }
    }

    /// Parameter bounds for validation
    pub fn param_bounds(&self) -> [[f64; 2]; 2] {
        match self {
            DistributionType::Beta => [[0.1, 10.0], [0.1, 10.0]],
            DistributionType::Normal => [[1e-3, 1.0 - 1e-3], [1e-3, 1.0 / 6.0]],
        }
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Generate evenly spaced values
pub fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| start + i as f64 * (end - start) / (n - 1) as f64)
        .collect()
}

/// Scale data from [min_val, max_val] to [0, 1]
pub fn scale_data(data: &[f64], min_val: f64, max_val: f64) -> Vec<f64> {
    let range = max_val - min_val;
    data.iter().map(|&x| (x - min_val) / range).collect()
}

// =============================================================================
// Hypergeometric Confidence Intervals
// =============================================================================

/// Fallback hypergeometric PMF for large numbers (log-space computation)
fn hypergeometric_pmf_fallback(n_total: u64, k_total: u64, n: u64, k: u64) -> f64 {
    if k > k_total || k > n || n > n_total || n - k > n_total - k_total {
        return 0.0;
    }

    let log_factorial = |m: u64| (1..=m).fold(0.0, |acc, x| acc + (x as f64).ln());
    let log_comb = |a: u64, b: u64| log_factorial(a) - log_factorial(b) - log_factorial(a - b);

    let log_pmf = log_comb(k_total, k) + log_comb(n_total - k_total, n - k) - log_comb(n_total, n);
    log_pmf.exp()
}

/// Calculate quality interval using hypergeometric distribution
fn quality_interval(pop_size: u64, samp_size: u64, samp_successes: u64, threshold_factor: f64) -> (f64, f64) {
    let prob: Vec<f64> = (samp_successes..=pop_size - samp_size + samp_successes)
        .map(|pop_successes| {
            let p = Hypergeometric::new(pop_size, pop_successes, samp_size)
                .unwrap()
                .pmf(samp_successes);
            if p.is_nan() {
                hypergeometric_pmf_fallback(pop_size, pop_successes, samp_size, samp_successes)
            } else {
                p
            }
        })
        .collect();

    let threshold = Statistics::max(&prob) / threshold_factor;
    let idx_min = prob.iter().position(|&x| x >= threshold).unwrap_or(0);
    let idx_max = prob.iter().rposition(|&x| x >= threshold).unwrap_or(prob.len() - 1);

    (
        (idx_min as u64 + samp_successes) as f64 / pop_size as f64,
        (idx_max as u64 + samp_successes) as f64 / pop_size as f64,
    )
}

/// Calculate confidence intervals for all sample outcomes
pub fn conf_int(population_size: usize, sample_size: usize, threshold_factor: f64) -> (Vec<f64>, Vec<f64>) {
    let mut cdf_min = vec![0.0; sample_size + NUM_ANCHORS];
    let mut cdf_max = vec![0.0; sample_size + NUM_ANCHORS];
    cdf_min[0] = 1.0;
    cdf_max[0] = 1.0;

    for (i, k) in (1..=sample_size).rev().enumerate() {
        let (lo, hi) = quality_interval(
            population_size as u64,
            sample_size as u64,
            k as u64,
            threshold_factor,
        );
        cdf_min[i + 1] = lo;
        cdf_max[i + 1] = hi;
    }

    (cdf_min, cdf_max)
}

// =============================================================================
// CDF and PDF Computation
// =============================================================================

/// Compute CDF values for given parameters
pub fn cdf(kind: DistributionType, domain: &[f64], params: [f64; 2]) -> Vec<f64> {
    match kind {
        DistributionType::Beta => {
            if let Ok(dist) = Beta::new(params[0], params[1]) {
                domain.iter().map(|&x| dist.cdf(x)).collect()
            } else {
                vec![f64::NAN; domain.len()]
            }
        }
        DistributionType::Normal => {
            if let Ok(dist) = Normal::new(params[0], params[1]) {
                domain.iter().map(|&x| dist.cdf(x)).collect()
            } else {
                vec![f64::NAN; domain.len()]
            }
        }
    }
}

/// Compute survival CDF (1 - CDF) values
pub fn survival_cdf(kind: DistributionType, domain: &[f64], params: [f64; 2]) -> Vec<f64> {
    cdf(kind, domain, params).into_iter().map(|x| 1.0 - x).collect()
}

/// Compute PDF values for given parameters
pub fn pdf(kind: DistributionType, domain: &[f64], params: [f64; 2]) -> Vec<f64> {
    match kind {
        DistributionType::Beta => {
            if let Ok(dist) = Beta::new(params[0], params[1]) {
                domain.iter().map(|&x| dist.pdf(x)).collect()
            } else {
                vec![f64::NAN; domain.len()]
            }
        }
        DistributionType::Normal => {
            if let Ok(dist) = Normal::new(params[0], params[1]) {
                domain.iter().map(|&x| dist.pdf(x)).collect()
            } else {
                vec![f64::NAN; domain.len()]
            }
        }
    }
}

// =============================================================================
// Histogram and Frequencies
// =============================================================================

/// Generate histogram bin edges
pub fn bin_edges(start: f64, end: f64, num_bins: usize) -> Vec<f64> {
    linspace(start, end, num_bins + 1)
}

/// Compute observed frequencies (histogram)
pub fn frequencies(bins: &[f64], data: &[f64]) -> Vec<f64> {
    let num_bins = bins.len() - 1;
    let mut freq = vec![0.0; num_bins];

    for &x in data {
        for i in 0..num_bins {
            if x >= bins[i] && x < bins[i + 1] {
                freq[i] += 1.0;
                break;
            }
        }
        // Handle edge case: x == last bin edge
        if (x - bins[num_bins]).abs() < 1e-10 {
            freq[num_bins - 1] += 1.0;
        }
    }

    freq
}

/// Compute expected frequencies from distribution
pub fn expected_freq(kind: DistributionType, params: [f64; 2], bins: &[f64], sample_size: usize) -> Vec<f64> {
    let num_bins = bins.len() - 1;
    let cdf_vals = cdf(kind, bins, params);

    (0..num_bins)
        .map(|i| (cdf_vals[i + 1] - cdf_vals[i]) * sample_size as f64)
        .collect()
}

// =============================================================================
// Chi-Square Test
// =============================================================================

/// Chi-square goodness-of-fit test result
#[derive(Debug, Clone, Serialize)]
pub struct ChiSquareResult {
    pub chi2: f64,
    pub p_value: f64,
    pub critical_value: f64,
    pub reject_null: bool,
    pub degrees_of_freedom: usize,
}

/// Perform chi-square goodness-of-fit test
pub fn chi_square_test(observed: &[f64], expected: &[f64], alpha: f64) -> ChiSquareResult {
    assert_eq!(observed.len(), expected.len(), "Observed and expected must have same length");

    // Calculate chi-square statistic
    let chi2: f64 = observed
        .iter()
        .zip(expected.iter())
        .filter(|(_, &e)| e > 0.0)
        .map(|(&o, &e)| (o - e).powi(2) / e)
        .sum();

    // Degrees of freedom = number of bins - 1 - number of estimated parameters
    // For distribution fitting, we estimate 2 parameters
    let df = observed.len().saturating_sub(3).max(1);

    let chi_dist = ChiSquared::new(df as f64).expect("Invalid degrees of freedom");
    let critical_value = chi_dist.inverse_cdf(1.0 - alpha);
    let p_value = 1.0 - chi_dist.cdf(chi2);
    let reject_null = chi2 > critical_value;

    ChiSquareResult {
        chi2,
        p_value,
        critical_value,
        reject_null,
        degrees_of_freedom: df,
    }
}

// =============================================================================
// Method of Moments Estimation
// =============================================================================

/// Estimate distribution parameters using method of moments
pub fn method_of_moments(kind: DistributionType, data: &[f64]) -> [f64; 2] {
    let mean = Statistics::mean(data);
    let variance = Statistics::variance(data);
    let std_dev = variance.sqrt();

    match kind {
        DistributionType::Beta => {
            // Beta distribution method of moments
            // α = μ * ((μ*(1-μ)/σ²) - 1)
            // β = (1-μ) * ((μ*(1-μ)/σ²) - 1)
            if variance > 0.0 && mean > 0.0 && mean < 1.0 {
                let factor = mean * (1.0 - mean) / variance - 1.0;
                if factor > 0.0 {
                    let alpha = mean * factor;
                    let beta = (1.0 - mean) * factor;
                    return [alpha.max(0.1), beta.max(0.1)];
                }
            }
            [1.0, 1.0] // Fallback to uniform
        }
        DistributionType::Normal => {
            [mean, std_dev.max(1e-6)]
        }
    }
}
