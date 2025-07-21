import { defineStore } from "pinia";
import i18n from '../services/i18n';
import JSZip from "jszip";

export const settingsStore = defineStore('settings', {
    state: () => ({
        backendUrl: 'wss://191.252.60.9:8081/quality',
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
    kind: 0,
    errorMessage: "",
    testMode: false,
    inputDisabled: false,
    batchVolume: NaN,
    samplingData: [],
    minValue: NaN,
    maxValue: NaN,
    binsNumber: NaN,
    showResults: false,
    info: "",
    scaledData: [],
    cdfMin: [],
    cdfMax: [],
    q: [],
    fittedCdfMin: [],
    fittedCdfMax: [],
    fittedPdfMin: [],
    fittedPdfMax: [],
    paramsMin: [0.0, 0.0],
    paramsMax: [0.0, 0.0],
    predictedParams: [0.0, 0.0],
    predictedCdf: [],
    predictedPdf: [],
    testModeParams: [0.0, 0.0],
    testModeCdf: [],
    testModePdf: [],
    bins: [],
    freq: [],
    predictedChi2: 0.0,
    predictedPval: 0.0,
    minChi2: 0.0,
    minPval: 0.0,
    maxChi2: 0.0,
    maxPval: 0.0,
    testModeChi2: 0.0,
    testModePval: 0.0,
    critVal: 0.0,
    minDecision: false,
    maxDecision: false,
    predictedDecision: false,
    testModeDecision: false,
    samplingMean: 0.0,
    samplingStd: 0.0,
    samplingParams: [0.0, 0.0],
    samplingCdf: [],
    samplingPdf: [],
    samplingChi2: -1.0,
    samplingPval: -1.0,
    samplingDecision: false,
    samplingMeanNative: 0.0,
    samplingStdNative: 0.0,
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
                    { key: "Batch Volume (Discretization)", value: this.batchVolume },
                    { key: "Min Value", value: this.minValue },
                    { key: "Max Value", value: this.maxValue },
                    { key: "Bins Number", value: this.binsNumber },
                    { key: "Min Quality Beta Parameters", value: this.paramsMin },
                    { key: "Max Quality Beta Parameters", value: this.paramsMax },
                    { key: "Predicted Beta Parameters", value: this.predictedParams },
                    ...(this.testMode ? [
                        { key: "Test Mode Beta Parameters", value: this.testModeParams }
                    ] : []),
                    { key: "Critical Value", value: this.critVal },
                    { key: "Predicted Chi2", value: this.predictedChi2 },
                    { key: "Predicted Pval", value: this.predictedPval },
                    { key: "Predicted Decision", value: this.predictedDecision ? 'Accept' : 'Reject' },
                    { key: "Min Quality Chi2", value: this.minChi2 },
                    { key: "Min Quality Pval", value: this.minPval },
                    { key: "Min Quality Decision", value: this.minDecision ? 'Accept' : 'Reject' },
                    { key: "Max Quality Chi2", value: this.maxChi2 },
                    { key: "Max Quality Pval", value: this.maxPval },
                    { key: "Max Quality Decision", value: this.maxDecision ? 'Accept' : 'Reject' },
                    ...(this.testMode ? [
                        { key: "True Quality Chi2", value: this.testModeChi2 },
                        { key: "True Quality Pval", value: this.testModePval },
                        { key: "True Quality Decision", value: this.testModeDecision ? 'Accept' : 'Reject' }
                    ] : []),
                ];
                const csvResultInfo = resultInfo.map(e => e.key + "," + e.value).join("\n");

                const freqHist = this.bins.map((bin, index) => ({
                    bin: bin,
                    frequency: this.freq[index],
                }));
                const csvFreqHist = [
                    "Bin,Frequency",
                    freqHist.slice(0, -1).map(e => e.bin + "," + e.frequency).join("\n")
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

                const resultData = this.q.map((value, index) => ({
                    q: value,
                    fittedCdfMin: this.fittedCdfMin[index],
                    fittedCdfMax: this.fittedCdfMax[index],
                    fittedPdfMin: this.fittedPdfMin[index],
                    fittedPdfMax: this.fittedPdfMax[index],
                    predictedCdf: this.predictedCdf[index],
                    predictedPdf: this.predictedPdf[index],
                    ...(this.testMode ? {
                        testModeCdf: this.testModeCdf[index],
                        testModePdf: this.testModePdf[index],
                    } : {}),
                }));
                const csvResultDataContent = resultData.map(e => 
                    e.q + "," + 
                    e.fittedCdfMin + "," + 
                    e.fittedCdfMax + "," + 
                    e.fittedPdfMin + "," + 
                    e.fittedPdfMax + "," + 
                    e.predictedCdf + "," + 
                    e.predictedPdf + "," + 
                    (this.testMode ? (
                        e.testModeCdf + "," + 
                        e.testModePdf)
                    : "")
                ).join("\n");
                
                const csvResultData = [
                    "x,CDF Min Quality,CDF Max Quality,PDF Min Quality,PDF Max Quality,CDF Predicted Quality,PDF Predicted Quality,CDF True Quality,PDF True Quality",
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

export const betaStore = createStore('beta',  {kind: 0, testMode: true});
export const normalStore = createStore('normal', {kind: 1});
export const defectsStore = createStore('defects', {kind: 2});
