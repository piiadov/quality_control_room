import { defineStore } from "pinia";

export const useSettingsStore = defineStore('settings', {
    state: () => ({
        backendUrl: import.meta.env.VITE_BACKEND_URL || 'ws://localhost:8081/quality',
        connectTimeout: 5000,
    }),
});
