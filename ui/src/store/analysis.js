import { defineStore } from "pinia";
import { exportResultsToZip } from "../utils/export";

/**
 * Default state for analysis stores (Beta, Normal, Defects)
 */
const defaultState = {
    // Distribution type: 0=Beta, 1=Normal
    distribution: 0,
    errorMessage: "",
    testMode: false,
    inputDisabled: false,
    
    // Input parameters
    populationSize: NaN,
    samplingData: [],
    minValue: NaN,
    maxValue: NaN,
    binsNumber: 10,
    
    // UI state
    showResults: false,
    info: "",
    
    // Core analysis results (from analyze command)
    sampleSize: 0,
    scaledData: [],
    paramsMin: [0.0, 0.0],
    paramsMax: [0.0, 0.0],
    predictedParams: [0.0, 0.0],
    samplingParams: [0.0, 0.0],
    
    // Chi-square results (from analyze command)
    chi2Min: { chi2: 0.0, p_value: 0.0, reject_null: false, critical_value: 0.0 },
    chi2Max: { chi2: 0.0, p_value: 0.0, reject_null: false, critical_value: 0.0 },
    chi2Pred: { chi2: 0.0, p_value: 0.0, reject_null: false, critical_value: 0.0 },
    
    // Confidence intervals (from get_intervals command)
    cdfMin: [],
    cdfMax: [],
    
    // CDF/PDF curves (from get_cdf, get_pdf commands)
    domain: [],
    fittedCdfMin: [],
    fittedCdfMax: [],
    fittedPdfMin: [],
    fittedPdfMax: [],
    predictedCdf: [],
    predictedPdf: [],
    samplingCdf: [],
    samplingPdf: [],
    
    // Histogram (from get_histogram command)
    binEdges: [],
    observedFreq: [],
    expectedFreqMin: [],
    expectedFreqMax: [],
    expectedFreqPred: [],
    
    // Test mode (for validation/debugging)
    testModeParams: [0.0, 0.0],
    testModeCdf: [],
    testModePdf: [],
};

/**
 * Factory function to create analysis stores
 */
function createAnalysisStore(storeId, initialState = {}) {
    return defineStore(storeId, {
        state: () => ({
            ...defaultState,
            ...initialState
        }),
        actions: {
            resetState() {
                Object.assign(this.$state, {
                    ...defaultState,
                    ...initialState
                });
            },
            
            /**
             * Update store with API response
             */
            updateFromResult(result) {
                // From analyze command
                this.sampleSize = result.sample_size;
                this.populationSize = result.population_size;
                this.minValue = result.min_value;
                this.maxValue = result.max_value;
                this.scaledData = result.scaled_data;
                this.paramsMin = result.params_min;
                this.paramsMax = result.params_max;
                this.predictedParams = result.predicted_params;
                this.samplingParams = result.sampling_params;
                this.chi2Min = result.chi2_min;
                this.chi2Max = result.chi2_max;
                this.chi2Pred = result.chi2_pred;
                
                // From get_intervals
                this.cdfMin = result.cdf_min;
                this.cdfMax = result.cdf_max;
                
                // From get_cdf
                this.domain = result.domain;
                this.fittedCdfMin = result.fitted_cdf_min;
                this.fittedCdfMax = result.fitted_cdf_max;
                this.predictedCdf = result.predicted_cdf;
                this.samplingCdf = result.sampling_cdf;
                
                // From get_pdf
                this.fittedPdfMin = result.fitted_pdf_min;
                this.fittedPdfMax = result.fitted_pdf_max;
                this.predictedPdf = result.predicted_pdf;
                this.samplingPdf = result.sampling_pdf;
                
                // From get_histogram
                this.binEdges = result.bin_edges;
                this.observedFreq = result.observed_freq;
                this.expectedFreqMin = result.expected_freq_min;
                this.expectedFreqMax = result.expected_freq_max;
                this.expectedFreqPred = result.expected_freq_pred;
            },
            
            /**
             * Update histogram data only
             */
            updateHistogram(result) {
                this.binEdges = result.bin_edges;
                this.observedFreq = result.observed_freq;
                this.expectedFreqMin = result.expected_freq_min;
                this.expectedFreqMax = result.expected_freq_max;
                this.expectedFreqPred = result.expected_freq_pred;
                this.binsNumber = result.bin_edges.length - 1;
                
                if (result.chi2_min) this.chi2Min = result.chi2_min;
                if (result.chi2_max) this.chi2Max = result.chi2_max;
                if (result.chi2_pred) this.chi2Pred = result.chi2_pred;
            },
            
            exportResults() {
                exportResultsToZip(this.$state);
            }
        }
    });
}

// Create stores for each distribution type
export const useBetaStore = createAnalysisStore('beta', { distribution: 0, testMode: true });
export const useNormalStore = createAnalysisStore('normal', { distribution: 1 });
export const useDefectsStore = createAnalysisStore('defects', { distribution: 2 });
