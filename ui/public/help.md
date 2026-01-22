# Quality Control Room - Help

Welcome to the Quality Control Room application. This statistical tool helps you estimate true distribution parameters from small samples of quality control measurements using machine learning.

---

## Table of Contents

1. [Overview](#overview)
2. [Theoretical Foundation](#theoretical-foundation)
3. [How the Server Works](#how-the-server-works)
4. [Using the User Interface](#using-the-user-interface)
5. [Understanding the Results](#understanding-the-results)
6. [Tips and Best Practices](#tips-and-best-practices)

---

## Overview

### The Problem

In quality control, you often face this scenario:
- You have a **large production batch** of N items (e.g., 10,000 units)
- Each item has a **quality parameter** x (humidity, weight, dimension, etc.)
- You can only afford to **sample a small number** k of items (5-100)
- You need to estimate the **true distribution** of quality across the entire batch

### The Solution

Quality Control Room uses a combination of:
1. **Hypergeometric confidence intervals** to bound the true distribution
2. **CDF curve fitting** to estimate distribution parameters
3. **XGBoost machine learning** to predict true parameters from confidence bounds

This approach provides not just point estimates, but also confidence intervals on your parameter estimates.

---

## Theoretical Foundation

### Hypergeometric Sampling Model

When sampling k items **without replacement** from a population of N items, the number of "successes" follows a hypergeometric distribution:

$$P(X = x) = \frac{\binom{K}{x} \binom{N-K}{k-x}}{\binom{N}{k}}$$

Where:
- N = total population size
- K = number of "successes" in population
- k = sample size
- x = observed successes in sample

### Quality Intervals

For each sorted sample value, we compute **confidence bounds** on the true population proportion that would produce such an observation. These bounds form the basis for CDF fitting.

**Key insight**: When you sort k sampled values, each order statistic provides information about the underlying distribution:

| Order Statistic | Interpretation |
|-----------------|----------------|
| X₍₁₎ (smallest) | ~k values are "above" this point |
| X₍₂₎ (2nd smallest) | ~(k-1) values are "above" |
| ... | ... |
| X₍ₖ₎ (largest) | ~1 value is "above" this point |

### Supported Distributions

**Beta Distribution** - for data bounded in [0, 1] or scaled to this range
- Parameters: α (alpha) and β (beta)
- Support: [0, 1]

**Normal Distribution** - for unbounded continuous data
- Parameters: μ (mean) and σ (standard deviation)
- Practical support: [μ-4σ, μ+4σ]

### Survival Function

We use the **survival function** S(x) = 1 - F(x) = P(X > x) because:
1. It directly maps to "proportion of population above threshold x"
2. Quality control often asks "what proportion meets minimum standard?"
3. The hypergeometric quality interpretation aligns naturally

---

## How the Server Works

### Architecture

The server is a Rust application using the Axum web framework with WebSocket communication. It provides:
- Real-time bidirectional communication
- TLS encryption (HTTPS/WSS)
- Concurrent request handling

### Analysis Pipeline

When you submit data for analysis, the server performs these steps:

#### 1. Data Validation & Scaling
```
Input data → Validate → Scale to [0,1] → Sort
```
Your raw measurements are scaled to the [0,1] range using your specified min/max values.

#### 2. Confidence Interval Computation
```
(population_size, sample_size) → quality_intervals → (cdf_min, cdf_max)
```
Using hypergeometric statistics, the server computes lower and upper bounds on the survival CDF.

#### 3. CDF Curve Fitting (Nelder-Mead Optimization)
```
(domain, cdf_bounds) → minimize MSE → fitted_parameters
```
The server fits theoretical CDF curves to both the lower and upper bounds using the Nelder-Mead simplex algorithm.

#### 4. XGBoost Prediction
```
[α_min, β_min, α_max, β_max] → XGBoost model → [α_pred, β_pred]
```
Pre-trained XGBoost models predict the true parameters from the fitted confidence bounds.

#### 5. Chi-Square Goodness-of-Fit
```
(observed_frequencies, expected_frequencies) → χ² statistic → p-value
```
The server evaluates how well each parameter estimate fits the observed data.

### Pre-trained Models

The server includes XGBoost models trained for different sample sizes:
- **beta_n5.xgb** through **beta_n100.xgb** - for Beta distribution
- **normal_n5.xgb** through **normal_n100.xgb** - for Normal distribution

The server automatically selects the closest available model for your sample size.

---

## Using the User Interface

### Main Input Panel

#### Distribution Selection
Choose between **Beta** and **Normal** distributions based on your data characteristics:
- **Beta**: Use for bounded data (percentages, proportions, quality scores 0-100)
- **Normal**: Use for unbounded continuous measurements

#### Population Size
Enter the total size of your production batch. This affects the width of confidence intervals:
- Larger populations → narrower intervals (more precision)
- Smaller populations → wider intervals (more uncertainty)

#### Value Range (Min/Max)
Specify the theoretical bounds of your quality metric:
- For percentages: 0 to 100
- For weights: e.g., 95g to 105g
- For dimensions: e.g., 9.5mm to 10.5mm

#### Sampling Data
Enter your measurements in one of these ways:
1. **Direct input**: Type or paste values separated by commas, spaces, or newlines
2. **File upload**: Click "Load File" to import from a text file

#### Number of Bins
Set the number of histogram bins (default: 20). More bins show finer detail but require more data.

### Test Mode

Toggle **Test Mode** to use server-generated sample data. This is useful for:
- Learning how the tool works
- Validating the analysis pipeline
- Demonstrating capabilities without real data

In Test Mode:
- The server generates random samples from a known distribution (Beta with α=2, β=2)
- You can see how well the algorithm recovers the true parameters
- True parameter values are displayed for comparison

### Running Analysis

1. Fill in all required fields (or enable Test Mode)
2. Click **Analyze**
3. Wait for results (typically < 1 second)
4. View charts and statistics in the results panel

### Results Panel

After analysis, you'll see:

1. **Distribution Parameters Table**
   - Minimum Quality (lower confidence bound)
   - Maximum Quality (upper confidence bound)
   - Predicted (XGBoost estimate)
   - Sampling (method of moments estimate)

2. **CDF Chart** - Shows survival probability curves for all parameter estimates

3. **PDF Chart** - Shows probability density functions

4. **Histogram** - Shows your sample data distribution with fitted curves

5. **Chi-Square Statistics** - Goodness-of-fit results for each estimate

### Language Support

The interface supports multiple languages. Use the language selector in the top navigation to switch between:
- English
- Russian
- Other available translations

---

## Understanding the Results

### Parameter Estimates Comparison

| Estimate | Source | Use When |
|----------|--------|----------|
| **Minimum Quality** | Lower confidence bound CDF fit | Conservative estimate (worst case) |
| **Maximum Quality** | Upper confidence bound CDF fit | Optimistic estimate (best case) |
| **Predicted** | XGBoost ML model | Best single estimate |
| **Sampling** | Method of moments | Quick reference, baseline |

### Reading the Charts

#### CDF Chart (Cumulative Distribution Function)
- X-axis: Quality metric value (scaled to 0-1)
- Y-axis: Probability that a random item exceeds this value
- Multiple curves show different parameter estimates
- Empirical CDF (stepped line) shows actual sample data

#### PDF Chart (Probability Density Function)
- X-axis: Quality metric value (scaled to 0-1)
- Y-axis: Probability density
- Shows the "shape" of the distribution
- Peak indicates the most likely quality value

#### Histogram
- Shows frequency distribution of your samples
- Bars represent binned counts
- Overlay curves show fitted distributions

### Chi-Square Test Interpretation

The chi-square test evaluates how well each fitted distribution matches your data:

| p-value | Interpretation |
|---------|----------------|
| **> 0.10** | Excellent fit |
| **0.05 - 0.10** | Good fit |
| **0.01 - 0.05** | Marginal fit (use caution) |
| **< 0.01** | Poor fit (consider different model) |

**Note**: The chi-square test is sensitive to sample size. With very large samples, even small deviations may be flagged as significant.

---

## Tips and Best Practices

### Sample Size Recommendations

| Sample Size | Reliability | Use Case |
|-------------|-------------|----------|
| 5-10 | Low | Quick screening only |
| 20-30 | Moderate | Standard quality checks |
| 50-100 | High | Critical applications |
| 100+ | Very High | High-stakes decisions |

### Common Pitfalls to Avoid

1. **Data outside range**: Ensure all measurements fall within your min/max values
2. **Too few samples**: Results become unreliable below 5 samples
3. **Wrong distribution**: Beta for bounded data, Normal for unbounded
4. **Ignoring confidence intervals**: Always consider the min/max bounds, not just predictions

### Workflow Recommendations

1. **Start with Test Mode** to familiarize yourself with the interface
2. **Validate your data** before analysis (check for outliers, typos)
3. **Compare all estimates** - if they diverge significantly, investigate your data
4. **Use chi-square results** to validate the fit
5. **Document your analysis** for quality records

---

## Technical Specifications

### Server Requirements
- WebSocket connection on port 8081 (WSS with TLS)
- Rust engine with XGBoost integration
- Pre-loaded distribution models

### Browser Compatibility
- Modern browsers with WebSocket support
- JavaScript enabled
- Recommended: Chrome, Firefox, Safari, Edge

### API Commands
- `about` - Server version information
- `analyze` - Full statistical analysis
- `generate_test_data` - Generate test samples

---
