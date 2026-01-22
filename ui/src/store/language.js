import { defineStore } from "pinia";
import i18n from '../services/i18n';

export const useLanguageStore = defineStore('language', {
    state: () => ({
        currentLanguage: localStorage.getItem('language') || 'en-us',
    }),
    actions: {
        toggleLanguage() {
            this.currentLanguage = this.currentLanguage === 'en-us' ? 'pt-br' : 'en-us';
            localStorage.setItem('language', this.currentLanguage);
            i18n.global.locale.value = this.currentLanguage;
        },
    },
});
