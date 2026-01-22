import { defineStore } from "pinia";

export const useThemeStore = defineStore("theme", {
    state: () => ({
        currentTheme: localStorage.getItem('theme') || 'dark',
    }),
    actions: {
        applyTheme() {
            document.documentElement.setAttribute("data-theme", this.currentTheme);
        },
        toggleTheme() {
            this.currentTheme = this.currentTheme === 'light' ? 'dark' : 'light';
            document.documentElement.setAttribute('data-theme', this.currentTheme);
            localStorage.setItem('theme', this.currentTheme);
        },
    },
});
