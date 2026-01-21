//! Data generation module for models_gen
//!
//! Handles synthetic distribution data generation, confidence intervals,
//! and CDF curve fitting for feature preparation.

use interp::{interp_slice, InterpMode};
use nlopt::{Algorithm, Nlopt, Target::Minimize};
use rand::rng;
use rand_distr::{Beta as RandBeta, Distribution, Normal as RandNormal};
use rayon::prelude::*;
use statrs::distribution::{Beta, ContinuousCDF, Discrete, Hypergeometric, Normal};
use statrs::statistics::Statistics;
use std::fmt;

// =============================================================================
// Distribution Types
// =============================================================================

/// Supported distribution types
#[derive(Debug, Clone, PartialEq)]
pub enum DistributionType {
    Beta,
    Normal,
}

impl fmt::Display for DistributionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistributionType::Beta => write!(f, "Beta"),
            DistributionType::Normal => write!(f, "Normal"),
        }
    }
}

impl DistributionType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Beta" => Some(DistributionType::Beta),
            "Normal" => Some(DistributionType::Normal),
            _ => None,
        }
    }

    /// Parameter bounds for optimization
    fn param_bounds(&self) -> [[f64; 2]; 2] {
        match self {
            DistributionType::Beta => [[0.1, 10.0], [0.1, 10.0]],
            DistributionType::Normal => [[1e-3, 1.0 - 1e-3], [1e-3, 1.0 / 6.0]],
        }
    }

    /// Initial guess for optimization
    fn init_guess(&self) -> [f64; 2] {
        match self {
            DistributionType::Beta => [0.1, 0.1],
            DistributionType::Normal => [0.5, 1.0 / 6.0],
        }
    }

    /// Domain for interpolation
    fn domain(&self) -> Vec<f64> {
        match self {
            DistributionType::Beta => linspace(0.0, 1.0, 101),
            DistributionType::Normal => linspace(-0.5, 1.5, 201),
        }
    }

    /// Compute 1 - CDF(x) for given parameters
    fn survival_cdf(&self, x: f64, params: &[f64]) -> Option<f64> {
        match self {
            DistributionType::Beta => Beta::new(params[0], params[1]).ok().map(|d| 1.0 - d.cdf(x)),
            DistributionType::Normal => Normal::new(params[0], params[1]).ok().map(|d| 1.0 - d.cdf(x)),
        }
    }

    /// Create random distribution for sampling
    fn create_rand_dist(&self, p1: f64, p2: f64) -> RandDistribution {
        match self {
            DistributionType::Beta => RandDistribution::Beta(RandBeta::new(p1, p2).unwrap()),
            DistributionType::Normal => RandDistribution::Normal(RandNormal::new(p1, p2).unwrap()),
        }
    }
}

/// Random distribution wrapper for sampling
#[derive(Clone)]
pub enum RandDistribution {
    Beta(RandBeta<f64>),
    Normal(RandNormal<f64>),
}

impl Distribution<f64> for RandDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        match self {
            RandDistribution::Beta(d) => d.sample(rng),
            RandDistribution::Normal(d) => d.sample(rng),
        }
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Generate evenly spaced values
fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| start + i as f64 * (end - start) / (n - 1) as f64)
        .collect()
}

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

// =============================================================================
// Confidence Intervals
// =============================================================================

/// Calculate quality interval using hypergeometric distribution
fn quality_interval(pop_size: u64, samp_size: u64, samp_successes: u64) -> (f64, f64) {
    const PROB_THRESHOLD_FACTOR: f64 = 10.0;

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

    let threshold = Statistics::max(&prob) / PROB_THRESHOLD_FACTOR;
    let idx_min = prob.iter().position(|&x| x >= threshold).unwrap_or(0);
    let idx_max = prob.iter().rposition(|&x| x >= threshold).unwrap_or(prob.len() - 1);

    (
        (idx_min as u64 + samp_successes) as f64 / pop_size as f64,
        (idx_max as u64 + samp_successes) as f64 / pop_size as f64,
    )
}

/// Calculate confidence intervals for all sample outcomes
pub fn conf_int(population_size: usize, sample_size: usize) -> (Vec<f64>, Vec<f64>) {
    let mut cdf_min = vec![0.0; sample_size + 2];
    let mut cdf_max = vec![0.0; sample_size + 2];
    cdf_min[0] = 1.0;
    cdf_max[0] = 1.0;

    for (i, k) in (1..=sample_size).rev().enumerate() {
        let (lo, hi) = quality_interval(
            population_size as u64,
            sample_size as u64,
            k as u64,
        );
        cdf_min[i + 1] = lo;
        cdf_max[i + 1] = hi;
    }

    (cdf_min, cdf_max)
}

