# Server Technical Documentation

> **Priority**: This document is authoritative for the server component. External documents will be aligned with this specification.

## Overview

The Quality Control Room server is a WebSocket API server for statistical analysis of quality control samples. It provides distribution fitting, confidence intervals, and goodness-of-fit testing using trained XGBoost models.

**Stack**: Rust + axum + axum-server (TLS via rustls)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Client (UI)                         │
└─────────────────────────┬───────────────────────────────────┘
                          │ WebSocket (JSON)
┌─────────────────────────▼───────────────────────────────────┐
│                      main.rs                                │
│  - TLS/non-TLS server setup (conditional compilation)       │
│  - WebSocket upgrade, tracing init                          │
│  - Routes via api::router()                                 │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                      api/ module                            │
│  ├── mod.rs       - router(), handle_request()              │
│  ├── types.rs     - ApiRequest, ApiResponse                 │
│  ├── state.rs     - AppState, find_model()                  │
│  ├── analyze.rs   - handle_about(), handle_analyze()        │
│  ├── curves.rs    - handle_get_intervals/cdf/pdf()          │
│  └── histogram.rs - handle_get_histogram()                  │
└──────────┬──────────────────────────────────┬───────────────┘
           │                                  │
┌──────────▼──────────┐          ┌────────────▼────────────────┐
│      stats.rs       │          │          xgb.rs             │
│  - conf_int()       │          │  - FFI to libxgbwrapper     │
│  - cdf(), pdf()     │          │  - predict()                │
│  - chi_square_test()│          │                             │
│  - method_of_moments│          │                             │
└─────────────────────┘          └──────────────────────────────┘
```

## Configuration

File: `config.yaml`

```yaml
server:
  host: "0.0.0.0"           # Bind address
  port: 8081                # Listen port
  ws_path: "quality"        # WebSocket endpoint path
  tls:                      # Optional TLS config
    cert_path: "/path/to/cert.pem"
    key_path: "/path/to/key.pem"

models:
  models_dir: "../models"   # XGBoost model directory
  sample_sizes: [5, 10, 20, 50, 100]  # Trained model sizes

statistics:
  default_population_size: 10000  # N for hypergeometric CI
  alpha: 0.05                     # Chi-square significance level
  default_bins: 10                # Histogram bin count
  prob_threshold_factor: 10.0     # Quality interval threshold
```

## WebSocket API

### Protocol

- **Transport**: WebSocket (ws:// or wss://)
- **Endpoint**: `ws[s]://{host}:{port}/{ws_path}`
- **Format**: JSON (request and response)

### Commands

| Command | Purpose | Response Size |
|---------|---------|---------------|
| `about` | Server info | ~50 bytes |
| `analyze` | Core analysis (params, chi2) | ~2KB |
| `get_intervals` | Confidence interval curves | ~8KB |
| `get_cdf` | CDF curves | ~8KB |
| `get_pdf` | PDF curves | ~8KB |
| `get_histogram` | Histogram + frequencies | ~2KB |

### Typical Workflow

```
1. Client → analyze(data)     → params, chi2, scaled_data
2. Client → get_cdf(params)   → CDF curves (when plotting)
3. Client → get_histogram()   → histogram (on-demand)
```

---

## Command Reference

### `about`

Server identification.

**Request:**
```json
{"command": "about"}
```

**Response:**
```json
{
  "command": "about",
  "success": true,
  "version": "0.1.0",
  "message": "Quality Control Room Server"
}
```

---

### `analyze`

Core analysis: scales data, fits parameters, runs XGBoost prediction, computes chi-square tests.

