import { createRouter, createWebHistory } from 'vue-router';

import Home from '../views/Home.vue';
import Help from '../views/Help.vue';
import Dashboard from '../components/Dashboard.vue';
import DefectsRate from '../components/defects_rate/DefectsRate.vue';
import BetaTool from "../components/beta_tool/BetaTool.vue";
import NormalTool from "../components/normal_tool/NormalTool.vue";

const routes = [
    { path: '/', component: Home },
    { path: '/help', component: Help },
    {
        path: '/tools',
        component: Dashboard,
        children: [
            { path: 'defects-rate', component: DefectsRate },
            { path: 'beta-profile', component: BetaTool },
            { path: 'normal-profile', component: NormalTool },
        ],
    },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;
