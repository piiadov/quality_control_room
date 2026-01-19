# xgbwrapper - Quality Control Room ML Bridge

**Version:** 0.3.0  
**Project:** Quality Control Room  
**Role:** C bridge between Rust engine and XGBoost for distribution parameter prediction

---

## Purpose in Quality Control Room

xgbwrapper is a specialized C library that enables the Rust-based Quality Control Room engine to leverage XGBoost for **predicting population distribution parameters from small samples**.

### The Problem We Solve

In industrial quality control, obtaining large samples is expensive. Given a small sample (5-100 items) from a batch (e.g., 3000 items), we need to estimate the underlying distribution parameters. The Rust engine:

1. Computes **hypergeometric confidence intervals** around the empirical CDF
2. Fits **Beta/Normal distributions** to these bounds via Nelder-Mead optimization
3. Uses **xgbwrapper** to predict optimal final parameters from these fitted features

### Where xgbwrapper Fits

```
┌─────────────────────────────────────────────────────────────────┐
│                 Quality Control Room Engine (Rust)              │
│                                                                 │
│  Sample Data → Confidence Intervals → CDF Fitting → Features   │
│        ↓                                                        │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ features = [α_min, β_min, α_max, β_max, n, N, ...]       │  │
│  └──────────────────────────────────────────────────────────┘  │
│        ↓                                                        │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              engine/models/src/wrapper.rs                 │  │
│  │                    (Rust FFI layer)                       │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↓ FFI calls
┌─────────────────────────────────────────────────────────────────┐
│                      xgbwrapper (C Library)                     │
│                                                                 │
│  xgbw_train()   - Train model on simulated data                │
│  xgbw_predict() - Predict [α, β] or [μ, σ] from features       │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                        libxgboost.so                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Installation

### Quick Install

```bash
cd xgbwrapper
./install.sh              # Release build
./install.sh --debug      # Debug build with symbols
./install.sh --clean      # Full rebuild including XGBoost
```

### Output Location

Libraries are installed to `quality_control_room/lib/`:

```
lib/
├── libxgboost.so         # XGBoost (~17 MB)
├── libdmlc.so            # DMLC dependency (~1.2 MB)
└── libxgbwrapper.so      # This library (~26 KB)
```

---

## API Reference

### Lifecycle

```c
XGBWrapperStatus xgbw_init(void);    // Call once at engine startup
void xgbw_cleanup(void);              // Call at engine shutdown
```

### Training

Used by the engine to train models on labeled parameter data.

```c
XGBWrapperStatus xgbw_train(
    const float* x,              // Feature matrix [n_samples × n_features]
    const float* y,              // Target parameters [n_samples × 2]
                                 //   Beta: (α, β), Normal: (μ, σ)
    int rows,                    // Number of training samples
    int x_cols,                  // Number of features (fitted params + metadata)
    int y_cols,                  // Number of targets (usually 2: α, β)
    const KVPair* config,        // XGBoost hyperparameters
    int len_config,              // Number of config entries
    const char* inference_path   // Output model path (JSON)
);
```

**Feature Vector Layout** (as used in Quality Control Room):

| Index | Feature | Description |
|-------|---------|-------------|
| 0 | α_min | Beta shape fitted to lower CDF bound |
| 1 | β_min | Beta shape fitted to lower CDF bound |
| 2 | α_max | Beta shape fitted to upper CDF bound |
| 3 | β_max | Beta shape fitted to upper CDF bound |
| 4-7 | μ_min, σ_min, μ_max, σ_max | Normal params (if applicable) |
| ... | n, N, ... | Sample size, population size, etc. |

**Target Vector:**
| Index | Beta Distribution | Normal Distribution |
|-------|-------------------|---------------------|
| 0 | α (shape) | μ (mean) |
| 1 | β (shape) | σ (std dev) |

### Prediction

Used during inference to predict distribution parameters from computed features.

```c
XGBWrapperStatus xgbw_predict(
    const float* data,           // Feature matrix [n_samples × n_features]
    int rows,                    // Number of samples to predict
    int x_cols,                  // Number of features
    int y_cols,                  // Number of outputs (2)
    const char* inference_path,  // Trained model path
    float* pred                  // Output predictions [n_samples × 2]
);
```

### Error Handling

All functions return status codes - no `exit()` calls that would crash the Rust engine.

```c
typedef enum {
    XGBW_SUCCESS = 0,
    XGBW_ERROR_INVALID_PARAM,    // NULL pointer, invalid dimensions
    XGBW_ERROR_MEMORY,           // Allocation failed
    XGBW_ERROR_FILE_IO,          // Model file read/write error
    XGBW_ERROR_XGBOOST,          // XGBoost internal error
    XGBW_ERROR_NOT_INITIALIZED,  // xgbw_init() not called
    XGBW_ERROR_SIZE_MISMATCH,    // Prediction size doesn't match
} XGBWrapperStatus;

const char* xgbw_get_last_error(void);  // Thread-local error message
```

### Utilities

```c
// Split data for train/test (used during model training)
XGBWrapperStatus xgbw_split_data(
    const float* x, const float* y,
    float* x_train, float* y_train,
    float* x_test, float* y_test,
    int x_cols, int y_cols, int rows, int rows_train
);