// =============================================================================
// Target Data Preparation
// =============================================================================

/// Prepare target data (distribution parameters) and random samplers
pub fn target_prepare(
    kind: &DistributionType,
    params_res: [usize; 2],
    dist_train_size: usize,
) -> (Vec<[f64; 2]>, Vec<RandDistribution>) {
    let bounds = kind.param_bounds();
    let p1_range = linspace(bounds[0][0], bounds[0][1], params_res[0]);
    let p2_range = linspace(bounds[1][0], bounds[1][1], params_res[1]);

    let total = params_res[0] * params_res[1] * dist_train_size;
    let mut y = Vec::with_capacity(total);
    let mut dist = Vec::with_capacity(total);

    for p1 in &p1_range {
        for p2 in &p2_range {
            for _ in 0..dist_train_size {
                y.push([*p1, *p2]);
                dist.push(kind.create_rand_dist(*p1, *p2));
            }
        }
    }

    (y, dist)
}

// =============================================================================
// Feature Preparation (CDF Curve Fitting)
// =============================================================================

/// MSE cost function for optimization
fn mse_cost(
    params: &[f64],
    _grad: Option<&mut [f64]>,
    data: &mut (&Vec<f64>, &Vec<f64>, &DistributionType),
) -> f64 {
    let (domain, target, kind) = data;
    let n = domain.len();

    // Invalid parameters → high cost
    if params[0] <= 0.0 || params[1] <= 0.0 {
        return 1e10;
    }

    let mut sse = 0.0;
    for i in 0..n {
        match kind.survival_cdf(domain[i], params) {
            Some(pred) => sse += (pred - target[i]).powi(2),
            None => return 1e10,
        }
    }

    sse / n as f64
}

/// Fit distribution parameters to CDF data using Nelder-Mead
fn fit_cdf(domain: &Vec<f64>, target: &Vec<f64>, kind: &DistributionType) -> [f64; 2] {
    let bounds = kind.param_bounds();
    let init = kind.init_guess();

    let mut opt = Nlopt::new(
        Algorithm::Neldermead,
        2,
        mse_cost,
        Minimize,
        (domain, target, kind),
    );

    opt.set_lower_bounds(&[bounds[0][0] * 0.7, bounds[1][0] * 0.7]).unwrap();
    opt.set_upper_bounds(&[bounds[0][1] * 1.3, bounds[1][1] * 1.3]).unwrap();
    opt.set_maxeval(10000).unwrap();
    opt.set_xtol_abs1(1e-20).unwrap();

    let mut result = init;
    let _ = opt.optimize(&mut result);
    result
}

/// Prepare features using CDF curve fitting (parallel)
pub fn features_prepare_nm(
    sample_size: usize,
    cdf_min: Vec<f64>,
    cdf_max: Vec<f64>,
    dist: Vec<RandDistribution>,
    kind: &DistributionType,
) -> Vec<[f64; 4]> {
    let domain = kind.domain();
    let anchors = [domain[0], *domain.last().unwrap()];

    dist.par_iter()
        .map(|d| {
            let mut rng = rng();

            // Sample and sort
            let mut samples: Vec<f64> = Vec::with_capacity(sample_size + 2);
            samples.push(anchors[0]);
            samples.extend((0..sample_size).map(|_| d.sample(&mut rng)));
            samples.push(anchors[1]);
            samples.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

            // Interpolate confidence bounds
            let cdf_min_interp = interp_slice(&samples, &cdf_min, &domain, &InterpMode::default());
            let cdf_max_interp = interp_slice(&samples, &cdf_max, &domain, &InterpMode::default());

            // Fit curves
            let params_min = fit_cdf(&domain, &cdf_min_interp, kind);
            let params_max = fit_cdf(&domain, &cdf_max_interp, kind);

            [params_min[0], params_min[1], params_max[0], params_max[1]]
        })
        .collect()
}

// =============================================================================
// Data Flattening for FFI
// =============================================================================

/// Flatten array of arrays to f32 vector for FFI
pub fn flat_vector<const N: usize>(vec: &[[f64; N]]) -> Vec<f32> {
    vec.iter().flatten().map(|&x| x as f32).collect()
}
