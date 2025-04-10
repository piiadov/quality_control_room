import { createRouter, createWebHistory } from 'vue-router';

import Home from '../views/Home.vue';
import About from '../views/About.vue';
import Dashboard from '../components/Dashboard.vue';
import DefectsRate from '../components/defects_rate/DefectsRate.vue';
import BetaTool from "../components/beta_tool/BetaTool.vue";

const routes = [
    { path: '/', component: Home },
    { path: '/about', component: About },
    {
        path: '/tools',
        component: Dashboard,
        children: [
            { path: 'defects-rate', component: DefectsRate },
            { path: 'beta-profile', component: BetaTool},
        ],
    },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;
