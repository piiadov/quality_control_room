use models::wrapper::*;
use models::train::*;
use serde::{Deserialize, Serialize};

const DATA_PATH: &str = "/home/vp/quality_control_room/data";

#[derive(Deserialize, Debug)]
pub struct ApiRequest {
    pub kind: u8,
    pub test_mode: bool,
    pub command: String,
    pub data: Option<Vec<f64>>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub population_size: Option<usize>,
    pub bins_number: Option<usize>,
    pub params_min: Option<[f64; 2]>,
    pub params_max: Option<[f64; 2]>,
    pub predicted_params: Option<[f64; 2]>,
    pub test_mode_params: Option<[f64; 2]>,
}

#[derive(Serialize, Debug)]
pub struct Response {
    command: String,
    info: String,
    error: u8,
    population_size: usize,
    min_value: f64,
    max_value: f64,
    data: Vec<f64>,
    scaled_data: Vec<f64>,
    cdf_min: Vec<f64>,
    cdf_max: Vec<f64>,
    q: Vec<f64>,
    fitted_cdf_min: Vec<f64>,
    fitted_cdf_max: Vec<f64>,
    fitted_pdf_min: Vec<f64>,
    fitted_pdf_max: Vec<f64>,
    params_min: [f64; 2],
    params_max: [f64; 2],
    predicted_params: [f64; 2],
    predicted_cdf: Vec<f64>,
    predicted_pdf: Vec<f64>,
    test_mode_params: [f64; 2],
    test_mode_cdf: Vec<f64>,
    test_mode_pdf: Vec<f64>,
    bins: Vec<f64>,
    freq: Vec<f64>,
    freq_min: Vec<f64>,
    freq_max: Vec<f64>,
    freq_pred: Vec<f64>,
    test_mode_freq: Vec<f64>,
    predicted_chi2: f64,
    min_chi2: f64,
    max_chi2: f64,
    test_mode_chi2: f64,
    predicted_pval: f64,
    min_pval: f64,
    max_pval: f64,
    test_mode_pval: f64,
    crit_val: f64,
    min_decision: bool,
    max_decision: bool,
    predicted_decision: bool,
    test_mode_decision: bool,
    sampling_mu: f64,
    sampling_sigma: f64,
    sampling_params: [f64; 2],
    sampling_cdf: Vec<f64>,
    sampling_pdf: Vec<f64>,
}

impl Default for Response {
    fn default() -> Self {
        Response {
            command: "".to_string(),
            info: "".to_string(),
            error: 1,
            population_size: 0,
            min_value: 0.0,
            max_value: 0.0,
            data: vec![],
            scaled_data: vec![],
            cdf_min: vec![],
            cdf_max: vec![],
            q: vec![],
            fitted_cdf_min: vec![],
            fitted_cdf_max: vec![],
            fitted_pdf_min: vec![],
            fitted_pdf_max: vec![],
            params_min: [0.0;2],
            params_max: [0.0;2],
            predicted_params: [0.0;2],
            predicted_cdf: vec![],
            predicted_pdf: vec![],
            test_mode_params: [0.0;2],
            test_mode_cdf: vec![],
            test_mode_pdf: vec![],
            bins: vec![],
            freq: vec![],
            freq_min: vec![],
            freq_max: vec![],
            freq_pred: vec![],
            test_mode_freq: vec![],
            predicted_chi2: 0.0,
            min_chi2: 0.0,
            max_chi2: 0.0,
            test_mode_chi2: 0.0,
            predicted_pval: 0.0,
            min_pval: 0.0,
            max_pval: 0.0,
            test_mode_pval: 0.0,
            crit_val: 0.0,
            min_decision: false,
            max_decision: false,
            predicted_decision: false,
            test_mode_decision: false,
            sampling_mu: 0.0,
            sampling_sigma: 0.0,
            sampling_params: [0.0;2],
            sampling_cdf: vec![],
            sampling_pdf: vec![],
        }
    }
}

