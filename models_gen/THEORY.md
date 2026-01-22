# Models Generator - Theoretical Foundation

> **Priority Note**: This document is the authoritative reference for models_gen logic.
> It takes precedence over any conflicting information in outer project documentation.

## Overview

The models_gen project generates XGBoost regression models that predict distribution 
parameters from small samples. The key insight is mapping discrete hypergeometric 
sampling statistics to continuous Beta/Normal CDF parameters.

## Problem Statement

Given:
- A finite population of N items (e.g., a production batch)
- Each item has a quality parameter x (e.g., humidity, weight, dimension)
- The quality parameter follows some distribution (Beta or Normal)
- We can only afford to sample k items from the batch

Goal:
- Estimate the true distribution parameters (α, β) or (μ, σ) from k samples
- Provide confidence bounds on these estimates

## Core Concepts

### 1. Hypergeometric Sampling Model

When sampling k items without replacement from a population of N items containing K 
"successes" (items meeting some criterion), the number of successes in the sample 
follows a hypergeometric distribution:

$$P(X = x) = \frac{\binom{K}{x} \binom{N-K}{k-x}}{\binom{N}{k}}$$

### 2. Quality Intervals

For each observed number of successes x in a sample of size k, we can compute 
confidence bounds on the true population proportion K/N.

The `quality_interval(N, k, x)` function returns (lo, hi) bounds representing the 
plausible range for the true population quality proportion.

**Key property**: Quality intervals are monotonic in x:
- More successes → higher estimated quality proportion
- Fewer successes → lower estimated quality proportion

### 3. Order Statistics Interpretation

When we sort k sampled values, each order statistic X₍ᵢ₎ provides information about 
the underlying distribution:

| Order Statistic | Interpretation | Quality Interval |
|-----------------|----------------|------------------|
| X₍₁₎ (smallest) | ~(k) successes "above" this value | HIGH survival prob |
| X₍₂₎ | ~(k-1) successes "above" | slightly lower |
| ... | ... | ... |
| X₍ₖ₎ (largest) | ~(1) success "above" this value | LOW survival prob |

### 4. Natural Boundary Anchors

For any continuous CDF with bounded support, we add anchor points:

**Beta Distribution** (support [0, 1]):
- At x = 0: S = 1.0 (everything is ≥ 0)
- At x = 1: S = 0.0 (nothing is > 1)

**Normal Distribution** (practical support [μ-2σ, μ+2σ]):
- At x = μ-2σ: S ≈ 1.0
- At x = μ+2σ: S ≈ 0.0

These anchors help constrain the CDF fitting and prevent extrapolation artifacts.

## Data Generation Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  1. PARAMETER GRID GENERATION                                               │
│                                                                             │
│     For each (p1, p2) in parameter bounds:                                  │
│       Create dist = Distribution(p1, p2)                                    │
│       Repeat dist_train_size times → Y targets                              │
│                                                                             │
│     Example: Beta with bounds [[0.1,10], [0.1,10]], resolution [10,10]     │
│              → 100 parameter pairs × 10 repeats = 1000 distributions        │
└─────────────────────────────────────────────────────────────────────────────┘
                                      ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  2. CONFIDENCE INTERVAL COMPUTATION                                         │
│                                                                             │
│     conf_int(population_size, sample_size) → (cdf_min, cdf_max)            │
│                                                                             │
│     For sample_size = 5:                                                    │
│       cdf_min = [1.0, q(k=5), q(k=4), q(k=3), q(k=2), q(k=1), 0.0]         │
│                  ↑                                              ↑           │
│               anchor                                         anchor         │
│                                                                             │
│     These are the Y-values for interpolation (survival probabilities)       │
└─────────────────────────────────────────────────────────────────────────────┘
                                      ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  3. SAMPLING AND SORTING                                                    │
