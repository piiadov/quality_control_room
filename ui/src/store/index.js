import { defineStore } from "pinia";
import i18n from '../services/i18n';

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

export const settingsStore = defineStore('settings', {
    state: () => ({
        backendUrl: 'ws://localhost:8080/quality',
        connectTimeout: 5000,
    }),
})

export const betaStore = defineStore('beta', {
    state: () => ({
        errorMessage: "",
        testMode: true,
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
        betaParamsMin: [0.0, 0.0],
        betaParamsMax: [0.0, 0.0],
        predictedBetaParams: [0.0, 0.0],
        predictedCdf: [],
        predictedPdf: [],
        testModeBetaParams: [0.0, 0.0],
        testModeCdf: [],
        testModePdf: [],
        bins:[],
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
    }),
})

export const defectsStore = defineStore('defects', {
    state: () => ({        
        testMode: true,
        showResults: false,
    }),
})

export const normalStore = defineStore('normal', {
    state: () => ({        
        testMode: true,
        showResults: false,
    }),
})