// RMSE calculation for model evaluation
XGBWrapperStatus xgbw_calculate_rmse(
    const float* y_pred, const float* y_test,
    int rows, int y_cols, float* rmse
);
```

---

## Rust Integration

### Current Usage in Engine

The Rust engine calls xgbwrapper through FFI bindings in `engine/models/src/wrapper.rs`:

```rust
// Simplified view of the Rust FFI layer
extern "C" {
    fn xgbw_init() -> i32;
    fn xgbw_predict(
        data: *const f32, rows: i32, x_cols: i32, y_cols: i32,
        model_path: *const c_char, pred: *mut f32
    ) -> i32;
    fn xgbw_cleanup();
}

pub fn predict_parameters(features: &[f32], model: &str) -> Result<(f32, f32), Error> {
    let mut pred = [0.0f32; 2];
    unsafe {
        let status = xgbw_predict(
            features.as_ptr(), 1, features.len() as i32, 2,
            CString::new(model)?.as_ptr(), pred.as_mut_ptr()
        );
        if status != 0 {
            return Err(Error::XGBoostPrediction);
        }
    }
    Ok((pred[0], pred[1]))
}
```

### Build Configuration

In `engine/models/build.rs`:

```rust
fn main() {
    println!("cargo:rustc-link-search=../../lib");
    println!("cargo:rustc-link-lib=xgbwrapper");
    println!("cargo:rustc-link-lib=xgboost");
}
```

Runtime library path:
```bash
export LD_LIBRARY_PATH="/path/to/quality_control_room/lib:$LD_LIBRARY_PATH"
```

---

## Model Configuration

### Recommended Settings for Quality Control Room

```c
KVPair config[] = {
    // Booster
    {"booster", "gbtree"},
    {"objective", "reg:squarederror"},
    {"eval_metric", "rmse"},
    
    // Tree parameters (tuned for parameter prediction)
    {"max_depth", "6"},
    {"min_child_weight", "1"},
    
    // Learning
    {"learning_rate", "0.1"},
    {"n_estimators", "100"},
    
    // Regularization (prevents overfitting on small datasets)
    {"subsample", "0.8"},
    {"colsample_bytree", "0.8"},
    {"reg_alpha", "0.0"},
    {"reg_lambda", "1.0"},
    
    // Performance
    {"nthread", "4"},
    {"verbosity", "0"}
};
```

### Model Files

Models are stored in JSON format for portability:

```
data/
├── beta_model.json      # Trained model for Beta distribution
└── normal_model.json    # Trained model for Normal distribution
```

---

## Thread Safety

The Quality Control Room server handles multiple concurrent requests. xgbwrapper is designed for this:

| Function | Thread-Safe | Notes |
|----------|-------------|-------|
| `xgbw_init()` | ❌ | Call once at server startup |
| `xgbw_cleanup()` | ❌ | Call once at server shutdown |
| `xgbw_predict()` | ✅ | Safe for concurrent API requests |
| `xgbw_get_last_error()` | ✅ | Thread-local storage |

### Server Initialization Pattern

```rust
// In engine/server/src/main.rs
fn main() {
    // Initialize once before starting Warp server
    unsafe { xgbw_init(); }
    
    // Start async server (handles concurrent requests)
    let api = warp::path("api").and(handlers::routes());
    warp::serve(api).run(([0, 0, 0, 0], 3030)).await;
    
    // Cleanup on shutdown
    unsafe { xgbw_cleanup(); }
}
```

---

## Building

### CMake Presets

```bash
cmake --list-presets
# debug          - Full debug symbols, no optimization
# release        - Production build with -O3 -march=native -flto
# asan           - AddressSanitizer for memory debugging
```

### Full Rebuild

```bash
./install.sh --clean
```

### Test Suite

```bash
cd xgbwrapper/build/release
ctest --output-on-failure

# Tests:
# test_shuffle          - Fisher-Yates shuffle correctness
# test_split_data       - Train/test split functionality
# test_generate_data    - Synthetic data generation
# test_xgboost          - Full train → predict → evaluate cycle
```

---

## Data Flow Example

**Scenario:** User submits 50 samples from a batch of 3000 items.

```
1. Vue.js UI → POST /api/calc with sample data

2. Rust Engine receives request:
   - Computes hypergeometric confidence intervals (α=0.05)
   - Fits Beta distribution to cdf_min → [α_min, β_min]
   - Fits Beta distribution to cdf_max → [α_max, β_max]
   - Assembles feature vector

3. Rust calls xgbwrapper:
   xgbw_predict(features, 1, 4, 2, "data/beta_model.json", pred)

4. xgbwrapper:
   - Loads XGBoost model from JSON
   - Creates DMatrix from features
   - Runs XGBoost prediction
   - Returns [α_predicted, β_predicted]

5. Rust Engine:
   - Uses predicted parameters to generate final distribution
   - Computes PDF, CDF, statistics
   - Returns JSON response to UI

6. Vue.js UI:
   - Renders CDF comparison chart
   - Shows predicted vs empirical distribution
```

---

## Extending for Other Uses

While xgbwrapper was built for Quality Control Room, the API is generic enough for other regression tasks:

```c
// Any multi-output regression problem works
xgbw_train(
    features,     // Your feature matrix
    targets,      // Your target values
    n_samples,
    n_features,
    n_outputs,    // Supports multi-output
    config,
    n_config,
    "your_model.json"
);
```

Key design decisions that enable reuse:
- Pure C API with no Quality Control Room dependencies
- Generic float arrays (no hardcoded dimensions)
- Status-code error handling (no crashes)
- Thread-safe operations


