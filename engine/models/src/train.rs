use interp::{interp_slice, InterpMode};
use nlopt::{Algorithm, Nlopt, SuccessState};
use nlopt::SuccessState::Success;
use nlopt::Target::Minimize;
use rand::rng;
use rand::seq::SliceRandom;
use rand_distr::{Beta as RandBeta, Normal as RandNormal, Distribution};
use rayon::prelude::*;
use statrs::distribution::{Discrete, Hypergeometric, Beta, Normal,
    ContinuousCDF, Continuous, ChiSquared};
use statrs::statistics::Statistics;
use std::fmt;

#[derive(PartialEq, Clone)]
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

fn hypergeometric_pmf(n_total: u64, k_total: u64, n: u64, k: u64) -> f64 {
    // Slow but can work with large numbers,
    // when statrs::distribution::Hypergeometric fails
    if k > k_total || k > n || n > n_total || n - k > n_total - k_total {
        eprintln!(
            "Out-of-bounds case in hypergeometric PMF: N {}, K {}, n {}, k {}",
            n_total, k_total, n, k
        );
        return 0.0;
    }
    fn log_factorial(n: u64) -> f64 {
        (1..=n).fold(0.0, |acc, x| acc + (x as f64).ln())
    }
    let log_comb = |a: u64, b: u64| log_factorial(a) - log_factorial(b) - log_factorial(a - b);
    let log_pmf = log_comb(k_total, k) + log_comb(n_total - k_total, n - k) - log_comb(n_total, n);
    log_pmf.exp()
}

pub fn generate_range(bounds: [f64; 2], resolution: usize) -> Vec<f64> {
    (0..resolution)
        .map(|i| bounds[0] + i as f64 * (bounds[1] - bounds[0]) / (resolution - 1) as f64)
        .collect()
}

pub fn quality_interval(population_size: u64, sample_size: u64,
                    sample_successes: u64, prob_threshold_factor: f64) -> (f64, f64) {
    let prob: Vec<f64> = (sample_successes
        ..= population_size - sample_size + sample_successes)
        .map(|population_successes|
             {
                 let mut p = Hypergeometric::new(population_size, population_successes, sample_size)
                     .expect("Invalid parameters for hypergeometric distribution")
                     .pmf(sample_successes);
                 if p.is_nan() {
                     // Fallback when Hypergeometric fails
                     // It should not be happened if user's problem defined well
                     p = hypergeometric_pmf(population_size, population_successes,
                                            sample_size, sample_successes);
                 }
                 p
             }
        )
        .collect();

    let prob_threshold = Statistics::max(&prob) / prob_threshold_factor;

    let index_min = prob.iter().position(|&x| x >= prob_threshold)
        .unwrap_or(0);
    let index_max = prob.iter().rposition(|&x| x >= prob_threshold)
        .unwrap_or(prob.len() - 1);

    let quality_min = (index_min as u64 + sample_successes) as f64 / population_size as f64;
    let quality_max = (index_max as u64 + sample_successes) as f64 / population_size as f64;

    (quality_min, quality_max)
}

fn mse_cost(x: &[f64], _grad: Option<&mut [f64]>, user_data: &mut (&Vec<f64>, &Vec<f64>, &DistributionType)) -> f64 {
    
    let kind = user_data.2;
    let points_num = user_data.0.len();
    let mut sse = 0.0;
    
    // Refactor this part to avoid code duplication, use implementation in DistributionType
    if *kind == DistributionType::Beta {
        if x[0] <= 0.0 || x[1] <= 0.0 {
            return 1e10;
        }
        let dist = match Beta::new(x[0], x[1]) {
            Ok(b) => b,
            Err(_) => return 1e10,
        };
        for i in 0 .. points_num {
            let pred = 1.0 - dist.cdf(user_data.0[i]);
            sse += (pred - user_data.1[i]).powi(2);
        }
    } else if *kind == DistributionType::Normal {
        let dist = match Normal::new(x[0], x[1]) {
            Ok(b) => b,
            Err(_) => return 1e10,
        };
        for i in 0 .. points_num {
            let pred = 1.0 - dist.cdf(user_data.0[i]);
            sse += (pred - user_data.1[i]).powi(2);
        }
    } else {
        panic!("mse_cost: Unknown distribution type");
    }

    sse / points_num as f64
}

pub fn conf_int(population_size: usize, sample_size: usize) -> (Vec<f64>, Vec<f64>) {
    // Vector of conf intervals [quality_min, quality_max]

    let mut cdf_min: Vec<f64> = vec![0.0; sample_size+2];
    cdf_min[0] = 1.0;

    let mut cdf_max: Vec<f64> = vec![0.0; sample_size+2];
    cdf_max[0] = 1.0;

    for (i,k) in (1..=sample_size).rev().enumerate() {
        (cdf_min[i+1], cdf_max[i+1])
            = quality_interval(population_size as u64, sample_size as u64, 
                k as u64, 10.0);
    }
    (cdf_min, cdf_max)
}

