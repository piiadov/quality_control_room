//! Data generation module for models_gen
//!
//! Handles synthetic distribution data generation, confidence intervals,
//! and CDF curve fitting for feature preparation.

use interp::{interp_slice, InterpMode};
use nlopt::{Algorithm, Nlopt, Target::Minimize};
use rand::rng;
use rand_distr::{Beta as RandBeta, Normal as RandNormal, Distribution};
use rayon::prelude::*;
use statrs::distribution::{Beta, ContinuousCDF, Discrete, Hypergeometric, Normal};
use statrs::statistics::Statistics;
use std::fmt;

/// Distribution type enumeration
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
    /// Parse distribution type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Beta" => Some(DistributionType::Beta),
            "Normal" => Some(DistributionType::Normal),
            _ => None,
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
            RandDistribution::Beta(beta) => beta.sample(rng),
            RandDistribution::Normal(normal) => normal.sample(rng),
        }
    }
}

/// Generate a range of values between bounds
pub fn generate_range(bounds: [f64; 2], resolution: usize) -> Vec<f64> {
    (0..resolution)
        .map(|i| bounds[0] + i as f64 * (bounds[1] - bounds[0]) / (resolution - 1) as f64)
        .collect()
}

/// Get parameter bounds for a distribution type
pub fn param_bounds(kind: &DistributionType) -> [[f64; 2]; 2] {
    match kind {
        DistributionType::Beta => [[0.1, 10.0], [0.1, 10.0]],
        DistributionType::Normal => [[1e-3, 1.0 - 1e-3], [1e-3, 1.0 / 6.0]],
    }
}

/// Get initial guess for curve fitting
pub fn init_guess(kind: &DistributionType) -> [f64; 2] {
    match kind {
        DistributionType::Beta => [0.1, 0.1],
        DistributionType::Normal => [0.5, 1.0 / 6.0],
    }
}

/// Get interpolation domain for a distribution type
pub fn dist_domain(kind: &DistributionType) -> Vec<f64> {
    match kind {
        DistributionType::Beta => generate_range([0.0, 1.0], 101),
        DistributionType::Normal => generate_range([-0.5, 1.5], 201),
    }
}

/// Fallback hypergeometric PMF for large numbers when statrs fails
fn hypergeometric_pmf(n_total: u64, k_total: u64, n: u64, k: u64) -> f64 {
    if k > k_total || k > n || n > n_total || n - k > n_total - k_total {
        return 0.0;
    }
    
    fn log_factorial(n: u64) -> f64 {
        (1..=n).fold(0.0, |acc, x| acc + (x as f64).ln())
    }
    
    let log_comb = |a: u64, b: u64| log_factorial(a) - log_factorial(b) - log_factorial(a - b);
    let log_pmf = log_comb(k_total, k) + log_comb(n_total - k_total, n - k) - log_comb(n_total, n);
    log_pmf.exp()
}

/// Calculate quality interval using hypergeometric distribution
pub fn quality_interval(
    population_size: u64,
    sample_size: u64,
    sample_successes: u64,
    prob_threshold_factor: f64,
) -> (f64, f64) {
    let prob: Vec<f64> = (sample_successes..=population_size - sample_size + sample_successes)
        .map(|population_successes| {
            let mut p = Hypergeometric::new(population_size, population_successes, sample_size)
                .expect("Invalid parameters for hypergeometric distribution")
                .pmf(sample_successes);
            if p.is_nan() {
                p = hypergeometric_pmf(population_size, population_successes, sample_size, sample_successes);
            }
            p
        })
        .collect();

    let prob_threshold = Statistics::max(&prob) / prob_threshold_factor;

    let index_min = prob.iter().position(|&x| x >= prob_threshold).unwrap_or(0);
    let index_max = prob.iter().rposition(|&x| x >= prob_threshold).unwrap_or(prob.len() - 1);

    let quality_min = (index_min as u64 + sample_successes) as f64 / population_size as f64;
    let quality_max = (index_max as u64 + sample_successes) as f64 / population_size as f64;

    (quality_min, quality_max)
}

/// Calculate confidence intervals for all sample outcomes
pub fn conf_int(population_size: usize, sample_size: usize) -> (Vec<f64>, Vec<f64>) {
    let mut cdf_min: Vec<f64> = vec![0.0; sample_size + 2];
    cdf_min[0] = 1.0;

    let mut cdf_max: Vec<f64> = vec![0.0; sample_size + 2];
    cdf_max[0] = 1.0;

    for (i, k) in (1..=sample_size).rev().enumerate() {
        (cdf_min[i + 1], cdf_max[i + 1]) =
            quality_interval(population_size as u64, sample_size as u64, k as u64, 10.0);
    }
    
    (cdf_min, cdf_max)
}

