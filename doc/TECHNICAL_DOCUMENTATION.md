# Quality Control Room - Technical Documentation

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Core Algorithms](#core-algorithms)
   - [Confidence Interval Calculation](#1-confidence-interval-calculation-via-hypergeometric-distribution)
   - [CDF Fitting via Nelder-Mead](#2-cdf-fitting-via-nelder-mead-optimization)
   - [XGBoost Prediction Pipeline](#3-xgboost-prediction-pipeline)
   - [Chi-Square Goodness-of-Fit Testing](#4-chi-square-goodness-of-fit-testing)
   - [Method of Moments Estimation](#5-sampling-parameter-estimation-method-of-moments)
4. [Data Flow](#data-flow-complete-request-processing)
5. [XGBoost Wrapper (C Library)](#xgboost-wrapper-c-library)
6. [Frontend (Vue.js)](#frontend-vuejs)
7. [Configuration](#configuration)
8. [Build System](#build-system)
9. [Supported Distributions](#supported-distributions)
10. [Test Suite](#test-suite)

---

## Overview

**Quality Control Room** is a hybrid statistical and machine learning system designed for industrial quality control analysis. The application addresses a fundamental problem in quality control: **predicting population distribution parameters from small sample sizes**.

### Problem Statement

In manufacturing and quality control scenarios, obtaining large samples is often expensive or impractical. Given a small sample (e.g., 5-100 observations) from a larger population (e.g., 3000 items), the system estimates the underlying distribution parameters with higher accuracy than traditional statistical methods.

### Solution Approach

The system combines:
1. **Classical statistics**: Hypergeometric-based confidence intervals for the empirical CDF
2. **Numerical optimization**: Nelder-Mead algorithm for fitting theoretical distributions to confidence bounds
3. **Machine learning**: XGBoost regression models trained to predict optimal parameters from fitted features

This hybrid approach leverages the interpretability of statistical methods with the predictive power of gradient boosting.

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Vue.js Frontend (UI)                              │
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │ Data Input  │  │ CDF Chart   │  │ PDF Chart   │  │ Histogram   │       │
│  │   Form      │  │ Comparison  │  │ Comparison  │  │   Chart     │       │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                                             │
│                      WebSocket Communication Layer                          │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Rust Engine (Warp Server)                            │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                       API Request Handler                              │ │
│  │              handle_calc() │ handle_update_bins()                     │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                      │                                      │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                         Models Library                                 │ │
│  │                                                                        │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐       │ │
│  │  │   Confidence    │  │   CDF Fitting   │  │   Chi-Square    │       │ │
│  │  │   Intervals     │  │  (Nelder-Mead)  │  │    Testing      │       │ │
│  │  │ (Hypergeometric)│  │                 │  │                 │       │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘       │ │
│  │                                                                        │ │
│  │  ┌─────────────────┐  ┌─────────────────┐                            │ │
│  │  │  Distribution   │  │    Frequency    │                            │ │
│  │  │   Parameter     │  │    Analysis     │                            │ │
│  │  │   Estimation    │  │                 │                            │ │
│  │  └─────────────────┘  └─────────────────┘                            │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       XGBoost Wrapper (C Library)                           │
│                                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │    Training     │  │   Prediction    │  │ Data Splitting  │            │
│  │   (Iterative)   │  │   (Inference)   │  │  & Shuffling    │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│                                                                             │
│                         Links to: libxgboost.so                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Component Summary

| Component | Language | Purpose |
|-----------|----------|---------|
| **Engine** | Rust | Core computation, API server, statistical algorithms |
| **XGBoost Wrapper** | C | Training and inference interface to XGBoost library |
| **Frontend** | Vue.js 3 | Interactive data visualization and user interface |

---

## Core Algorithms

### 1. Confidence Interval Calculation via Hypergeometric Distribution

**Source File:** `engine/models/src/train.rs` - `conf_int()` function

#### Theoretical Background

When sampling without replacement from a finite population, the number of "successes" follows a **hypergeometric distribution**. This is the appropriate model for quality control where we're sampling items from a finite batch.

#### Mathematical Definition

For a population of size $N$ containing $K$ success items, if we draw a sample of size $n$ without replacement, the probability of observing exactly $k$ successes is:

$$P(X = k) = \frac{\binom{K}{k} \binom{N-K}{n-k}}{\binom{N}{n}}$$

Where:
- $N$ = total population size
- $K$ = number of success items in population  
- $n$ = sample size drawn
- $k$ = number of successes observed in sample
- $\binom{a}{b} = \frac{a!}{b!(a-b)!}$ = binomial coefficient

#### Computational Implementation

For numerical stability with large factorials, the PMF is computed in log-space:

$$\log P(X = k) = \log\binom{K}{k} + \log\binom{N-K}{n-k} - \log\binom{N}{n}$$

Where the log of the binomial coefficient is:

$$\log\binom{a}{b} = \sum_{i=1}^{a}\log(i) - \sum_{i=1}^{b}\log(i) - \sum_{i=1}^{a-b}\log(i)$$

```rust
fn hypergeometric_pmf(n_total: u64, k_total: u64, n: u64, k: u64) -> f64 {
    fn log_factorial(n: u64) -> f64 {
        (1..=n).fold(0.0, |acc, x| acc + (x as f64).ln())
    }
    
    fn log_binomial(n: u64, k: u64) -> f64 {
        log_factorial(n) - log_factorial(k) - log_factorial(n - k)
    }
    
    let log_pmf = log_binomial(k_total, k) 
                + log_binomial(n_total - k_total, n - k) 
                - log_binomial(n_total, n);
    
    log_pmf.exp()
}
```

#### Algorithm: Confidence Interval Computation

**Input:**
- $N$ = population size
- $n$ = sample size  
- $\alpha$ = significance level (default: 0.05)
- $m$ = number of quantile points (default: 101)

**Output:**
- `q_points`: array of quantile values $[q_1, q_2, ..., q_m]$ where $q_i \in [0, 1]$
- `cdf_min`: lower confidence bounds for each quantile
- `cdf_max`: upper confidence bounds for each quantile

**Procedure:**

```
Algorithm: ConfidenceInterval(N, n, α, m)
────────────────────────────────────────────────────────────────
1.  Initialize q_points = linspace(0, 1, m)
2.  Initialize cdf_min[m], cdf_max[m]
3.  
4.  FOR i = 0 TO m-1:
5.      q ← q_points[i]
6.      K ← floor(q × N)                    // Success count in population
7.      
8.      // Compute PMF for all possible k values
9.      FOR k = 0 TO n:
10.         pmf[k] ← HypergeometricPMF(N, K, n, k)
11.     END FOR
12.     
13.     // Find lower bound (smallest k where CDF ≥ α/2)
14.     cumsum ← 0
15.     FOR k = 0 TO n:
16.         cumsum ← cumsum + pmf[k]
17.         IF cumsum ≥ α/2:
18.             k_min ← k
19.             BREAK
20.     END FOR
21.     
22.     // Find upper bound (largest k where CDF ≤ 1 - α/2)
23.     cumsum ← 0
24.     FOR k = n TO 0 (descending):
25.         cumsum ← cumsum + pmf[k]
26.         IF cumsum ≥ α/2:
27.             k_max ← k
28.             BREAK
29.     END FOR
30.     
31.     cdf_min[i] ← k_min / n
32.     cdf_max[i] ← k_max / n
33. END FOR
34. 
35. RETURN (q_points, cdf_min, cdf_max)
────────────────────────────────────────────────────────────────
```

#### Interpretation

The resulting `cdf_min` and `cdf_max` arrays define an envelope around the theoretical CDF. With $(1-\alpha) \times 100\%$ confidence, the true population CDF lies within this envelope at each quantile point.

```
    CDF
    1.0 ┤                                    ████████
        │                              ██████
        │                        ██████      ← cdf_max (upper bound)
        │                  ██████
    0.5 ┤            ██████
        │      ██████────────────────────────── True CDF (unknown)
        │██████
        │    ████████
        │          ████████                  ← cdf_min (lower bound)
    0.0 ┼────────────────────────────────────────────
        0                                    1.0
                        Quantile (q)
```

---

### 2. CDF Fitting via Nelder-Mead Optimization

**Source File:** `engine/models/src/train.rs` - `cdf_fitting()`, `features_prepare_nm()`

#### Purpose

Fit theoretical distribution CDFs to the confidence interval boundaries (`cdf_min` and `cdf_max`) to extract features for the ML model.

#### Nelder-Mead Simplex Algorithm

The **Nelder-Mead** algorithm is a derivative-free optimization method suitable for problems where gradients are unavailable or expensive to compute. It maintains a simplex of $n+1$ points in $n$-dimensional space and iteratively transforms it towards the optimum.

**Operations:**
1. **Reflection**: Move away from worst point
2. **Expansion**: If reflection is good, expand further
3. **Contraction**: If reflection is bad, contract towards centroid
4. **Shrink**: If all else fails, shrink simplex towards best point

#### Objective Function: Mean Squared Error

For distribution parameters $\theta = (\theta_1, \theta_2)$:

$$\text{MSE}(\theta) = \frac{1}{n} \sum_{i=1}^{n} \left( F_{\theta}(x_i) - \hat{F}(x_i) \right)^2$$

Where:
- $F_{\theta}(x)$ = theoretical CDF with parameters $\theta$
- $\hat{F}(x)$ = target CDF values (interpolated confidence bounds)
- $x_i$ = sorted sample data points

#### Distribution CDFs

**Beta Distribution:**

$$F_{\text{Beta}}(x; \alpha, \beta) = I_x(\alpha, \beta) = \frac{B(x; \alpha, \beta)}{B(\alpha, \beta)}$$

Where $I_x$ is the regularized incomplete beta function and $B$ is the beta function:

$$B(\alpha, \beta) = \frac{\Gamma(\alpha)\Gamma(\beta)}{\Gamma(\alpha + \beta)}$$

**Normal Distribution:**

$$F_{\text{Normal}}(x; \mu, \sigma) = \frac{1}{2}\left[1 + \text{erf}\left(\frac{x - \mu}{\sigma\sqrt{2}}\right)\right]$$

Where $\text{erf}$ is the error function:

$$\text{erf}(z) = \frac{2}{\sqrt{\pi}} \int_0^z e^{-t^2} dt$$

#### Algorithm: CDF Fitting

```
Algorithm: CDFFitting(data, q_points, cdf_min, cdf_max, distribution_type)
────────────────────────────────────────────────────────────────────────────
Input:
  - data: sample observations [x₁, x₂, ..., xₙ]
  - q_points: quantile evaluation points
  - cdf_min, cdf_max: confidence interval bounds
  - distribution_type: "Beta" or "Normal"

Output:
  - params_min: [α_min, β_min] fitted to lower bound
  - params_max: [α_max, β_max] fitted to upper bound
────────────────────────────────────────────────────────────────────────────
1.  // Prepare evaluation points
2.  sorted_data ← SORT(data)
3.  x ← [0] ∪ sorted_data ∪ [1]          // Add boundary anchors
4.  
5.  // Interpolate confidence bounds onto sorted data
6.  cdf_min_interp ← INTERPOLATE(q_points, cdf_min, x)
7.  cdf_max_interp ← INTERPOLATE(q_points, cdf_max, x)
8.  
9.  // Define initial parameter bounds based on distribution
10. IF distribution_type = "Beta":
11.     bounds ← [[0.1, 50], [0.1, 50]]   // α ∈ [0.1, 50], β ∈ [0.1, 50]
12. ELSE:
13.     bounds ← [[-10, 10], [0.01, 10]]  // μ ∈ [-10, 10], σ ∈ [0.01, 10]
14. 
15. // Fit to lower confidence bound
16. initial_guess ← [1.0, 1.0]
17. objective_min(θ) ← MSE(F_θ(x), cdf_min_interp)
18. params_min ← NELDER_MEAD(objective_min, initial_guess, bounds)
19. 
20. // Fit to upper confidence bound  
21. objective_max(θ) ← MSE(F_θ(x), cdf_max_interp)
22. params_max ← NELDER_MEAD(objective_max, initial_guess, bounds)
23. 
24. RETURN [params_min[0], params_min[1], params_max[0], params_max[1]]
────────────────────────────────────────────────────────────────────────────
```

#### Implementation Details

```rust
// Cost function for Nelder-Mead optimization
fn mse_cost(params: &[f64], _grad: Option<&mut [f64]>, 
            data: &(&Vec<f64>, &Vec<f64>, &DistributionType)) -> f64 {
    let (q, target_cdf, kind) = data;
    let dist = match kind {
        DistributionType::Beta => 
            Beta::new(params[0], params[1]).expect("Invalid params"),
        DistributionType::Normal => 
            Normal::new(params[0], params[1]).expect("Invalid params"),
    };
    
    let mse: f64 = q.iter()
        .zip(target_cdf.iter())
        .map(|(x, y)| (dist.cdf(*x) - y).powi(2))
        .sum::<f64>() / q.len() as f64;
    
    mse
}

// Optimizer setup
let mut opt = Nlopt::new(
    Algorithm::Neldermead, 
    2,                      // 2 parameters
    mse_cost, 
    Minimize, 
    (&q, &cdf_target, &kind)
);
opt.set_lower_bounds(&[0.1, 0.1]).unwrap();
opt.set_upper_bounds(&[50.0, 50.0]).unwrap();
opt.set_maxeval(10000).unwrap();
opt.set_xtol_rel(1e-8).unwrap();
```

---

### 3. XGBoost Prediction Pipeline

**Source Files:** 
- `engine/models/src/wrapper.rs` - Rust FFI bindings
- `xgbwrapper/src/xgbwrapper.c` - C implementation

#### XGBoost Algorithm Overview

**XGBoost** (eXtreme Gradient Boosting) is an ensemble learning method that builds trees sequentially, where each tree corrects errors from previous trees.

#### Mathematical Foundation

For $K$ trees, the prediction is:

$$\hat{y}_i = \sum_{k=1}^{K} f_k(x_i)$$

Where $f_k$ is the $k$-th regression tree.

**Objective Function:**

$$\mathcal{L}(\theta) = \sum_{i=1}^{n} l(y_i, \hat{y}_i) + \sum_{k=1}^{K} \Omega(f_k)$$

Where:
- $l$ = loss function (squared error for regression)
- $\Omega(f)$ = regularization term

**Regularization:**

$$\Omega(f) = \gamma T + \frac{1}{2}\lambda \sum_{j=1}^{T} w_j^2$$

Where:
- $T$ = number of leaves
- $w_j$ = weight of leaf $j$
- $\gamma, \lambda$ = regularization hyperparameters

#### Feature Engineering

The system transforms raw sample data into a 4-dimensional feature vector:

| Index | Feature | Description |
|-------|---------|-------------|
| 0 | $\alpha_{\min}$ | First parameter fitted to lower confidence bound |
| 1 | $\beta_{\min}$ | Second parameter fitted to lower confidence bound |
| 2 | $\alpha_{\max}$ | First parameter fitted to upper confidence bound |
| 3 | $\beta_{\max}$ | Second parameter fitted to upper confidence bound |

**Target Variables:**

| Index | Target | Description |
|-------|--------|-------------|
| 0 | $\alpha_{\text{true}}$ | True first distribution parameter |
| 1 | $\beta_{\text{true}}$ | True second distribution parameter |

#### Training Data Generation

Training data is generated synthetically:

```
Algorithm: GenerateTrainingData(N, n, distribution_type, num_samples)
────────────────────────────────────────────────────────────────────────────
1.  X ← []  // Feature matrix
2.  Y ← []  // Target matrix
3.  
4.  FOR i = 1 TO num_samples:
5.      // Sample random true parameters
6.      IF distribution_type = "Beta":
7.          α_true ~ Uniform(0.5, 20)
8.          β_true ~ Uniform(0.5, 20)
9.      ELSE:
10.         μ_true ~ Uniform(-5, 5)
11.         σ_true ~ Uniform(0.1, 5)
12.     
13.     // Generate synthetic sample from true distribution
14.     sample ← RANDOM_SAMPLE(Distribution(α_true, β_true), n)
15.     
16.     // Compute confidence intervals
17.     (q, cdf_min, cdf_max) ← ConfidenceInterval(N, n, α=0.05)
18.     
19.     // Fit CDFs to get features
20.     features ← CDFFitting(sample, q, cdf_min, cdf_max)
21.     
22.     // Store training example
23.     X.append(features)
24.     Y.append([α_true, β_true])
25. END FOR
26. 
27. RETURN (X, Y)
────────────────────────────────────────────────────────────────────────────
```

#### C Implementation: Training

```c
void train(float* x, float *y, int rows, int x_cols, int y_cols, 
           const KVPair* config, int len_config, const char* inference_path) {
    
    DMatrixHandle dtrain;
    
    // Create DMatrix from feature array
    XGDMatrixCreateFromMat(x, rows, x_cols, -1, &dtrain);
    
    // Set labels (target values)
    XGDMatrixSetFloatInfo(dtrain, "label", y, rows * y_cols);
    
    // Create booster
    BoosterHandle booster;
    XGBoosterCreate(&dtrain, 1, &booster);
    
    // Set hyperparameters
    int n_estimators = 100;
    for (int i = 0; i < len_config; ++i) {
        if (strcmp(config[i].key, "n_estimators") == 0) {
            n_estimators = atoi(config[i].value);
        } else {
            XGBoosterSetParam(booster, config[i].key, config[i].value);
        }
    }
    
    // Iterative training
    for (int i = 0; i < n_estimators; ++i) {
        XGBoosterUpdateOneIter(booster, i, dtrain);
    }
    
    // Save model
    XGBoosterSaveModel(booster, inference_path);
    
    // Cleanup
    XGDMatrixFree(dtrain);
    XGBoosterFree(booster);
}
```

#### C Implementation: Inference

```c
void predict(float* data, int rows, int x_cols, int y_cols,
             const char* inference_path, float *pred) {
    
    DMatrixHandle dtest;
    BoosterHandle booster;
    
    // Create DMatrix from input features
    XGDMatrixCreateFromMat(data, rows, x_cols, -1, &dtest);
    
    // Load trained model
    XGBoosterCreate(&dtest, 1, &booster);
    XGBoosterLoadModel(booster, inference_path);
    
    // Make predictions
    bst_ulong out_len;
    const float* out_result;
    XGBoosterPredict(booster, dtest, 0, 0, 0, &out_len, &out_result);
    
    // Copy results
    memcpy(pred, out_result, rows * y_cols * sizeof(float));
    
    // Cleanup
    XGDMatrixFree(dtest);
    XGBoosterFree(booster);
}
```

#### Rust FFI Bindings

```rust
#[repr(C)]
pub struct KVPair {
    pub key: *const c_char,
    pub value: *const c_char,
}

extern "C" {
    fn train(
        x: *const c_float, 
        y: *const c_float, 
        rows: c_int,
        x_cols: c_int, 
        y_cols: c_int,
        config: *const KVPair, 
        len_config: c_int, 
        inference_path: *const c_char
    );
    
    fn predict(
        x: *const c_float, 
        rows: c_int, 
        x_cols: c_int, 
        y_cols: c_int,
        inference_path: *const c_char, 
        pred: *mut c_float
    );
}

pub fn xgb_predict(x: &Vec<[f64; 4]>, inference_path: &str) -> Vec<[f64; 2]> {
    let rows = x.len() as c_int;
    let x_cols = 4 as c_int;
    let y_cols = 2 as c_int;
    
    // Flatten input to row-major format
    let x_flat: Vec<c_float> = x.iter()
        .flat_map(|row| row.iter().map(|&v| v as c_float))
        .collect();
    
    let mut pred = vec![0.0f32; (rows * y_cols) as usize];
    let path = CString::new(inference_path).unwrap();
    
    unsafe {
        predict(
            x_flat.as_ptr(),
            rows, x_cols, y_cols,
            path.as_ptr(),
            pred.as_mut_ptr()
        );
    }
    
    // Reshape output
    pred.chunks(2)
        .map(|chunk| [chunk[0] as f64, chunk[1] as f64])
        .collect()
}
```

#### Sample-Size Specific Models

The system maintains separate models for different sample sizes to optimize prediction accuracy:

| Model File | Sample Size |
|------------|-------------|
| `xgb_Beta_5.json` | n = 5 |
| `xgb_Beta_10.json` | n = 10 |
| `xgb_Beta_15.json` | n = 15 |
| ... | ... |
| `xgb_Beta_100.json` | n = 100 |

Model selection logic:
```rust
fn select_model(sample_size: usize, distribution: &str) -> String {
    let n_step = 5;  // From config
    let model_n = ((sample_size / n_step) * n_step).max(5);
    format!("inference/xgb_{}_{}.json", distribution, model_n)
}
```

---

### 4. Chi-Square Goodness-of-Fit Testing

**Source File:** `engine/models/src/train.rs` - `chi2_test()`, `chi2_stat()`

#### Purpose

Validate how well the estimated distribution parameters fit the observed data using the chi-square goodness-of-fit test.

#### Test Statistic

The Pearson chi-square statistic measures the discrepancy between observed and expected frequencies:

$$\chi^2 = \sum_{i=1}^{k} \frac{(O_i - E_i)^2}{E_i}$$

Where:
- $O_i$ = observed frequency in bin $i$
- $E_i$ = expected frequency in bin $i$ under the fitted distribution
- $k$ = number of bins

#### Expected Frequencies

For a continuous distribution with CDF $F_{\theta}$, the expected frequency in bin $[a_i, b_i]$ is:

$$E_i = n \cdot \left( F_{\theta}(b_i) - F_{\theta}(a_i) \right)$$

Where $n$ is the total sample size.

#### Degrees of Freedom

$$\text{df} = k - 1 - p$$

Where:
- $k$ = number of bins
- $p$ = number of estimated parameters (2 for Beta and Normal)

For practical purposes, when comparing fits with the same number of parameters:
$$\text{df} = k - 1$$

#### Hypothesis Testing

- **Null Hypothesis ($H_0$)**: The data follows the specified distribution
- **Alternative Hypothesis ($H_1$)**: The data does not follow the specified distribution

**Decision Rule:**

$$\text{Reject } H_0 \text{ if } \chi^2 > \chi^2_{\alpha, \text{df}}$$

Where $\chi^2_{\alpha, \text{df}}$ is the critical value from the chi-square distribution.

#### P-Value Calculation

$$p\text{-value} = P(\chi^2_{\text{df}} > \chi^2_{\text{observed}}) = 1 - F_{\chi^2}(\chi^2_{\text{observed}})$$

#### Implementation

```rust
/// Compute chi-square test statistic
pub fn chi2_stat(observed: &Vec<f64>, expected: &Vec<f64>) -> f64 {
    observed.iter()
        .zip(expected.iter())
        .filter(|(_, e)| **e > 0.0)  // Avoid division by zero
        .map(|(o, e)| (o - e).powi(2) / e)
        .sum()
}

/// Perform chi-square goodness-of-fit test
/// Returns: (chi2_statistic, critical_value, p_value, decision)
/// decision = true means accept H₀ (good fit)
pub fn chi2_test(
    observed: &Vec<f64>, 
    expected: &Vec<f64>, 
    significance: f64
) -> (f64, f64, f64, bool) {
    let chi2 = chi2_stat(observed, expected);
    let dof = (observed.len() as f64 - 1.0).max(1.0);
    
    let chi2_dist = ChiSquared::new(dof)
        .expect("Invalid degrees of freedom");
    
    let p_value = 1.0 - chi2_dist.cdf(chi2);
    let critical_value = chi2_dist.inverse_cdf(1.0 - significance);
    
    let decision = chi2 <= critical_value;  // true = accept H₀
    
    (chi2, critical_value, p_value, decision)
}
```

#### Bin Frequency Calculation

```rust
/// Calculate observed frequencies from data
pub fn calculate_freq(data: &Vec<f64>, bins: &Vec<f64>) -> Vec<f64> {
    let mut freq = vec![0.0; bins.len() - 1];
    
    for &x in data {
        for i in 0..freq.len() {
            if x >= bins[i] && x < bins[i + 1] {
                freq[i] += 1.0;
                break;
            }
        }
        // Handle upper boundary
        if x == bins[bins.len() - 1] {
            freq[freq.len() - 1] += 1.0;
        }
    }
    
    freq
}

/// Calculate expected frequencies from theoretical distribution
pub fn expected_freq(
    params: &[f64; 2], 
    bins: &Vec<f64>, 
    sample_size: usize,
    kind: DistributionType
) -> Vec<f64> {
    let dist = match kind {
        DistributionType::Beta => Beta::new(params[0], params[1]).unwrap(),
        DistributionType::Normal => Normal::new(params[0], params[1]).unwrap(),
    };
    
    let n = sample_size as f64;
    (0..bins.len()-1)
        .map(|i| n * (dist.cdf(bins[i + 1]) - dist.cdf(bins[i])))
        .collect()
}
```

---

### 5. Sampling Parameter Estimation (Method of Moments)

**Source File:** `engine/models/src/train.rs` - `calculate_sampling_params()`

#### Purpose

Provide baseline parameter estimates using classical statistical methods for comparison with ML predictions.

#### Method of Moments

The method of moments estimates distribution parameters by equating sample moments to theoretical moments.

#### Beta Distribution

**Sample Statistics:**
$$\bar{x} = \frac{1}{n}\sum_{i=1}^{n} x_i$$

$$s^2 = \frac{1}{n-1}\sum_{i=1}^{n} (x_i - \bar{x})^2$$

**Theoretical Moments for Beta($\alpha$, $\beta$):**

$$E[X] = \frac{\alpha}{\alpha + \beta}$$

$$\text{Var}[X] = \frac{\alpha\beta}{(\alpha + \beta)^2(\alpha + \beta + 1)}$$

**Parameter Estimation:**

Setting $E[X] = \bar{x}$ and $\text{Var}[X] = s^2$:

$$\text{coeff} = \frac{\bar{x}(1-\bar{x})}{s^2} - 1$$

$$\hat{\alpha} = \text{coeff} \cdot \bar{x}$$

$$\hat{\beta} = \text{coeff} \cdot (1 - \bar{x})$$

#### Normal Distribution

The method of moments estimators for the normal distribution are simply:

$$\hat{\mu} = \bar{x}$$

$$\hat{\sigma} = s$$

#### Implementation

```rust
pub fn calculate_sampling_params(
    kind: DistributionType, 
    data: Vec<f64>
) -> [f64; 4] {
    let n = data.len() as f64;
    
    // Sample mean
    let mean = data.iter().sum::<f64>() / n;
    
    // Sample variance (using Bessel's correction)
    let var = data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (n - 1.0);
    
    let std = var.sqrt();
    
    match kind {
        DistributionType::Beta => {
            // Method of moments for Beta distribution
            let coeff = mean * (1.0 - mean) / var - 1.0;
            
            // Ensure positive parameters
            let alpha = (coeff * mean).max(0.01);
            let beta = (coeff * (1.0 - mean)).max(0.01);
            
            [mean, std, alpha, beta]
        }
        DistributionType::Normal => {
            // Method of moments for Normal distribution
            [mean, std, mean, std]
        }
    }
}
```

---

## Data Flow: Complete Request Processing

### API Endpoint: `handle_calc`

**Source File:** `engine/server/src/api.rs`

This is the main computation endpoint that orchestrates all algorithms.

#### Request Format

```json
{
    "command": "calc",
    "kind": 0,                           // 0=Beta, 1=Normal
    "test_mode": false,
    "data": [0.45, 0.52, 0.38, ...],    // Sample observations
    "min_value": 0.0,                    // Domain minimum
    "max_value": 100.0,                  // Domain maximum
    "population_size": 3000,             // Batch/population size
    "bins_number": 10                    // Histogram bins
}
```

#### Processing Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          handle_calc() Pipeline                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  STAGE 1: INPUT VALIDATION                                                  │
│  ─────────────────────────────────────────────────────────────────────────  │
│  │                                                                         │
│  ├─ Validate distribution type (kind ∈ {0, 1})                            │
│  ├─ Check min_value < max_value                                            │
│  ├─ Verify data contains no NaN or Inf values                             │
│  ├─ Ensure population_size > 0                                             │
│  └─ Validate bins_number > 0                                               │
│                                                                             │
│  STAGE 2: DATA PREPROCESSING                                                │
│  ─────────────────────────────────────────────────────────────────────────  │
│  │                                                                         │
│  │  For each x_i in data:                                                  │
│  │      x'_i = (x_i - min_value) / (max_value - min_value)                │
│  │                                                                         │
│  └─ sorted_data = SORT(scaled_data)                                        │
│                                                                             │
│  STAGE 3: CONFIDENCE INTERVALS (Hypergeometric)                            │
│  ─────────────────────────────────────────────────────────────────────────  │
│  │                                                                         │
│  └─ (q_points, cdf_min, cdf_max) = conf_int(N, n, α=0.05)                 │
│                                                                             │
│  STAGE 4: CDF FITTING (Nelder-Mead)                                        │
│  ─────────────────────────────────────────────────────────────────────────  │
│  │                                                                         │
│  ├─ features = cdf_fitting(sorted_data, q, cdf_min, cdf_max, kind)        │
│  │             → [α_min, β_min, α_max, β_max]                             │
│  │                                                                         │
│  ├─ params_min = [features[0], features[1]]                               │
│  └─ params_max = [features[2], features[3]]                               │
│                                                                             │
│  STAGE 5: ML PREDICTION (XGBoost)                                          │
│  ─────────────────────────────────────────────────────────────────────────  │
│  │                                                                         │
│  ├─ model_path = select_model(sample_size, distribution_type)             │
│  └─ predicted_params = xgb_predict([features], model_path)                │
│                        → [α_pred, β_pred]                                  │
│                                                                             │
│  STAGE 6: SAMPLING ESTIMATION (Method of Moments)                          │
│  ─────────────────────────────────────────────────────────────────────────  │
│  │                                                                         │
│  └─ sampling_params = calculate_sampling_params(kind, sorted_data)        │
│                       → [mean, std, α_sampling, β_sampling]               │
│                                                                             │
│  STAGE 7: FREQUENCY ANALYSIS                                               │
│  ─────────────────────────────────────────────────────────────────────────  │
│  │                                                                         │
│  ├─ bins = linspace(0, 1, bins_number + 1)                                │
│  ├─ observed_freq = calculate_freq(sorted_data, bins)                     │
│  │                                                                         │
│  ├─ expected_min = expected_freq(params_min, bins, n, kind)               │
│  ├─ expected_max = expected_freq(params_max, bins, n, kind)               │
│  └─ expected_pred = expected_freq(predicted_params, bins, n, kind)        │
│                                                                             │
│  STAGE 8: CHI-SQUARE TESTING                                               │
│  ─────────────────────────────────────────────────────────────────────────  │
│  │                                                                         │
│  ├─ (χ²_min, crit_min, p_min, dec_min) =                                  │
│  │       chi2_test(observed_freq, expected_min, α=0.05)                   │
│  │                                                                         │
│  ├─ (χ²_max, crit_max, p_max, dec_max) =                                  │
│  │       chi2_test(observed_freq, expected_max, α=0.05)                   │
│  │                                                                         │
│  └─ (χ²_pred, crit_pred, p_pred, dec_pred) =                              │
│          chi2_test(observed_freq, expected_pred, α=0.05)                  │
│                                                                             │
│  STAGE 9: RESPONSE ASSEMBLY                                                │
│  ─────────────────────────────────────────────────────────────────────────  │
│  │                                                                         │
│  └─ Return JSON response with all computed values                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Response Format

```json
{
    "status": "success",
    "scaled_data": [0.0045, 0.0052, ...],
    "q_points": [0.0, 0.01, 0.02, ...],
    "cdf_min": [0.0, 0.0, 0.0, ...],
    "cdf_max": [0.1, 0.15, 0.2, ...],
    
    "params_min": [2.5, 3.1],
    "params_max": [4.2, 5.8],
    "predicted_params": [3.2, 4.5],
    "sampling_params": [0.42, 0.15, 2.8, 4.1],
    
    "bins": [0.0, 0.1, 0.2, ...],
    "observed_freq": [2, 5, 8, ...],
    "expected_min": [1.8, 4.2, 7.5, ...],
    "expected_max": [2.5, 5.8, 8.2, ...],
    "expected_pred": [2.1, 4.8, 7.9, ...],
    
    "min_chi2": 3.45,
    "min_crit": 16.92,
    "min_pval": 0.85,
    "min_decision": true,
    
    "max_chi2": 4.12,
    "max_crit": 16.92,
    "max_pval": 0.78,
    "max_decision": true,
    
    "predicted_chi2": 2.89,
    "predicted_crit": 16.92,
    "predicted_pval": 0.92,
    "predicted_decision": true
}
```

---

## XGBoost Wrapper (C Library)

**Location:** `xgbwrapper/`

### Directory Structure

```
xgbwrapper/
├── CMakeLists.txt          # Build configuration
├── CMakePresets.json       # CMake presets
├── CPackConfig.cmake       # Packaging configuration
├── src/
│   ├── xgbwrapper.h       # Public API header
│   └── xgbwrapper.c       # Implementation
└── tests/
    ├── test_xgbwrapper.h  # Test header
    └── test_xgbwrapper.c  # Test implementation
```

### Public API

```c
// xgbwrapper.h

#ifndef XGBWRAPPER_H
#define XGBWRAPPER_H

// Key-value pair for configuration
typedef struct {
    const char* key;
    const char* value;
} KVPair;

/**
 * Shuffle an integer array using Fisher-Yates algorithm
 * @param array  Array to shuffle (modified in place)
 * @param n      Array length
 */
void shuffle(int* array, int n);

/**
 * Split data into training and test sets
 * @param x          Feature matrix (row-major)
 * @param y          Target matrix (row-major)
 * @param x_train    Output: training features
 * @param y_train    Output: training targets
 * @param x_test     Output: test features
 * @param y_test     Output: test targets
 * @param x_cols     Number of feature columns
 * @param y_cols     Number of target columns
 * @param rows       Total number of samples
 * @param rows_train Number of training samples
 */
void split_data(const float* x, const float* y,
                float* x_train, float* y_train,
                float* x_test, float* y_test,
                int x_cols, int y_cols, int rows, int rows_train);

/**
 * Train XGBoost model
 * @param x              Feature matrix
 * @param y              Target matrix
 * @param rows           Number of samples
 * @param x_cols         Number of features
 * @param y_cols         Number of targets
 * @param config         Hyperparameter configuration
 * @param len_config     Number of config entries
 * @param inference_path Path to save model
 */
void train(float* x, float* y, int rows, int x_cols, int y_cols,
           const KVPair* config, int len_config, const char* inference_path);

/**
 * Make predictions with trained model
 * @param data           Feature matrix
 * @param rows           Number of samples
 * @param x_cols         Number of features
 * @param y_cols         Number of targets
 * @param inference_path Path to model file
 * @param pred           Output: predictions
 */
void predict(float* data, int rows, int x_cols, int y_cols,
             const char* inference_path, float* pred);

/**
 * Calculate RMSE for each target column
 * @param y_pred  Predicted values
 * @param y_test  Actual values
 * @param rows    Number of samples
 * @param y_cols  Number of targets
 * @param rmse    Output: RMSE for each target
 */
void calculate_rmse(const float* y_pred, const float* y_test,
                    int rows, int y_cols, float* rmse);

#endif // XGBWRAPPER_H
```

### Fisher-Yates Shuffle

```c
void shuffle(int* array, int n) {
    srand(time(NULL));
    for (int i = n - 1; i > 0; i--) {
        int j = rand() % (i + 1);
        // Swap array[i] and array[j]
        int temp = array[i];
        array[i] = array[j];
        array[j] = temp;
    }
}
```

### Data Splitting

```c
void split_data(const float* x, const float* y,
                float* x_train, float* y_train,
                float* x_test, float* y_test,
                int x_cols, int y_cols, int rows, int rows_train) {
    
    // Create index array
    int* indices = malloc(rows * sizeof(int));
    for (int i = 0; i < rows; i++) {
        indices[i] = i;
    }
    
    // Shuffle indices
    shuffle(indices, rows);
    
    // Split based on shuffled indices
    for (int i = 0; i < rows_train; i++) {
        int idx = indices[i];
        for (int j = 0; j < x_cols; j++) {
            x_train[i * x_cols + j] = x[idx * x_cols + j];
        }
        for (int j = 0; j < y_cols; j++) {
            y_train[i * y_cols + j] = y[idx * y_cols + j];
        }
    }
    
    int rows_test = rows - rows_train;
    for (int i = 0; i < rows_test; i++) {
        int idx = indices[rows_train + i];
        for (int j = 0; j < x_cols; j++) {
            x_test[i * x_cols + j] = x[idx * x_cols + j];
        }
        for (int j = 0; j < y_cols; j++) {
            y_test[i * y_cols + j] = y[idx * y_cols + j];
        }
    }
    
    free(indices);
}
```

### RMSE Calculation

$$\text{RMSE}_j = \sqrt{\frac{1}{n} \sum_{i=1}^{n} (\hat{y}_{ij} - y_{ij})^2}$$

```c
void calculate_rmse(const float* y_pred, const float* y_test,
                    int rows, int y_cols, float* rmse) {
    for (int j = 0; j < y_cols; j++) {
        float sse = 0.0f;
        for (int i = 0; i < rows; i++) {
            float diff = y_pred[i * y_cols + j] - y_test[i * y_cols + j];
            sse += diff * diff;
        }
        rmse[j] = sqrtf(sse / rows);
    }
}
```

---

## Frontend (Vue.js)

**Location:** `ui/`

### Technology Stack

| Technology | Purpose |
|------------|---------|
| Vue.js 3 | Reactive UI framework |
| Vite | Build tool and dev server |
| Pinia | State management |
| TailwindCSS | Utility-first CSS |
| Vue Router | Client-side routing |
| vue-i18n | Internationalization |

### Application Structure

```
ui/
├── index.html              # Entry HTML
├── package.json            # Dependencies
├── vite.config.js         # Vite configuration
├── tailwind.config.js     # Tailwind configuration
├── postcss.config.js      # PostCSS configuration
├── public/                # Static assets
└── src/
    ├── App.vue            # Root component
    ├── main.js            # Application entry
    ├── style.css          # Global styles
    ├── assets/            # Compiled assets
    ├── components/
    │   ├── Dashboard.vue  # Main layout
    │   ├── beta_tool/
    │   │   ├── BetaTool.vue          # Container
    │   │   ├── Inputs.vue            # Input form
    │   │   ├── Cdf.vue               # CDF chart
    │   │   ├── Pdf.vue               # PDF chart
    │   │   ├── Freq.vue              # Histogram
    │   │   ├── ChiSquared.vue        # Test results
    │   │   └── DistributionParams.vue # Parameters
    │   ├── normal_tool/   # Normal distribution tool
    │   └── defects_rate/  # Defect rate tool
    ├── locales/
    │   ├── en-us.json     # English translations
    │   └── pt-br.json     # Portuguese translations
    ├── router/
    │   └── index.js       # Route definitions
    ├── services/
    │   ├── i18n.js        # i18n service
    │   └── websocketService.js  # WebSocket client
    ├── store/
    │   └── index.js       # Pinia store
    └── views/
        ├── Home.vue       # Home page
        └── Help.vue       # Help page
```

### State Management (Pinia Store)

```javascript
// store/index.js
import { defineStore } from 'pinia';

export const useMainStore = defineStore('main', {
    state: () => ({
        // Connection state
        connected: false,
        
        // Input parameters
        batchVolume: 3000,
        minValue: 0,
        maxValue: 100,
        binsNumber: 10,
        samplingData: "",
        
        // Processed data
        scaledData: [],
        qPoints: [],
        cdfMin: [],
        cdfMax: [],
        
        // Fitted parameters
        paramsMin: [0.0, 0.0],
        paramsMax: [0.0, 0.0],
        predictedParams: [0.0, 0.0],
        samplingParams: [0.0, 0.0, 0.0, 0.0],
        
        // Frequency data
        bins: [],
        observedFreq: [],
        expectedMin: [],
        expectedMax: [],
        expectedPred: [],
        
        // Chi-square test results (min fit)
        minChi2: 0.0,
        minCrit: 0.0,
        minPval: 0.0,
        minDecision: false,
        
        // Chi-square test results (max fit)
        maxChi2: 0.0,
        maxCrit: 0.0,
        maxPval: 0.0,
        maxDecision: false,
        
        // Chi-square test results (predicted)
        predictedChi2: 0.0,
        predictedCrit: 0.0,
        predictedPval: 0.0,
        predictedDecision: false,
    }),
    
    actions: {
        updateFromResponse(response) {
            // Update all state from server response
            this.scaledData = response.scaled_data;
            this.qPoints = response.q_points;
            this.cdfMin = response.cdf_min;
            this.cdfMax = response.cdf_max;
            // ... etc
        },
        
        resetState() {
            // Reset to default values
        }
    }
});
```

### WebSocket Service

```javascript
// services/websocketService.js

class WebSocketService {
    constructor() {
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000;
    }
    
    connect(url) {
        return new Promise((resolve, reject) => {
            this.ws = new WebSocket(url);
            
            this.ws.onopen = () => {
                this.reconnectAttempts = 0;
                resolve();
            };
            
            this.ws.onclose = () => {
                this.attemptReconnect(url);
            };
            
            this.ws.onerror = (error) => {
                reject(error);
            };
        });
    }
    
    send(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        }
    }
    
    onMessage(callback) {
        if (this.ws) {
            this.ws.onmessage = (event) => {
                const data = JSON.parse(event.data);
                callback(data);
            };
        }
    }
    
    attemptReconnect(url) {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            setTimeout(() => {
                this.connect(url);
            }, this.reconnectDelay * this.reconnectAttempts);
        }
    }
}

export default new WebSocketService();
```

### Component Communication Flow

```
┌──────────────────────────────────────────────────────────────────────┐
│                            Dashboard.vue                              │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │                         BetaTool.vue                            │  │
│  │                                                                  │  │
│  │  ┌─────────────┐   User Input    ┌──────────────────────────┐  │  │
│  │  │ Inputs.vue  │ ─────────────▶  │   Pinia Store            │  │  │
│  │  └─────────────┘                 │   (state management)     │  │  │
│  │                                  └──────────┬───────────────┘  │  │
│  │                                             │                   │  │
│  │                                    WebSocket Request            │  │
│  │                                             │                   │  │
│  │                                             ▼                   │  │
│  │                                  ┌──────────────────────────┐  │  │
│  │                                  │   Rust Engine Server     │  │  │
│  │                                  └──────────┬───────────────┘  │  │
│  │                                             │                   │  │
│  │                                    WebSocket Response           │  │
│  │                                             │                   │  │
│  │                                             ▼                   │  │
│  │  ┌─────────────┐                 ┌──────────────────────────┐  │  │
│  │  │  Cdf.vue    │ ◀───────────── │   Pinia Store            │  │  │
│  │  │  Pdf.vue    │   Reactive     │   (updated state)        │  │  │
│  │  │  Freq.vue   │   Binding      │                          │  │  │
│  │  │ ChiSquared  │                └──────────────────────────┘  │  │
│  │  └─────────────┘                                               │  │
│  │                                                                  │  │
│  └────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Configuration

**Location:** `data/config.json`

### Configuration Schema

```json
{
    "main_params": {
        "booster": "gbtree",
        "objective": "reg:squarederror",
        "n_estimators": "500",
        "max_depth": "10",
        "learning_rate": "0.3",
        "subsample": "0.8",
        "colsample_bytree": "0.8",
        "min_child_weight": "1",
        "gamma": "0",
        "reg_alpha": "0",
        "reg_lambda": "1"
    },
    "n_min": 5,
    "n_max": 100,
    "n_step": 5
}
```

### Parameter Descriptions

| Parameter | Type | Description |
|-----------|------|-------------|
| `booster` | string | Type of booster: `gbtree`, `gblinear`, `dart` |
| `objective` | string | Learning objective function |
| `n_estimators` | int | Number of boosting rounds |
| `max_depth` | int | Maximum tree depth |
| `learning_rate` | float | Step size shrinkage (η) |
| `subsample` | float | Row subsampling ratio |
| `colsample_bytree` | float | Column subsampling ratio |
| `min_child_weight` | float | Minimum sum of instance weight in child |
| `gamma` | float | Minimum loss reduction for split |
| `reg_alpha` | float | L1 regularization term |
| `reg_lambda` | float | L2 regularization term |
| `n_min` | int | Minimum sample size for models |
| `n_max` | int | Maximum sample size for models |
| `n_step` | int | Step between model sample sizes |

### Model File Naming Convention

Models are stored with the pattern:
```
inference/xgb_{distribution}_{sample_size}.json
```

Examples:
- `inference/xgb_Beta_5.json`
- `inference/xgb_Beta_10.json`
- `inference/xgb_Normal_50.json`

---

## Build System

### XGBoost Wrapper Build

**CMakeLists.txt:**
```cmake
cmake_minimum_required(VERSION 3.10)
project(xgbwrapper VERSION 0.1.0 LANGUAGES C)

# Set C standard
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Include XGBoost headers
include_directories(/home/vp/xgboost-headers/include)

# Create shared library
add_library(xgbwrapper SHARED src/xgbwrapper.c)

# Link XGBoost library
target_link_libraries(xgbwrapper PRIVATE 
    /home/vp/quality_control_room/lib/libxgboost.so)

# Install targets
install(TARGETS xgbwrapper
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib)
install(FILES src/xgbwrapper.h DESTINATION include)
```

**Build Commands:**
```bash
cd xgbwrapper
cmake --preset=default
cmake --build build
```

### Rust Engine Build

**engine/models/build.rs:**
```rust
fn main() {
    let lib_path = "/home/vp/quality_control_room/lib";
    
    // Link search path
    println!("cargo:rustc-link-search=native={}", lib_path);
    
    // Link libraries
    println!("cargo:rustc-link-lib=dylib=xgbwrapper");
    println!("cargo:rustc-link-lib=dylib=xgboost");
    
    // Runtime library path
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path);
}
```

**Build Commands:**
```bash
cd engine
cargo build --release
```

### Frontend Build

```bash
cd ui
npm install
npm run build    # Production build
npm run dev      # Development server
```

---

## Supported Distributions

### Beta Distribution

**Probability Density Function:**

$$f(x; \alpha, \beta) = \frac{x^{\alpha-1}(1-x)^{\beta-1}}{B(\alpha, \beta)}$$

Where $B(\alpha, \beta)$ is the beta function.

**Domain:** $x \in [0, 1]$

**Parameters:**
- $\alpha > 0$ (shape parameter 1)
- $\beta > 0$ (shape parameter 2)

**Moments:**
- Mean: $E[X] = \frac{\alpha}{\alpha + \beta}$
- Variance: $\text{Var}[X] = \frac{\alpha\beta}{(\alpha + \beta)^2(\alpha + \beta + 1)}$
- Mode: $\frac{\alpha - 1}{\alpha + \beta - 2}$ (for $\alpha, \beta > 1$)

**Use Cases:**
- Proportion data (percentages, rates)
- Bounded continuous measurements
- Quality metrics on [0, 1] scale

### Normal Distribution

**Probability Density Function:**

$$f(x; \mu, \sigma) = \frac{1}{\sigma\sqrt{2\pi}} \exp\left(-\frac{(x-\mu)^2}{2\sigma^2}\right)$$

**Domain:** $x \in (-\infty, +\infty)$

**Parameters:**
- $\mu \in \mathbb{R}$ (location/mean)
- $\sigma > 0$ (scale/standard deviation)

**Moments:**
- Mean: $E[X] = \mu$
- Variance: $\text{Var}[X] = \sigma^2$
- Mode: $\mu$

**Use Cases:**
- Measurement errors
- Physical dimensions
- Process variations

### Data Scaling

For the Beta distribution, input data is scaled to [0, 1]:

$$x' = \frac{x - x_{\min}}{x_{\max} - x_{\min}}$$

For inverse transformation:

$$x = x' \cdot (x_{\max} - x_{\min}) + x_{\min}$$

Parameters are reported in both scaled and original units.

---

## Test Suite

**Location:** `engine/tests/`

### Test Categories

| Test | Description |
|------|-------------|
| `test_split_data` | Data partitioning into train/test sets |
| `test_xgb` | End-to-end XGBoost training and prediction |
| `test_chi2` | Chi-square statistic and test validation |
| `test_cdf_min_max` | Confidence interval computation |
| `test_freq` | Frequency histogram calculation |
| `test_nmead` | Nelder-Mead optimization convergence |
| `test_sampling_params` | Method of moments estimation |

### Running Tests

```bash
cd engine
cargo test --workspace
```

### Example Test: Chi-Square

```rust
#[test]
fn test_chi2() {
    let observed = vec![10.0, 15.0, 20.0, 25.0, 30.0];
    let expected = vec![12.0, 14.0, 22.0, 24.0, 28.0];
    
    let (chi2, crit, pval, decision) = chi2_test(&observed, &expected, 0.05);
    
    // Verify chi-square statistic
    let manual_chi2 = (10.0-12.0).powi(2)/12.0 
                    + (15.0-14.0).powi(2)/14.0 
                    + (20.0-22.0).powi(2)/22.0 
                    + (25.0-24.0).powi(2)/24.0 
                    + (30.0-28.0).powi(2)/28.0;
    
    assert!((chi2 - manual_chi2).abs() < 1e-10);
    assert!(pval > 0.0 && pval < 1.0);
    assert!(crit > 0.0);
}
```

---

## Deployment

### Systemd Service

**Location:** `systemd/quality-engine.service`

```ini
[Unit]
Description=Quality Control Room Engine
After=network.target

[Service]
Type=simple
User=vp
WorkingDirectory=/home/vp/quality_control_room
ExecStart=/home/vp/quality_control_room/engine/target/release/server
Restart=on-failure
RestartSec=5
Environment=LD_LIBRARY_PATH=/home/vp/quality_control_room/lib

[Install]
WantedBy=multi-user.target
```

### Service Management

```bash
# Install service
sudo cp systemd/quality-engine.service /etc/systemd/system/
sudo systemctl daemon-reload

# Start service
sudo systemctl start quality-engine

# Enable on boot
sudo systemctl enable quality-engine

# Check status
sudo systemctl status quality-engine
```

---

## Glossary

| Term | Definition |
|------|------------|
| **CDF** | Cumulative Distribution Function - $F(x) = P(X \leq x)$ |
| **PDF** | Probability Density Function - derivative of CDF |
| **PMF** | Probability Mass Function - discrete analog of PDF |
| **Hypergeometric** | Distribution for sampling without replacement |
| **Nelder-Mead** | Derivative-free simplex optimization algorithm |
| **XGBoost** | eXtreme Gradient Boosting - ensemble ML method |
| **Chi-Square Test** | Goodness-of-fit test comparing observed vs expected frequencies |
| **Method of Moments** | Parameter estimation by equating sample and theoretical moments |
| **Confidence Interval** | Range likely to contain true parameter value |
| **RMSE** | Root Mean Square Error - prediction accuracy metric |

---

## References

1. Johnson, N. L., Kotz, S., & Balakrishnan, N. (1995). *Continuous Univariate Distributions, Vol. 2*. Wiley.
2. Chen, T., & Guestrin, C. (2016). XGBoost: A Scalable Tree Boosting System. *KDD*.
3. Nelder, J. A., & Mead, R. (1965). A Simplex Method for Function Minimization. *The Computer Journal*.
4. Pearson, K. (1900). On the Criterion that a Given System of Deviations from the Probable. *Philosophical Magazine*.
5. Rice, J. A. (2007). *Mathematical Statistics and Data Analysis*. Cengage Learning.

---

*Documentation generated: January 2026*
*Version: 1.0.0*
