# Quality Control Room - Help

Welcome to the Quality Control Room application. This tool helps you analyze quality metrics using statistical distribution fitting.

## Getting Started

### 1. Input Your Data

- **Population Size**: The total population size for confidence interval calculations
- **Min/Max Values**: The range of your quality metric (e.g., 0-100 for percentages)
- **Sampling Data**: Your measurement data, either typed directly or loaded from a file

### 2. Test Mode

Enable **Test Mode** to use automatically generated sample data. This is useful for:
- Learning how the tool works
- Validating your understanding of the results
- Demonstrating the application

### 3. Analyze

Click the **Analyze** button to perform the statistical analysis. The tool will:
- Fit Beta distribution parameters to your data
- Calculate confidence intervals
- Generate CDF, PDF, and histogram charts
- Perform chi-square goodness-of-fit tests

## Understanding the Results

### Distribution Parameters

| Parameter | Description |
|-----------|-------------|
| Minimum Quality | Lower bound of confidence interval |
| Maximum Quality | Upper bound of confidence interval |
| Predicted | XGBoost model prediction |
| Sampling | Method of moments estimate |

### Charts

- **CDF Chart**: Cumulative Distribution Function showing probability curves
- **PDF Chart**: Probability Density Function showing the distribution shape
- **Histogram**: Frequency distribution of your sample data

### Chi-Square Test

The chi-square test evaluates how well the fitted distribution matches your data:
- **p-value > 0.05**: Good fit (null hypothesis not rejected)
- **p-value ≤ 0.05**: Poor fit (consider different parameters or distribution)

## Tips

1. Use at least 20-30 data points for reliable results
2. Ensure your data falls within the specified min/max range
3. Higher population sizes give narrower confidence intervals

## Support

For questions or issues, please contact the development team.

---

*Quality Control Room v0.1.0*
