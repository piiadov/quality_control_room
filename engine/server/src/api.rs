use models::wrapper::*;
use models::train::*;
use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "/home/vp/GitHub/quality_control_room/data/config.json";

#[derive(Deserialize, Debug)]
pub struct ApiRequest {
    pub test_mode: bool,
    pub command: String,
    pub data: Option<Vec<f64>>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub population_size: Option<usize>,
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
    beta_params_min: [f64; 2],
    beta_params_max: [f64; 2],
    predicted_beta_params: [f64; 2],
    predicted_cdf: Vec<f64>,
    predicted_pdf: Vec<f64>,
    test_mode_beta_params: [f64; 2],
    test_mode_cdf: Vec<f64>,
    test_mode_pdf: Vec<f64>,
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
            beta_params_min: [0.0;2],
            beta_params_max: [0.0;2],
            predicted_beta_params: [0.0;2],
            predicted_cdf: vec![],
            predicted_pdf: vec![],
            test_mode_beta_params: [0.0;2],
            test_mode_cdf: vec![],
            test_mode_pdf: vec![],
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

pub fn handle_calc(test_mode: bool, mut data: Vec<f64>, mut min_value: f64, mut max_value: f64, mut population_size: usize) -> Response {
    let mut response = Response::default();
    response.command = "Calc using Beta-distribution".to_string();

    // Interpolation domain
    let q = generate_range([0.0, 1.0], 101);
    response.q = q.clone();

    let config = read_config(CONFIG_PATH.to_string());
    let folder_path = config.paths.data_folder;

    if test_mode {
        let (a, b) = (3.0, 3.0);
        population_size = 3000;
        data = generate_beta_random_numbers(100, a, b);
        min_value = 0.0;
        max_value = 1.0;
        response.test_mode_beta_params = [a, b];
        response.test_mode_cdf = beta_cdf(q.clone(), a, b);
        response.test_mode_pdf = beta_pdf(q.clone(), a, b);
    }

    response.population_size = population_size;
    response.min_value = min_value;
    response.max_value = max_value;
    response.data = data.clone();

    let fn_inference = format!("{}/xgb_{}.json", folder_path, data.len());

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

    // Check if inference exists
    if !std::path::Path::new(&fn_inference).exists() {
        response.info = "Inference file not found".to_string();
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

    // Check if scaled data is in [0, 1]
    if scaled_data.iter().any(|&x| x < 0.0 || x > 1.0) {
        response.info = "Scaled data is out of bounds [0, 1]".to_string();
        return response;
    }
    let sample_size = scaled_data.len();
    response.scaled_data = scaled_data.clone();

    // Get confidence intervals for CDF
    let (cdf_min, cdf_max) = conf_int(population_size, sample_size);
    response.cdf_min = cdf_min.clone();
    response.cdf_max = cdf_max.clone();

    // Get fitted min/max beta-distributions
    let init_params = [0.1, 0.1];
    let alpha_bounds = [0.1, 10.0];
    let beta_bounds = [0.1, 10.0];
    let fitted_params = cdf_fitting(scaled_data.clone(), cdf_min, cdf_max, alpha_bounds, beta_bounds, 
                init_params, q.clone());
    response.beta_params_min = [fitted_params[0], fitted_params[1]];
    response.beta_params_max = [fitted_params[2], fitted_params[3]];

    // Get fitted CDF and PDF
    response.fitted_cdf_min = beta_cdf(q.clone(), fitted_params[0], fitted_params[1]);
    response.fitted_cdf_max = beta_cdf(q.clone(), fitted_params[2], fitted_params[3]);
    response.fitted_pdf_min = beta_pdf(q.clone(), fitted_params[0], fitted_params[1]);
    response.fitted_pdf_max = beta_pdf(q.clone(), fitted_params[2], fitted_params[3]);
    
    // Get predicted beta-distribution parameters
    let x: Vec<f32> = fitted_params.iter().map(|&x| x as f32).collect();
    let pred = xgb_predict(x, 1, 4, 2,
        fn_inference);
    response.predicted_beta_params = [pred[0] as f64, pred[1] as f64];

    // Get predicted CDF and PDF
    response.predicted_cdf = beta_cdf(q.clone(), pred[0] as f64, pred[1] as f64);
    response.predicted_pdf = beta_pdf(q.clone(), pred[0] as f64, pred[1] as f64);
    if test_mode {
        response.info = "Test mode".to_string();
    }
    response.error = 0;
    // println!("beta_params_min: {:?}", response.beta_params_min);
    // println!("beta_params_max: {:?}", response.beta_params_max);
    // println!("predicted_beta_params: {:?}", response.predicted_beta_params);
    response
}