│                                                                             │
│     For each distribution:                                                  │
│       samples = [anchor_lo] ∪ SORT(sample(dist, k)) ∪ [anchor_hi]          │
│                                                                             │
│     Example: samples = [0.0, 0.23, 0.45, 0.51, 0.67, 0.89, 1.0]            │
│                         ↑                                   ↑               │
│                      anchor                              anchor             │
│                                                                             │
│     Sorting doesn't change probabilities - only sequence of independent     │
│     samples. The sorted values become X-coordinates for interpolation.      │
└─────────────────────────────────────────────────────────────────────────────┘
                                      ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  4. INTERPOLATION                                                           │
│                                                                             │
│     interp_slice(samples, cdf_min, domain) → cdf_min_interp                │
│     interp_slice(samples, cdf_max, domain) → cdf_max_interp                │
│                                                                             │
│     This maps: (sample_values, quality_bounds) → regular domain grid        │
│                                                                             │
│     Input:  7 points (samples[i], cdf_min[i])                              │
│     Output: 101 points on domain [0, 1] for Beta                           │
└─────────────────────────────────────────────────────────────────────────────┘
                                      ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  5. CDF CURVE FITTING (Nelder-Mead)                                         │
│                                                                             │
│     fit_cdf(domain, cdf_min_interp, kind) → [α_min, β_min]                 │
│     fit_cdf(domain, cdf_max_interp, kind) → [α_max, β_max]                 │
│                                                                             │
│     Minimizes MSE between theoretical survival CDF and interpolated bounds  │
│                                                                             │
│     Objective: min_θ Σᵢ (S_θ(xᵢ) - target[i])²                             │
│     where S_θ(x) = 1 - F_θ(x) is the survival function                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                      ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  6. FEATURE EXTRACTION                                                      │
│                                                                             │
│     X features = [α_min, β_min, α_max, β_max]  (4 values per sample)       │
│     Y targets  = [α_true, β_true]              (2 values per sample)       │
│                                                                             │
│     These become training data for XGBoost regression                       │
└─────────────────────────────────────────────────────────────────────────────┘
                                      ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  7. XGBOOST TRAINING                                                        │
│                                                                             │
│     Train XGBoost regressor: X[N×4] → Y[N×2]                               │
│                                                                             │
│     Save models: beta_n{sample_size}.xgb, normal_n{sample_size}.xgb        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Mathematical Properties

### Monotonicity of Quality Intervals

The quality interval bounds form monotonically decreasing sequences:

```
cdf_min: 1.0 ≥ q(k=n) ≥ q(k=n-1) ≥ ... ≥ q(k=1) ≥ 0.0
cdf_max: 1.0 ≥ q(k=n) ≥ q(k=n-1) ≥ ... ≥ q(k=1) ≥ 0.0
```

This matches the monotonically decreasing nature of survival CDFs, enabling 
meaningful CDF fitting.

### Survival Function vs CDF

We use the **survival function** S(x) = 1 - F(x) = P(X > x) because:

1. It directly maps to "proportion of population above threshold x"
2. Quality control often asks "what proportion meets minimum standard?"
3. The hypergeometric quality interpretation aligns naturally

### Why 4 Features from 2 Bounds?

Fitting both cdf_min and cdf_max captures uncertainty:

- **cdf_min fit** → parameters for conservative (lower bound) estimate
- **cdf_max fit** → parameters for optimistic (upper bound) estimate

The XGBoost model learns to map this uncertainty envelope to the true parameters.

## Configuration Parameters

| Parameter | Description | Typical Value |
|-----------|-------------|---------------|
| `population_size` | N in hypergeometric model | 10000 |
| `sample_sizes` | k values to generate models for | [5, 10, 20, 50, 100] |
| `params_resolution` | Grid resolution for (p1, p2) | [10, 10] |
| `dist_train_size` | Samples per parameter pair | 10 |

Total training rows = `params_resolution[0] × params_resolution[1] × dist_train_size`

## File Structure

```
models_gen/
├── THEORY.md           ← This document (authoritative)
├── config.yaml         ← Training configuration
├── src/
│   ├── main.rs         ← Entry point, orchestration
│   ├── config.rs       ← YAML config parsing
│   ├── datagen.rs      ← Data generation (this theory)
│   └── xgb.rs          ← XGBoost FFI bindings
└── models/             ← Output directory for .xgb files
```
