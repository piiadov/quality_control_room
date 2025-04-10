<script setup>
import { onMounted, onUnmounted } from 'vue';
import { sidebarStore } from "../../store/index.js";

const sidebar = sidebarStore();

onMounted(() => {
  sidebar.sidebarResults = true;
});

onUnmounted(() => {
  sidebar.sidebarResults = false;
})

</script>

<template>

</template>

<style scoped>

</style>