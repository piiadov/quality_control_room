use ndarray::{Array1, Array2};
use statrs::distribution::{Discrete, Hypergeometric, Beta, ContinuousCDF};
use statrs::statistics::Statistics;
use std::time::Instant;
use rand::rng;
use rand_distr::{Beta as RandBeta, Distribution};
use interp::{interp_slice, InterpMode};
use rayon::prelude::*;
use nlopt::{Algorithm, Nlopt, SuccessState};
use nlopt::SuccessState::Success;
use nlopt::Target::Minimize;

fn quality_interval(population_size: u64, sample_size: u64,
                    sample_successes: u64, prob_threshold_factor: f64) -> (f64, f64) {
    let prob: Array1<f64> = (sample_successes
        ..= population_size - sample_size + sample_successes)
        .collect::<Array1<u64>>()
        .map(|population_successes|
            Hypergeometric::new(population_size, *population_successes, sample_size)
                .expect("Invalid parameters for hypergeometric distribution")
                .pmf(sample_successes)
        );

    let prob_threshold = Statistics::max(&prob) / prob_threshold_factor;

    let index_min = prob.iter().position(|&x| x >= prob_threshold)
        .unwrap_or(0);
    let index_max = prob.iter().rposition(|&x| x >= prob_threshold)
        .unwrap_or(prob.len() - 1);

    let quality_min = (index_min as u64 + sample_successes) as f64 / population_size as f64;
    let quality_max = (index_max as u64 + sample_successes) as f64 / population_size as f64;

    (quality_min, quality_max)
}

fn mse_cost(x: &[f64], _grad: Option<&mut [f64]>, user_data: &mut (&Vec<f64>, &Vec<f64>)) -> f64 {
    if x[0] <= 0.0 || x[1] <= 0.0 {
        return 1e10;
    }
    let beta_dist = match Beta::new(x[0], x[1]) {
        Ok(b) => b,
        Err(_) => return 1e10,
    };
    let mut sse = 0.0;
    let points_num = user_data.0.len();
    for i in 0 .. points_num {
        let pred = 1.0 - beta_dist.cdf(user_data.0[i]);
        sse += (pred - user_data.1[i]).powi(2);
    }
    sse / points_num as f64
}

