import { defineStore } from "pinia";
import i18n from '../services/i18n';

export const useThemeStore = defineStore("theme", {
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

export const useLanguageStore = defineStore('language', {
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

export const useSidebarStore = defineStore('sidebar', {
    state: () => ({
        activeTool: "",
        sidebarResults: false
    }),
});

export const useSettings = defineStore('settings', {
    state: () => ({
        backendUrl: 'ws://localhost:8080/quality',
        connectTimeout: 5000,
    }),
})

export const useQualityProfileInput = defineStore('quality-profile-input', {
    state: () => ({        
        testMode: true,
        batchVolume: "",
        samplingData: [],
        minValue: "",
        maxValue: ""
    }),
})

export const useQualityProfileResults = defineStore('quality-profile-results', {
    state: () => ({
        showResults: false,
        info: "",
        q: []
    }),
})