pub fn target_prepare(kind: DistributionType, params_res: [usize;2],
                      dist_train_size: usize) -> (Vec<[f64; 2]>, Vec<RandDistribution>){
    
    let bounds = param_bounds(kind.clone());
    let p1_range = generate_range(bounds[0], params_res[0]);
    let p2_range = generate_range(bounds[1], params_res[1]);
    let iter_num = params_res[0] * params_res[1] * dist_train_size;

    let mut dist: Vec<RandDistribution> = Vec::with_capacity(iter_num);
    
    let mut y: Vec<[f64; 2]> = Vec::with_capacity(iter_num);
    y.resize(iter_num, [0.0, 0.0]);

    let mut j = 0;
    (0 .. p1_range.len() * p2_range.len()).for_each(|i|{
        let p1 = p1_range[i / p2_range.len()];
        let p2 = p2_range[i % p2_range.len()];
        (0 .. dist_train_size).for_each(|_|{
            y[j][0] = p1;
            y[j][1] = p2;
            if kind == DistributionType::Beta {
                dist.push(RandDistribution::Beta(
                    RandBeta::new(p1, p2)
                    .expect("Invalid Beta distribution parameters"),
                ));
            } else if kind == DistributionType::Normal {
                dist.push(RandDistribution::Normal(
                    RandNormal::new(p1, p2)
                    .expect("Invalid Normal distribution parameters"),
                ));
            } else {
                panic!("target_prepare: Unknown distribution type");
            }
            j += 1;
        });
    });
    (y, dist)
}

pub fn features_prepare_nm(sample_size: usize, cdf_min: Vec<f64>,
                           cdf_max: Vec<f64>, dist: Vec<RandDistribution>,
                           kind: DistributionType) -> (Vec<[f64; 4]>, Vec<[f64; 2]>){

    let iter_num: usize = dist.len();
    let mut x: Vec<[f64; 4]> = vec![[0.0; 4]; iter_num];
    let mut opt_stat: Vec<[(SuccessState, f64); 2]> = vec![[(Success,0.0); 2]; iter_num];
    let mut sampling_params: Vec<[f64; 2]> = vec![[0.0; 2]; iter_num];

    let bounds = param_bounds(kind.clone());
    let init_guess = init_guess(kind.clone());

    // Interpolation domain for CDF
    let q = dist_domain(kind.clone());

    // "first" and "last" anchors for interpolation over all the domain
    let anchors = [q[0], q[q.len()-1]];

    dist
        .par_iter()
        .zip(x.par_iter_mut())
        .zip(opt_stat.par_iter_mut())
        .zip(sampling_params.par_iter_mut())
        .for_each(|(((dist_i, x_i), opt_stat_i), sampling_params_i)| {
            
            let mut rng = rng();
            let mut samples: Vec<f64> = Vec::with_capacity(sample_size+2);
            
            samples.push(anchors[0]);
            for _ in 1 .. sample_size + 1 {
                samples.push(dist_i.sample(&mut rng));
            }
            samples.push(anchors[1]);

            let slice = &samples[1..samples.len() - 1];
            let mean: f64 = slice.iter().sum::<f64>() / slice.len() as f64;
            let var: f64 = slice.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / (slice.len() as f64 - 1.0);
            let std = var.sqrt();

            if kind == DistributionType::Beta {
                let coeff = mean * (1.0 - mean) / var - 1.0;
                let alpha = coeff * mean;
                let beta = coeff * (1.0 - mean);
                if alpha <= 0.0 || beta <= 0.0 {
                    *sampling_params_i = [1e-3, 1e-3];
                } else {
                    *sampling_params_i = [alpha, beta];
                }
            } else if kind == DistributionType::Normal {
                *sampling_params_i = [mean, std];
            } else {
                panic!("features_prepare_nm: Unknown distribution type");
            }

            samples.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

            let cdf_min_int: Vec<f64> = interp_slice(&samples, &cdf_min, &q, &InterpMode::default());
            let cdf_max_int: Vec<f64> = interp_slice(&samples, &cdf_max, &q, &InterpMode::default());

            // Fit curves
            let mut opt = Nlopt::new(Algorithm::Neldermead, 2,
                                     mse_cost, Minimize, (&q, &cdf_min_int, &kind));

            opt.set_lower_bounds(&[bounds[0][0]*0.7, bounds[1][0]*0.7]).unwrap();
            opt.set_upper_bounds(&[bounds[0][1]*1.3, bounds[1][1]*1.3]).unwrap();

            opt.set_maxeval(10000).unwrap();
            opt.set_xtol_abs1(1e-20).unwrap();
            let mut params_min = init_guess.clone();
            let stat_min = opt.optimize(&mut params_min)
                .expect("Fitting failed");

            let mut opt = Nlopt::new(Algorithm::Neldermead, 2,
                                     mse_cost, Minimize, (&q, &cdf_max_int, &kind));

            opt.set_lower_bounds(&[bounds[0][0]*0.7, bounds[1][0]*0.7]).unwrap();
            opt.set_upper_bounds(&[bounds[0][1]*1.3, bounds[1][1]*1.3]).unwrap();

            opt.set_maxeval(10000).unwrap();
            opt.set_xtol_abs1(1e-20).unwrap();
            let mut params_max = init_guess.clone();
            let stat_max = opt.optimize(&mut params_max)
                .expect("Fitting failed");

            *x_i = [params_min[0], params_min[1], params_max[0], params_max[1]];
            *opt_stat_i = [stat_min, stat_max];
        });
    (x, sampling_params)
}

