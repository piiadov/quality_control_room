import { defineStore } from "pinia";

export const useSettingsStore = defineStore('settings', {
    state: () => ({
        backendUrl: 'ws://localhost:8081/quality',
        connectTimeout: 5000,
    }),
});
