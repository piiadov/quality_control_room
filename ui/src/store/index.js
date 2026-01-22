import { defineStore } from "pinia";
import i18n from '../services/i18n';
import JSZip from "jszip";

export const settingsStore = defineStore('settings', {
    state: () => ({
        backendUrl: 'ws://localhost:8081/quality',
        connectTimeout: 5000,
    }),
})

export const themeStore = defineStore("theme", {
    state: () => ({
        currentTheme: localStorage.getItem('theme') || 'dark',
    }),
    actions: {
        applyTheme() {
            const theme_store = this;
            document.documentElement.setAttribute("data-theme", theme_store.currentTheme);
        },
        toggleTheme() {
            const theme_store = this;
            theme_store.currentTheme = theme_store.currentTheme === 'light' ? 'dark' : 'light';
            document.documentElement.setAttribute('data-theme', theme_store.currentTheme);
            localStorage.setItem('theme', theme_store.currentTheme);
        },
    },
});

export const languageStore = defineStore('language', {
    state: () => ({
        currentLanguage: localStorage.getItem('language') || 'en-us', // Default to English
    }),
    actions: {
        toggleLanguage() {
            const lang_store = this;
            this.currentLanguage = lang_store.currentLanguage === 'en-us' ? 'pt-br' : 'en-us';
            localStorage.setItem('language', lang_store.currentLanguage);
            i18n.global.locale.value = lang_store.currentLanguage;
        },
    },
});

export const sidebarStore = defineStore('sidebar', {
    state: () => ({
        activeTool: undefined,
    }),
});

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

function createStore(storeId, initialState = {}) {
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
            exportResults() {
                const data = this.samplingData.map((item, index) => ({
                    index: index,
                    value: item,
                }));
                const csvData = [
                    "Index,Value",
                    data.map(e => e.index + "," + e.value).join("\n")
                ].join("\n");
                
                const resultInfo = [
                    { key: "Population Size", value: this.populationSize },
                    { key: "Sample Size", value: this.sampleSize },
                    { key: "Min Value", value: this.minValue },
                    { key: "Max Value", value: this.maxValue },
                    { key: "Bins Number", value: this.binsNumber },
                    { key: "Min Quality Parameters", value: this.paramsMin },
                    { key: "Max Quality Parameters", value: this.paramsMax },
                    { key: "Predicted Parameters", value: this.predictedParams },
                    { key: "Sampling Parameters (MoM)", value: this.samplingParams },
                    ...(this.testMode ? [
                        { key: "Test Mode Parameters", value: this.testModeParams }
                    ] : []),
                    { key: "Critical Value", value: this.chi2Pred.critical_value },
                    { key: "Predicted Chi2", value: this.chi2Pred.chi2 },
                    { key: "Predicted P-value", value: this.chi2Pred.p_value },
                    { key: "Predicted Decision", value: this.chi2Pred.reject_null ? 'Reject' : 'Accept' },
                    { key: "Min Quality Chi2", value: this.chi2Min.chi2 },
                    { key: "Min Quality P-value", value: this.chi2Min.p_value },
                    { key: "Min Quality Decision", value: this.chi2Min.reject_null ? 'Reject' : 'Accept' },
                    { key: "Max Quality Chi2", value: this.chi2Max.chi2 },
                    { key: "Max Quality P-value", value: this.chi2Max.p_value },
                    { key: "Max Quality Decision", value: this.chi2Max.reject_null ? 'Reject' : 'Accept' },
                ];
                const csvResultInfo = resultInfo.map(e => e.key + "," + e.value).join("\n");

                const freqHist = this.binEdges.map((bin, index) => ({
                    bin: bin,
                    observed: this.observedFreq[index] ?? '',
                    expectedMin: this.expectedFreqMin[index] ?? '',
                    expectedMax: this.expectedFreqMax[index] ?? '',
                    expectedPred: this.expectedFreqPred[index] ?? '',
                }));
                const csvFreqHist = [
                    "Bin,Observed,Expected Min,Expected Max,Expected Predicted",
                    freqHist.slice(0, -1).map(e => 
                        `${e.bin},${e.observed},${e.expectedMin},${e.expectedMax},${e.expectedPred}`
                    ).join("\n")
                ].join("\n");


                const estimatedCdf = this.scaledData.map((value, index) => ({
                    index: index,
                    scaledData: value,
                    cdfMin: this.cdfMin[index],
                    cdfMax: this.cdfMax[index],
                })).map(e => e.index + "," + e.scaledData + "," + e.cdfMin + "," + e.cdfMax).join("\n");

                const csvEstimatedCdf = [
                    "Index,Scaled Data,CDF Min Quality,CDF Max Quality",
                    estimatedCdf
                ].join("\n");

                const resultData = this.domain.map((value, index) => ({
                    x: value,
                    fittedCdfMin: this.fittedCdfMin[index],
                    fittedCdfMax: this.fittedCdfMax[index],
                    fittedPdfMin: this.fittedPdfMin[index],
                    fittedPdfMax: this.fittedPdfMax[index],
                    predictedCdf: this.predictedCdf[index],
                    predictedPdf: this.predictedPdf[index],
                    samplingCdf: this.samplingCdf[index],
                    samplingPdf: this.samplingPdf[index],
                    ...(this.testMode ? {
                        testModeCdf: this.testModeCdf[index],
                        testModePdf: this.testModePdf[index],
                    } : {}),
                }));
                const csvResultDataContent = resultData.map(e => 
                    e.x + "," + 
                    e.fittedCdfMin + "," + 
                    e.fittedCdfMax + "," + 
                    e.fittedPdfMin + "," + 
                    e.fittedPdfMax + "," + 
                    e.predictedCdf + "," + 
                    e.predictedPdf + "," +
                    e.samplingCdf + "," +
                    e.samplingPdf +
                    (this.testMode ? ("," +
                        e.testModeCdf + "," + 
                        e.testModePdf)
                    : "")
                ).join("\n");
                
                const csvResultData = [
                    "x,CDF Min,CDF Max,PDF Min,PDF Max,CDF Predicted,PDF Predicted,CDF Sampling,PDF Sampling" + 
                        (this.testMode ? ",CDF TestMode,PDF TestMode" : ""),
                    csvResultDataContent
                ].join("\n");

                const zip = new JSZip();
                zip.file("data.csv", csvData);
                zip.file("result_info.csv", csvResultInfo);
                zip.file("frequency_histogram.csv", csvFreqHist);
                zip.file("estimated_cdf.csv", csvEstimatedCdf);
                zip.file("result_data.csv", csvResultData);

                zip.generateAsync({ type: "blob" }).then((content) => {
                    const link = document.createElement("a");
                    link.href = URL.createObjectURL(content);
                    const timestamp = new Date().toISOString().replace(/[-:.]/g, "_");
                    link.download = `quality_room_${timestamp}.zip`;
                    document.body.appendChild(link);
                    link.click();
                    document.body.removeChild(link);
                });
            }
        }
    });
}

export const betaStore = createStore('beta',  {distribution: 0, testMode: true});
export const normalStore = createStore('normal', {distribution: 1});
export const defectsStore = createStore('defects', {distribution: 2});
