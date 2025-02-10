use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ApiRequest {
    pub command: String,
    pub data: Option<Vec<f64>>, // Optional for non-calc commands
}

#[derive(Serialize, Debug)]
pub struct Response {
    command: String,
    info: String,
    pub x: Vec<f64>,
    pub q: Vec<Vec<f64>>,
}

pub fn handle_about() -> Response {
    Response {
        command: "About".to_string(),
        info: "Quality analysis engine v1.1".to_string(),
        x: vec![],
        q: vec![]
    }
}

pub fn handle_calc(data: Vec<f64>) -> Response {
    let n_total = data.len() as u64 * 50; // n_total >> n for small sampling
    let (x, q) = quant_quality(n_total, data);
    Response {
        command: "Calc".to_string(),
        info: "Calculation OK".to_string(),
        x,
        q
    }
}

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

pub fn quality(n_total: u64, n: u64, k: u64, p_threshold_factor: u64) -> Vec<f64> {
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

    vec![quality_min, quality_max]
}

pub fn quant_quality(n_total: u64, data: Vec<f64>) -> (Vec<f64>, Vec<Vec<f64>>) {
    let mut data_range: Vec<f64> = data.iter().copied().collect();
    data_range.sort_by(|a, b| a.partial_cmp(b).unwrap());
    data_range.dedup();

    let mut k: Vec<_> = data_range
        .iter()
        .map(|&x| data.iter().filter(|&&y| y >= x).count())
        .collect();
    k.push(0);

    (
        data_range,
        k.iter()
            .map(|&x| quality(n_total, data.len() as u64, x as u64, 10))
            .collect(),
    )
}