**Request:**
```json
{
  "command": "analyze",
  "distribution": 0,
  "data": [12.5, 14.2, 11.8, ...],
  "min_value": 0,
  "max_value": 100,
  "population_size": 10000
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `distribution` | u8 | Yes | 0 = Beta, 1 = Normal |
| `data` | f64[] | Yes | Raw sample values |
| `min_value` | f64 | No | Domain lower bound (auto-detect if omitted) |
| `max_value` | f64 | No | Domain upper bound (auto-detect if omitted) |
| `population_size` | usize | No | Population size for CI (default: 10000) |

**Response:**
```json
{
  "command": "analyze",
  "success": true,
  "sample_size": 50,
  "population_size": 10000,
  "min_value": 0.0,
  "max_value": 100.0,
  "scaled_data": [0.125, 0.142, 0.118, ...],
  "params_min": [2.5, 3.1],
  "params_max": [2.8, 3.4],
  "predicted_params": [2.65, 3.25],
  "sampling_params": [2.4, 3.0],
  "chi2_min": {"chi2": 5.2, "p_value": 0.39, "reject_null": false, ...},
  "chi2_max": {"chi2": 4.8, "p_value": 0.44, "reject_null": false, ...},
  "chi2_pred": {"chi2": 3.1, "p_value": 0.68, "reject_null": false, ...}
}
```

**Client stores:** `scaled_data`, `params_min`, `params_max`, `predicted_params`, `sampling_params`

---

### `get_intervals`

Hypergeometric confidence interval curves.

**Request:**
```json
{
  "command": "get_intervals",
  "distribution": 0,
  "scaled_data": [...],
  "population_size": 10000
}
```

**Response:**
```json
{
  "command": "get_intervals",
  "success": true,
  "domain": [0.0, 0.01, 0.02, ..., 1.0],
  "cdf_min": [1.0, 0.98, 0.95, ...],
  "cdf_max": [1.0, 0.99, 0.97, ...]
}
```

---

### `get_cdf`

CDF curves for fitted/predicted parameters.

**Request:**
```json
{
  "command": "get_cdf",
  "distribution": 0,
  "params_min": [2.5, 3.1],
  "params_max": [2.8, 3.4],
  "predicted_params": [2.65, 3.25],
  "sampling_params": [2.4, 3.0]
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `params_min` | [f64; 2] | No | CI lower bound fit |
| `params_max` | [f64; 2] | No | CI upper bound fit |
| `predicted_params` | [f64; 2] | No | XGBoost prediction |
| `sampling_params` | [f64; 2] | No | Method of moments |

**Response:**
```json
{
  "command": "get_cdf",
  "success": true,
  "domain": [0.0, 0.01, ..., 1.0],
  "fitted_cdf_min": [...],
  "fitted_cdf_max": [...],
  "predicted_cdf": [...],
  "sampling_cdf": [...]
}
```

---

### `get_pdf`

PDF curves for fitted/predicted parameters.

**Request:**
```json
{
  "command": "get_pdf",
  "distribution": 0,
  "params_min": [2.5, 3.1],
  "params_max": [2.8, 3.4],
  "predicted_params": [2.65, 3.25],
  "sampling_params": [2.4, 3.0]
}
```

**Response:**
```json
{
  "command": "get_pdf",
  "success": true,
  "domain": [0.0, 0.01, ..., 1.0],
  "fitted_pdf_min": [...],
  "fitted_pdf_max": [...],
  "predicted_pdf": [...],
  "sampling_pdf": [...]
}
```

---

### `get_histogram`

Histogram with observed and expected frequencies.

**Request:**
```json
{
  "command": "get_histogram",
  "distribution": 0,
  "scaled_data": [...],
  "bins": 15,
  "params_min": [2.5, 3.1],
  "params_max": [2.8, 3.4],
  "predicted_params": [2.65, 3.25]
}
```

**Response:**
```json
{
  "command": "get_histogram",
  "success": true,
  "bin_edges": [0.0, 0.066, 0.133, ..., 1.0],
  "observed_freq": [3, 5, 8, ...],
  "expected_freq_min": [2.8, 5.2, 7.9, ...],
  "expected_freq_max": [3.1, 5.5, 8.2, ...],
  "expected_freq_pred": [2.9, 5.3, 8.0, ...],
  "chi2_min": {...},
  "chi2_max": {...},
  "chi2_pred": {...}
}
```

---

## Statistical Methods

### Distribution Types

| ID | Type | Parameters | Domain |
|----|------|------------|--------|
| 0 | Beta | α (shape1), β (shape2) | [0, 1] |
| 1 | Normal | μ (mean), σ (std dev) | [-0.5, 1.5] |

### Parameter Fitting

1. **Method of Moments** (`sampling_params`)
   - Beta: α = μ·k, β = (1-μ)·k where k = μ(1-μ)/σ² - 1
   - Normal: μ = mean, σ = std_dev

2. **CDF Fitting** (`params_min`, `params_max`)
   - Nelder-Mead optimization to fit CI bounds
   - *TODO: Currently uses method of moments as placeholder*

3. **XGBoost Prediction** (`predicted_params`)
   - Input: [params_min[0], params_min[1], params_max[0], params_max[1]]
   - Output: [param1, param2] - optimal population parameters

### Confidence Intervals

Hypergeometric distribution-based quality intervals:

```
For each outcome k in sample of size n from population N:
  P(K=k | N, M, n) = C(M,k) · C(N-M, n-k) / C(N,n)
  
Quality interval: {M : P(k|M) ≥ max(P)/threshold_factor}
```

### Chi-Square Test

Goodness-of-fit test:

```
χ² = Σ (Oᵢ - Eᵢ)² / Eᵢ

df = num_bins - 1 - 2  (subtract estimated parameters)
reject_null = χ² > χ²_critical(α, df)
```

---

## Module Reference

### `stats.rs`

| Function | Signature | Description |
|----------|-----------|-------------|
| `conf_int` | `(pop_size, samp_size, threshold) → (cdf_min, cdf_max)` | Hypergeometric CI |
| `cdf` | `(kind, domain, params) → Vec<f64>` | CDF values |
| `survival_cdf` | `(kind, domain, params) → Vec<f64>` | 1 - CDF |
| `pdf` | `(kind, domain, params) → Vec<f64>` | PDF values |
| `chi_square_test` | `(observed, expected, α) → ChiSquareResult` | GoF test |
| `method_of_moments` | `(kind, data) → [f64; 2]` | Parameter estimation |
| `bin_edges` | `(start, end, num_bins) → Vec<f64>` | Histogram edges |
| `frequencies` | `(bins, data) → Vec<f64>` | Observed counts |
| `expected_freq` | `(kind, params, bins, n) → Vec<f64>` | Expected counts |

### `xgb.rs`

| Function | Signature | Description |
|----------|-----------|-------------|
| `init` | `() → Result<()>` | Initialize xgbwrapper |
| `cleanup` | `()` | Release resources |
| `predict` | `(features, model_path) → Result<[f32; 2]>` | Run inference |

---

## Error Handling

All responses include `success: bool` and `message: Option<String>`.

| Error | Cause |
|-------|-------|
| `"Invalid distribution type: X"` | distribution not 0 or 1 |
| `"Data is empty"` | empty data array |
| `"Data contains NaN or infinite values"` | invalid numbers |
| `"min_value must be less than max_value"` | invalid bounds |
| `"No model found for sample size"` | missing model file |
| `"Prediction failed: ..."` | xgbwrapper error |
| `"scaled_data required"` | missing for get_intervals/histogram |

---

## Build & Run

### Build Modes

| Mode | Command | TLS | Use Case |
|------|---------|-----|----------|
| Debug | `cargo build` | Disabled (HTTP) | Local development |
| Release | `cargo build --release` | **Required** (HTTPS) | Production |

### Using run.sh (Recommended)

```bash
# Debug mode (HTTP only, TLS config ignored)
./run.sh debug

# Release mode (HTTPS required, TLS must be configured)
./run.sh release

# With custom config
./run.sh debug /path/to/config.yaml
```

The script sets `LD_LIBRARY_PATH` for xgbwrapper and runs the server.

### Manual Build

```bash
# Debug build (HTTP only, TLS config ignored)
cargo build
export LD_LIBRARY_PATH="../lib:$LD_LIBRARY_PATH"
./target/debug/server

# Release build (HTTPS required, TLS must be configured)
cargo build --release
export LD_LIBRARY_PATH="../lib:$LD_LIBRARY_PATH"
./target/release/server
```

**Release mode** will exit with error if `tls` section is missing from config.yaml.

### Custom Config

```bash
./target/release/server /path/to/config.yaml
```

### Dependencies

- **axum 0.8** - Web framework
- **axum-server 0.8** - TLS support (rustls)
- **tokio 1.43** - Async runtime
- **statrs 0.18** - Statistical distributions
- **libxgbwrapper.so** - XGBoost C wrapper (see xgbwrapper/)

### Environment

```bash
# Library path for xgbwrapper
export LD_LIBRARY_PATH=/path/to/lib:$LD_LIBRARY_PATH
```

---

## Future Enhancements

- [ ] Nelder-Mead CDF fitting (replace method of moments placeholder)
- [ ] MessagePack binary protocol option
- [ ] Connection authentication
- [ ] Rate limiting
- [ ] Health check endpoint
