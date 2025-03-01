use ndarray::{Array1, Array2};
use rand::rng;
use rand_distr::{Beta as RandBeta, Distribution};
use statrs::distribution::{Beta, ContinuousCDF};
//use ndarray_interp::interp1d::{Interp1DBuilder, Linear};
use argmin::solver::neldermead::NelderMead;
use std::time::Instant;

fn log_factorial(n: u64) -> f64 {
    (1..=n).fold(0.0, |acc, x| acc + (x as f64).ln())
}

pub fn hypergeometric_pmf(n_total: u64, k_total: u64, n: u64, k: u64) -> f64 {
    if k > k_total || k > n || n > n_total || n - k > n_total - k_total {
        eprintln!(
            "Out-of-bounds case in hypergeometric PMF: N {}, K {}, n {}, k {}",
            n_total, k_total, n, k
        );
        return 0.0;
    }
    let log_comb = |a: u64, b: u64| log_factorial(a) - log_factorial(b) - log_factorial(a - b);
    let log_pmf = log_comb(k_total, k) + log_comb(n_total - k_total, n - k) - log_comb(n_total, n);
    log_pmf.exp()
}

fn quality(n_total: u64, n: u64, k: u64, p_threshold_factor: u64) -> (f32, f32) {
    let k_total: Vec<_> = (k..=n_total - n + k).collect();

    let p: Vec<_> = k_total
        .iter()
        .map(|&k_total| hypergeometric_pmf(n_total, k_total, n, k))
        .collect();

    let p_threshold = p
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .copied()
        .unwrap_or(0.0)
        / p_threshold_factor as f64;

    let index_min = p.iter().position(|&x| x >= p_threshold).unwrap_or(0);

    let quality_min = k_total[index_min] as f64 / n_total as f64;

    let index_max = p
        .iter()
        .rposition(|&x| x >= p_threshold)
        .unwrap_or(p.len() - 1);

    let quality_max = k_total[index_max] as f64 / n_total as f64;

    (quality_min as f32, quality_max as f32)
}

fn main() {
    let start = Instant::now();

    let (n_total, n) = (1000_usize, 5_usize); // Batch and sampling sizes
    let alpha_range: Array1<f32> = Array1::linspace(1.0, 5.0, 2);
    let beta_range: Array1<f32> = Array1::linspace(1.0, 5.0, 2);
    let n_0 = 3; // Number of examples for each pair (a,b)

    // Interpolation domain for CDF
    let q: Array1<f32> = Array1::linspace(0.0, 1.0, 101);
    
    // Vector of conf intervals [Kmin,Kmax]
    let mut f: Array2<f32> = Array2::zeros((2, n+2));
    f[(0,0)] = 1.0;
    f[(1,0)] = 1.0;
    for (i,k) in (1..=n).rev().enumerate() {
        (f[(0,i+1)], f[(1,i+1)])
            = quality(n_total as u64, n as u64, k as u64, 10);
    }

//     let data = array![
//     [0.0, 2.0],
//     [0.5, 2.5],
//     [1.0, 3.0],
// ];
    //println!("{:?}", data);
    //println!("{:?}", f);
    
    // Array for generated samples
        let mut samples: Vec<f32> = Vec::with_capacity(n+2);
        samples.resize(n+2, 0.0);
        samples[n+1] = 1.0;

    // Buffer for interpolated data
    let mut buf: Array2<f32> = Array2::zeros((n+2, 2));

    let iter_num = alpha_range.len() * beta_range.len() * n_0;
    let mut y: Array2<f32> = Array2::zeros((iter_num, 2));
    let mut x: Array2<f32> = Array2::zeros((iter_num, 4));
    
    let mut rng = rng();
    let mut i = 0;
    for alpha in alpha_range {
        for beta in beta_range.clone() {
            let beta_dist = RandBeta::new(alpha as f64, beta as f64).unwrap();
            for _ in 0..n_0 {
                y[(i,0)] = alpha;
                y[(i,1)] = beta;
                for i in 1..=n {
                    samples[i] = beta_dist.sample(&mut rng) as f32;
                }
                samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

                //let x1 = interp_slice(&samples, &fmin, &q, &InterpMode::default());
                //let x2 = interp_slice(&samples, &fmax, &q, &InterpMode::default());

                // let interpolator = Interp1DBuilder::new(f)
                //     .x(samples)
                //     .strategy(Linear::new())
                //     .build().unwrap();
                
                // fit curve x1(q)
                
                

                i += 1;
            }
        }
    }

    
    
    println!("Elapsed time: {:.3} s", start.elapsed().as_secs_f64());
    
}