pub fn cdf_fitting(data: Vec<f64>, cdf_min: Vec<f64>, cdf_max: Vec<f64>,
                   alpha_bounds: [f64; 2], beta_bounds: [f64; 2],
                   init_params: [f64;2], interp_domain: Vec<f64>, kind: DistributionType) -> [f64;4]{
    let mut samples: Vec<f64> = Vec::with_capacity(data.len()+2);
    samples.push(0.0);
    samples.extend(data);
    samples.push(1.0);
    samples.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    let cdf_min_int: Vec<f64> = interp_slice(&samples, &cdf_min,
                                             &interp_domain, &InterpMode::default());
    let cdf_max_int: Vec<f64> = interp_slice(&samples, &cdf_max,
                                             &interp_domain, &InterpMode::default());

    let mut opt = Nlopt::new(Algorithm::Neldermead, 2,
                             mse_cost, Minimize, (&interp_domain, &cdf_min_int, &kind));
    opt.set_lower_bounds(&[alpha_bounds[0]*0.7, beta_bounds[0]*0.7]).unwrap();
    opt.set_upper_bounds(&[alpha_bounds[1]*1.3, beta_bounds[1]*1.3]).unwrap();
    opt.set_maxeval(10000).unwrap();
    opt.set_xtol_abs1(1e-20).unwrap();
    let mut params_min = init_params.clone();
    let _stat_min = opt.optimize(&mut params_min)
        .expect("Fitting failed");

    let mut opt = Nlopt::new(Algorithm::Neldermead, 2,
                             mse_cost, Minimize, (&interp_domain, &cdf_max_int, &kind));
    opt.set_lower_bounds(&[alpha_bounds[0]*0.7, beta_bounds[0]*0.7]).unwrap();
    opt.set_upper_bounds(&[alpha_bounds[1]*1.3, beta_bounds[1]*1.3]).unwrap();
    opt.set_maxeval(10000).unwrap();
    opt.set_xtol_abs1(1e-20).unwrap();
    let mut params_max = [1.0f64, 1.0f64];
    let _stat_max = opt.optimize(&mut params_max)
        .expect("Fitting failed");

    [params_min[0], params_min[1], params_max[0], params_max[1]]
}

pub fn beta_cdf(domain: Vec<f64>, alpha: f64, beta: f64) -> Vec<f64> {
    // Generate a beta distribution with the given parameters
    // and calculate the CDF for the given domain
    domain.iter().map(|&x| {
        let dist = Beta::new(alpha, beta)
            .expect(format!("Invalid Beta distribution parameters: alpha={}, beta={}", alpha, beta).as_str());
        dist.cdf(x)
    }).collect()
}

pub fn beta_pdf(domain: Vec<f64>, alpha: f64, beta: f64) -> Vec<f64> {
    // Generate a beta distribution with the given parameters
    // and calculate the PDF for the given domain
    domain.iter().map(|&x| {
        let dist = Beta::new(alpha, beta)
            .expect("Invalid Beta distribution parameters");
        dist.pdf(x)
    }).collect()
}

pub fn generate_beta_random_numbers(n: usize, alpha: f64, beta: f64) -> Vec<f64> {
    let beta = RandBeta::new(alpha, beta).unwrap();
    let mut rng = rng();
    (0..n).map(|_| beta.sample(&mut rng)).collect()
}

pub fn frequencies(bins: &Vec<f64>, data: &Vec<f64>) -> Vec<f64> {
    assert!(bins.len() > 1, "Bins must have at least two elements");
    let mut freq = vec![0.0; bins.len() - 1];
    for &value in data {
        if let Some(pos) = bins.iter().position(|&bin| bin > value) {
            if pos > 0 {
                freq[pos - 1] += 1.0;
            }
        } else if (value - bins[bins.len() - 1]).abs() < f64::EPSILON {
            freq[bins.len() - 2] += 1.0;
        }
    }
    freq
}

