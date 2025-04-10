import { createApp } from 'vue'
import { createPinia } from 'pinia';
import '@fortawesome/fontawesome-free/css/all.css';
import 'flag-icons/css/flag-icons.min.css';
import './style.css'
import App from './App.vue'
import router from './router';
import {useThemeStore} from "./store/index.js";
import i18n from './services/i18n.js';

const app = createApp(App);

app.use(router);
app.use(createPinia());
app.use(i18n);

useThemeStore().applyTheme();

app.mount('#app');
