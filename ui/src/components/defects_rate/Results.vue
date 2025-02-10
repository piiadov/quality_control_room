<script setup>
import { onMounted, onUnmounted } from 'vue';
import {useSidebarStore} from "../../store/index.js";

const sidebarStore = useSidebarStore();

onMounted(() => {
  sidebarStore.sidebarResults = true;
});

onUnmounted(() => {
  sidebarStore.sidebarResults = false;
})

</script>

<template>

</template>

<style scoped>

</style>