fn main() {
    let start = Instant::now();

    let (population_size, sample_size): (usize, usize) = (1000, 50); // Batch and sampling sizes

    // Interpolation domain for CDF
    let q = Array1::linspace(0.0, 1.0, 101).to_vec();

    // Vector of conf intervals [quality_min, quality_max]
    let mut cdf_min: Vec<f64> = Vec::with_capacity(sample_size+2);
    cdf_min.resize(sample_size+2, 0.0);
    cdf_min[0] = 1.0;
    cdf_min[sample_size+1] = 0.0;
    let mut cdf_max: Vec<f64> = Vec::with_capacity(sample_size+2);
    cdf_max.resize(sample_size+2, 0.0);
    cdf_max[0] = 1.0;
    cdf_max[sample_size+1] = 0.0;

    for (i,k) in (1..=sample_size).rev().enumerate() {
        (cdf_min[i+1], cdf_max[i+1])
            = quality_interval(population_size as u64, sample_size as u64, k as u64, 10.0);
    }

    let alpha_bounds = [1.0, 5.0];
    let beta_bounds = [1.0, 5.0];
    let alpha_range: Array1<f64> = Array1::linspace(alpha_bounds[0], alpha_bounds[1], 10);
    let beta_range: Array1<f64> = Array1::linspace(beta_bounds[0], beta_bounds[1], 10);
    let dist_train_size: usize = 100; // Number of examples for each pair (a,b)
    let iter_num = alpha_range.len() * beta_range.len() * dist_train_size;

    // How much memory we need
    println!("Elements num: {}", iter_num);
    let dist_mem = iter_num * size_of::<RandBeta<f64>>() + size_of::<Vec<RandBeta<f64>>>();
    println!("Memory for Beta-distributions: {} MB", dist_mem/1024/1024);
    let y_mem = 2 * iter_num * size_of::<f64>() + size_of::<Array2<f64>>();
    println!("Memory for y: {} MB", y_mem/1024/1024);

    // Allocate main data structures
    let mut dist: Vec<RandBeta<f64>> = Vec::with_capacity(iter_num);
    let mut y: Vec<[f64; 2]> = Vec::with_capacity(iter_num);
    y.resize(iter_num, [0.0, 0.0]);
    let mut x: Vec<[f64; 4]> = Vec::with_capacity(iter_num);
    x.resize(iter_num, [0.0, 0.0, 0.0, 0.0]);
    let mut opt_stat: Vec<[(SuccessState, f64); 2]> = Vec::with_capacity(iter_num);
    opt_stat.resize(iter_num, [(Success,0.0), (Success,0.0)]);

    let mut j = 0;
    (0..alpha_range.len() * beta_range.len()).for_each(|i|{
        let alpha = alpha_range[i / beta_range.len()];
        let beta = beta_range[i % beta_range.len()];
        (0 .. dist_train_size).for_each(|_|{
            y[j][0] = alpha;
            y[j][1] = beta;
            dist.push(RandBeta::new(alpha, beta).expect("Invalid Beta distribution parameters"));
            j += 1;
        });
    });

    dist
        .par_iter()
        .zip(x.par_iter_mut())
        .zip(opt_stat.par_iter_mut())
        .for_each(|((beta_dist, x_i), opt_stat_i)| {
            let mut rng = rng();
            let mut samples: Vec<f64> = Vec::with_capacity(sample_size+2);
            samples.push(0.0);
            for _ in 1 .. sample_size + 1 {
                samples.push(beta_dist.sample(&mut rng));
            }
            samples.push(1.0);
            samples.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

            let cdf_min_int: Vec<f64> = interp_slice(&samples, &cdf_min, &q, &InterpMode::default());
            let cdf_max_int: Vec<f64> = interp_slice(&samples, &cdf_max, &q, &InterpMode::default());

            // Fit curves
            let mut opt = Nlopt::new(Algorithm::Neldermead, 2,
                                     mse_cost, Minimize, (&q, &cdf_min_int));
            opt.set_lower_bounds(&[alpha_bounds[0], beta_bounds[0]]).unwrap();
            opt.set_upper_bounds(&[alpha_bounds[1], beta_bounds[1]]).unwrap();
            opt.set_maxeval(1000).unwrap();
            opt.set_xtol_abs1(1e-20).unwrap();
            let mut params_min = [1.0f64, 1.0f64];
            let stat_min = opt.optimize(&mut params_min).expect("Optimization failed");

            let mut opt = Nlopt::new(Algorithm::Neldermead, 2,
                                     mse_cost, Minimize, (&q, &cdf_max_int));
            opt.set_lower_bounds(&[alpha_bounds[0], beta_bounds[0]]).unwrap();
            opt.set_upper_bounds(&[alpha_bounds[1], beta_bounds[1]]).unwrap();
            opt.set_maxeval(1000).unwrap();
            opt.set_xtol_abs1(1e-20).unwrap();
            let mut params_max = [1.0f64, 1.0f64];
            let stat_max = opt.optimize(&mut params_max).expect("Optimization failed");

            *x_i = [params_min[0], params_min[1], params_max[0], params_max[1]];
            *opt_stat_i = [stat_min, stat_max];
    });

    // let i = 567;
    // println!("{:?}", opt_stat[i]);
    // println!("x: {:?}", x[i]);
    // println!("y: {:?}", y[i]);

    // let (q1 ,q2) = quality_interval(300,15, 3, 10.0);
    // println!("{:.4} : {:.4}", q1, q2);

    // Split train/test, XGBoost, MSE test




    println!("Elapsed time: {:.3} s", start.elapsed().as_secs_f64());
}
