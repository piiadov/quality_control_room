import { defineStore } from "pinia";

export const useSidebarStore = defineStore('sidebar', {
    state: () => ({
        activeTool: undefined,
        sidebarResults: false,
    }),
});