pub fn handle_about() -> Response {
    let version = env!("CARGO_PKG_VERSION");
    let mut response = Response::default();
    response.command = "About".to_string();
    response.info = format!("Quality analysis engine v{}", version);
    response.error = 0;
    response
}

pub fn handle_calc(kind_uint: u8, test_mode: bool, mut data: Vec<f64>, mut min_value: f64,
                   mut max_value: f64, mut population_size: usize, bins_number: usize) -> Response {
    let mut response = Response::default();
    response.command = "Calc".to_string();

    let kind = match kind_uint {
        0 => DistributionType::Beta,
        1 => DistributionType::Normal,
        _ => {
            response.info = format!("Invalid distribution type {}", kind_uint);
            return response;
        },
    };

    // Interpolation domain
    let q = dist_domain(kind.clone());
    response.q = q.clone();

    let config = read_config(format!("{}/config.json", DATA_PATH));
    let inferences_folder = format!("{}/inferences", DATA_PATH);

    if test_mode {
        population_size = 3000;
        let params: [f64; 2];
        if kind == DistributionType::Beta {
            params = [5.0, 2.0];
            min_value = 0.0;
            max_value = 1.0;
        } else if kind == DistributionType::Normal {
            params = [0.5, 0.5/3.0];
            min_value = 0.0;
            max_value = 1.0;
        } else {
            response.info = format!("Impossible if-else branch for kind {}", kind);
            return response;
        }
        data = generate_random_data(kind.clone(), 100, params);
        response.test_mode_params = params;
        response.test_mode_cdf = cdf(kind.clone(), q.clone(), params)
            .into_iter().map(|x| 1.0 - x).collect();
        response.test_mode_pdf = pdf(kind.clone(), q.clone(), params);
        response.info = response.info + "Test mode\n";
    }

    response.population_size = population_size;
    response.min_value = min_value;
    response.max_value = max_value;
    response.data = data.clone();

    // Calculate sampling parameters
    let sampling_params = calculate_sampling_params(kind.clone(), data.clone());
    response.sampling_mu = sampling_params[0];
    response.sampling_sigma = sampling_params[1];
    response.sampling_params = [sampling_params[2], sampling_params[3]];
    response.sampling_cdf = cdf(kind.clone(), q.clone(), response.sampling_params)
        .into_iter().map(|x| 1.0 - x).collect();
    response.sampling_pdf = pdf(kind.clone(), q.clone(), response.sampling_params);

    // Select inference file by nearest suffix to data.len()
    let n_grid: Vec<i32> = (config.n_min..=config.n_max).step_by(config.n_step).collect();
    let n_inference: i32 = n_grid.iter().min_by_key(|&&x| (x - data.len() as i32).abs()).unwrap().clone();
    if n_inference > config.n_max {
        response.info = response.info + "Warn: Data size exceeds maximum inference range\n";
    } else if n_inference < config.n_min {
        response.info = response.info + "Warn: Data size is less than minimum inference range\n";
    }

    let inference_filename = format!("{}/xgb_{}_{}.json", 
        inferences_folder, kind.clone(), n_inference);

    if !std::path::Path::new(&inference_filename).exists() {
        response.info = "Inference file not found".to_string();
        return response;
    }

    // Check if population_size is valid
    if population_size == 0 {
        response.info = "population_size must be greater than 0".to_string();
        return response;
    }

    // Check if min_value and max_value are valid
    if min_value >= max_value {
        response.info = "min_value must be less than max_value".to_string();
        return response;
    }

    // Check if data is empty
    if data.is_empty() {
        response.info = "Data is empty".to_string();
        return response;
    }
    // Check if data is valid
    if data.iter().any(|&x| x.is_nan() || x.is_infinite()) {
        response.info = "Data contains NaN or infinite values".to_string();
        return response;
    }

    // Scale sampling data with min_value and max_value to [0, 1]
    let scaled_data: Vec<f64> = data.iter()
        .map(|&x| (x - min_value) / (max_value - min_value))
        .collect();

    let sample_size = scaled_data.len();
    let mut sorted_scaled_data = scaled_data.clone();
    sorted_scaled_data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    //sorted_scaled_data.reverse();
    response.scaled_data = sorted_scaled_data;

    // Get confidence intervals for CDF
    let (cdf_min, cdf_max) = conf_int(population_size, sample_size);
    response.cdf_min = cdf_min.clone();
    response.cdf_max = cdf_max.clone();

    // Get fitted min/max distributions
    let bounds = param_bounds(kind.clone());
    let init_guess = init_guess(kind.clone());

    let fitted_params = cdf_fitting(kind.clone(), scaled_data.clone(), cdf_min, cdf_max, 
            bounds, init_guess, q.clone());
    
    response.params_min = [fitted_params[0], fitted_params[1]];
    response.params_max = [fitted_params[2], fitted_params[3]];

    // Get fitted CDF and PDF
    response.fitted_cdf_min = cdf(kind.clone(), q.clone(), response.params_min)
        .into_iter().map(|x| 1.0 - x).collect();
    response.fitted_cdf_max = cdf(kind.clone(), q.clone(), response.params_max)
        .into_iter().map(|x| 1.0 - x).collect();
    response.fitted_pdf_min = pdf(kind.clone(), q.clone(), response.params_min);
    response.fitted_pdf_max = pdf(kind.clone(), q.clone(), response.params_max);
    
    // Get predicted distribution parameters
    let x: Vec<f32> = fitted_params.iter().map(|&x| x as f32).collect();
    let pred = xgb_predict(x, 1, 4, 2,
        inference_filename);
    response.predicted_params = [pred[0] as f64, pred[1] as f64];

    // Check if predicted beta-distribution parameters are valid
    if pred[0] <= 0.0 || pred[1] <= 0.0 {
        response.info = "Predicted beta-distribution parameters are invalid".to_string();
        return response;
    }

    // Get predicted CDF and PDF
    response.predicted_cdf = cdf(kind.clone(), q.clone(), response.predicted_params)
        .into_iter().map(|x| 1.0 - x).collect();
    response.predicted_pdf = pdf(kind.clone(), q.clone(), response.predicted_params);

    // Bins and sampling frequencies
    let bins = generate_range([q[0], q[q.len()-1]], bins_number + 1);
    response.bins = bins.clone();
    response.freq = frequencies(&bins, &scaled_data);

    response.freq_min = expected_freq(kind.clone(), response.params_min, &bins, scaled_data.len());
    response.freq_max = expected_freq(kind.clone(), response.params_max, &bins, scaled_data.len());
    response.freq_pred = expected_freq(kind.clone(), response.predicted_params, &bins, scaled_data.len());

    if test_mode {
        response.test_mode_freq = expected_freq(kind.clone(), response.test_mode_params, &bins, scaled_data.len());
    }

    // Chi2 tests
    (
        response.min_chi2, 
        response.crit_val, 
        response.min_pval, 
        response.min_decision
    ) = chi2_test(&response.freq, &response.freq_min, 0.05);

    (
        response.max_chi2, 
        _, 
        response.max_pval, 
        response.max_decision
    ) = chi2_test(&response.freq, &response.freq_max, 0.05);

    (
        response.predicted_chi2, 
        _, 
        response.predicted_pval, 
        response.predicted_decision
    ) = chi2_test(&response.freq, &response.freq_pred, 0.05);
    if test_mode {
        (
            response.test_mode_chi2, 
            _, 
            response.test_mode_pval, 
            response.test_mode_decision
        ) = chi2_test(&response.freq, &response.test_mode_freq, 0.05);
    }

    response.error = 0;
    response
}