fn chi2_stat(observed: &Vec<f64>, expected: &Vec<f64>) -> f64 {
    assert_eq!(observed.len(), expected.len(), "Observed and expected vectors must have the same length");
    observed.iter()
        .zip(expected.iter())
        .map(|(obs, exp)| 
            if *exp <= 0.0 { 
                panic!("Expected value is zero or negative: {}", exp)
            } else {
                (obs - exp).powi(2) / exp
            })
        .sum()
}

pub fn chi2_test(observed: &Vec<f64>, expected: &Vec<f64>, significance: f64) -> 
                (f64, f64, f64, bool) {
    let chi2 = chi2_stat(observed, expected);
    let dof = observed.len() as f64 - 1.0;
    let chi2_dist = ChiSquared::new(dof)
        .expect("Invalid degrees of freedom for Chi-squared distribution");
    let p_value = 1.0 - chi2_dist.cdf(chi2);
    let crit_value = chi2_dist.inverse_cdf(1.0 - significance);
    let decision: bool;
    if chi2 > crit_value {
        decision = false;
    } else {
        decision = true;
    }
    (chi2, crit_value, p_value, decision)
}

pub fn expected_freq_beta(alpha: f64, beta: f64, bins: &Vec<f64>, sample_size: usize) -> Vec<f64> {

    let dist = Beta::new(alpha, beta)
        .expect("Invalid Beta distribution parameters");

    let mut expected_freq = vec![0.0; bins.len() - 1];
    for i in 0..(bins.len() - 1) {
        let lower_bound = bins[i];
        let upper_bound = bins[i + 1];
        expected_freq[i] = (dist.cdf(upper_bound) - dist.cdf(lower_bound)) * sample_size as f64;
    }
    expected_freq
}

pub fn param_bounds(kind: DistributionType) -> [[f64; 2]; 2] {
    if kind == DistributionType::Beta {
        return [[0.1, 10.0], [0.1, 10.0]];
    }
    else if kind == DistributionType::Normal {
        return [[1e-3, 1.0-1e-3], [1e-3, 1.0/6.0]];
    }
    panic!("param_bounds: Unknown distribution type");
}

pub fn init_guess(kind: DistributionType) -> [f64; 2] {
    if kind == DistributionType::Beta {
        return [0.1, 0.1];
    }
    else if kind == DistributionType::Normal {
        return [0.5, 1.0/6.0];
    }
    panic!("init_guess: Unknown distribution type");
}

pub fn dist_domain(kind: DistributionType) -> Vec<f64> {
    if kind == DistributionType::Beta {
        return generate_range([0.0, 1.0], 101);
    }
    else if kind == DistributionType::Normal {
        return generate_range([-0.5, 1.5], 201);
    }
    panic!("cdf_domain: Unknown distribution type");
}

pub fn split_data(ratio: f64, size: usize) -> (Vec<usize>, Vec<usize>) {
    let mut rng = rng();
    let mut indices: Vec<usize> = (0..size).collect();
    indices.shuffle(&mut rng);
    let split_index = (ratio * size as f64) as usize;
    let (train_indices, test_indices) = indices.split_at(split_index);
    assert_eq!(train_indices.len() + test_indices.len(), size,
        "train_indices.len() + test_indices.len() != size");
    (train_indices.to_vec(), test_indices.to_vec())
}

pub fn calculate_rmse(y_test: &Vec<[f64; 2]>, y_pred: &Vec<[f64; 2]>) -> [f64; 2] {
    assert_eq!(y_test.len(), y_pred.len(), "y_test and y_pred must have the same length");
    let n = y_test.len();
    let mut rmse = [0.0, 0.0];
    for i in 0 .. n {
        rmse[0] += (y_test[i][0] - y_pred[i][0]).powi(2);
        rmse[1] += (y_test[i][1] - y_pred[i][1]).powi(2);
    }
    rmse[0] = (rmse[0] / n as f64).sqrt();
    rmse[1] = (rmse[1] / n as f64).sqrt();
    rmse
}

pub fn calculate_mae(y_test: &Vec<[f64; 2]>, y_pred: &Vec<[f64; 2]>) -> [f64; 2] {
    assert_eq!(y_test.len(), y_pred.len(), "y_test and y_pred must have the same length");
    let n = y_test.len();
    let mut mae = [0.0, 0.0];
    for i in 0 .. n {
        mae[0] += (y_test[i][0] - y_pred[i][0]).abs();
        mae[1] += (y_test[i][1] - y_pred[i][1]).abs();
    }
    mae[0] = mae[0] / n as f64;
    mae[1] = mae[1] / n as f64;
    mae
}