/// Prepare target data (distribution parameters)
pub fn target_prepare(
    kind: &DistributionType,
    params_res: [usize; 2],
    dist_train_size: usize,
) -> (Vec<[f64; 2]>, Vec<RandDistribution>) {
    let bounds = param_bounds(kind);
    let p1_range = generate_range(bounds[0], params_res[0]);
    let p2_range = generate_range(bounds[1], params_res[1]);
    let iter_num = params_res[0] * params_res[1] * dist_train_size;

    let mut dist: Vec<RandDistribution> = Vec::with_capacity(iter_num);
    let mut y: Vec<[f64; 2]> = vec![[0.0, 0.0]; iter_num];

    let mut j = 0;
    for i in 0..p1_range.len() * p2_range.len() {
        let p1 = p1_range[i / p2_range.len()];
        let p2 = p2_range[i % p2_range.len()];
        
        for _ in 0..dist_train_size {
            y[j] = [p1, p2];
            
            match kind {
                DistributionType::Beta => {
                    dist.push(RandDistribution::Beta(
                        RandBeta::new(p1, p2).expect("Invalid Beta distribution parameters"),
                    ));
                }
                DistributionType::Normal => {
                    dist.push(RandDistribution::Normal(
                        RandNormal::new(p1, p2).expect("Invalid Normal distribution parameters"),
                    ));
                }
            }
            j += 1;
        }
    }
    
    (y, dist)
}

/// MSE cost function for NLopt curve fitting
fn mse_cost(
    x: &[f64],
    _grad: Option<&mut [f64]>,
    user_data: &mut (&Vec<f64>, &Vec<f64>, &DistributionType),
) -> f64 {
    let kind = user_data.2;
    let points_num = user_data.0.len();
    let mut sse = 0.0;

    match kind {
        DistributionType::Beta => {
            if x[0] <= 0.0 || x[1] <= 0.0 {
                return 1e10;
            }
            let dist = match Beta::new(x[0], x[1]) {
                Ok(b) => b,
                Err(_) => return 1e10,
            };
            for i in 0..points_num {
                let pred = 1.0 - dist.cdf(user_data.0[i]);
                sse += (pred - user_data.1[i]).powi(2);
            }
        }
        DistributionType::Normal => {
            let dist = match Normal::new(x[0], x[1]) {
                Ok(n) => n,
                Err(_) => return 1e10,
            };
            for i in 0..points_num {
                let pred = 1.0 - dist.cdf(user_data.0[i]);
                sse += (pred - user_data.1[i]).powi(2);
            }
        }
    }

    sse / points_num as f64
}

/// Prepare features using Nelder-Mead optimization for CDF curve fitting
pub fn features_prepare_nm(
    sample_size: usize,
    cdf_min: Vec<f64>,
    cdf_max: Vec<f64>,
    dist: Vec<RandDistribution>,
    kind: &DistributionType,
) -> Vec<[f64; 4]> {
    let iter_num = dist.len();
    let mut x: Vec<[f64; 4]> = vec![[0.0; 4]; iter_num];

    let bounds = param_bounds(kind);
    let init = init_guess(kind);
    let q = dist_domain(kind);
    let anchors = [q[0], q[q.len() - 1]];

    dist.par_iter()
        .zip(x.par_iter_mut())
        .for_each(|(dist_i, x_i)| {
            let mut rng = rng();
            let mut samples: Vec<f64> = Vec::with_capacity(sample_size + 2);

            samples.push(anchors[0]);
            for _ in 0..sample_size {
                samples.push(dist_i.sample(&mut rng));
            }
            samples.push(anchors[1]);

            samples.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

            let cdf_min_int: Vec<f64> = interp_slice(&samples, &cdf_min, &q, &InterpMode::default());
            let cdf_max_int: Vec<f64> = interp_slice(&samples, &cdf_max, &q, &InterpMode::default());

            // Fit curve for cdf_min
            let mut opt = Nlopt::new(
                Algorithm::Neldermead,
                2,
                mse_cost,
                Minimize,
                (&q, &cdf_min_int, kind),
            );
            opt.set_lower_bounds(&[bounds[0][0] * 0.7, bounds[1][0] * 0.7]).unwrap();
            opt.set_upper_bounds(&[bounds[0][1] * 1.3, bounds[1][1] * 1.3]).unwrap();
            opt.set_maxeval(10000).unwrap();
            opt.set_xtol_abs1(1e-20).unwrap();
            
            let mut params_min = init;
            let _ = opt.optimize(&mut params_min);

            // Fit curve for cdf_max
            let mut opt = Nlopt::new(
                Algorithm::Neldermead,
                2,
                mse_cost,
                Minimize,
                (&q, &cdf_max_int, kind),
            );
            opt.set_lower_bounds(&[bounds[0][0] * 0.7, bounds[1][0] * 0.7]).unwrap();
            opt.set_upper_bounds(&[bounds[0][1] * 1.3, bounds[1][1] * 1.3]).unwrap();
            opt.set_maxeval(10000).unwrap();
            opt.set_xtol_abs1(1e-20).unwrap();
            
            let mut params_max = init;
            let _ = opt.optimize(&mut params_max);

            *x_i = [params_min[0], params_min[1], params_max[0], params_max[1]];
        });

    x
}

/// Flatten a vector of arrays to a flat f32 vector for FFI
pub fn flat_vector<const N: usize>(vec: &[[f64; N]]) -> Vec<f32> {
    vec.iter().flatten().map(|&x| x as f32).collect()
}