pub fn handle_update_bins(kind_uint: u8, data: Vec<f64>, min_value: f64, max_value: f64, bins_number: usize,
                          params_min: [f64; 2], params_max: [f64; 2],
                          predicted_params: [f64; 2], test_mode_params: [f64; 2],
                          test_mode: bool) -> Response {
    let mut response = Response::default();
    response.command = "Update".to_string();

     let kind = match kind_uint {
        0 => DistributionType::Beta,
        1 => DistributionType::Normal,
        _ => {
            response.info = format!("Invalid distribution type {}", kind_uint);
            return response;
        },
    };

    response.min_value = min_value;
    response.max_value = max_value;
    response.data = data.clone();
    
    // Check if data is empty
    if data.is_empty() {
        response.info = "Data is empty".to_string();
        return response;
    }
    // Check if data is valid
    if data.iter().any(|&x| x.is_nan()) {
        response.info = "Data contains NaN".to_string();
        return response;
    }

    // Check if min_value and max_value are valid
    if min_value >= max_value {
        response.info = "min_value must be less than max_value".to_string();
        return response;
    }

    // Scaling data with min_value and max_value to [0, 1]
    let scaled_data: Vec<f64> = data.iter()
        .map(|&x| (x - min_value) / (max_value - min_value))
        .collect();

    // Check bins_number
    if bins_number < 1 || bins_number > 50 {
        response.info = "bins_number must be in [0, 50]".to_string();
        return response;
    }

    // Check beta params
    if params_min.iter().any(|&x| x.is_nan() || (x <= 0.0 && kind == DistributionType::Beta)) {
        response.info = "params_min contains NaN or 0.0 (if Beta)".to_string();
        return response;
    }
    if params_max.iter().any(|&x| x.is_nan() || (x <= 0.0 && kind == DistributionType::Beta)) {
        response.info = "beta_params_max contains NaN or 0.0 (if Beta)".to_string();
        return response;
    }
    if predicted_params.iter().any(|&x| x.is_nan() || (x <= 0.0 && kind == DistributionType::Beta)) {
        response.info = "predicted_beta_params contains NaN or 0.0 (if Beta)".to_string();
        return response;
    }
    if test_mode {
        if test_mode_params.iter().any(|&x| x.is_nan() || (x <= 0.0 && kind == DistributionType::Beta)) {
            response.info = "test_mode_beta_params contains NaN or 0.0 (if Beta)".to_string();
            return response;
        }
    }

    // Bins and sampling frequencies
    let q = dist_domain(kind.clone());
    let bins = generate_range([q[0], q[q.len()-1]], bins_number + 1);

    response.bins = bins.clone();
    response.freq = frequencies(&bins, &scaled_data);

    response.freq_min = expected_freq(kind.clone(), params_min, &bins, scaled_data.len());
    response.freq_max = expected_freq(kind.clone(), params_max, &bins, scaled_data.len());
    response.freq_pred = expected_freq(kind.clone(), predicted_params, &bins, scaled_data.len());

    if test_mode {
        response.test_mode_freq = expected_freq(kind.clone(), test_mode_params, &bins, scaled_data.len());
    }

    // Chi2 tests
    (
        response.min_chi2, 
        response.crit_val, 
        response.min_pval, 
        response.min_decision
    ) = chi2_test(&response.freq, &response.freq_min, 0.05);

    (
        response.max_chi2, 
        _, 
        response.max_pval, 
        response.max_decision
    ) = chi2_test(&response.freq, &response.freq_max, 0.05);

    (
        response.predicted_chi2, 
        _, 
        response.predicted_pval, 
        response.predicted_decision
    ) = chi2_test(&response.freq, &response.freq_pred, 0.05);
    if test_mode {
        (
            response.test_mode_chi2, 
            _, 
            response.test_mode_pval, 
            response.test_mode_decision
        ) = chi2_test(&response.freq, &response.test_mode_freq, 0.05);
    }

    response.error = 0;
    response
}
