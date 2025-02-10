import { createI18n } from 'vue-i18n';
import enUS from '../locales/en-us.json';
import ptBR from '../locales/pt-br.json';

const messages = {
    'en-us': enUS,
    'pt-br': ptBR
};

const i18n = createI18n({
    legacy: false,
    locale: localStorage.getItem('language') || 'en-us', // Default locale
    fallbackLocale: 'en-us',
    messages,
});

export default i18n;
