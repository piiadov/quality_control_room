<script setup>
import { onMounted, onUnmounted } from 'vue';
import { sidebarStore, normalStore } from '../../store/index.js';

const normal = normalStore();
const sidebar = sidebarStore();

onMounted(() => {
  sidebar.activeTool = "NormalProfile";
  if (normal.showResults === true) {
    sidebar.sidebarResults = true;
  }
});

onUnmounted(() => {
  sidebar.sidebarResults = false;
});


</script>

<template>
  <div>
    <h1 class="text-2xl font-bold">Normal Distribution</h1>
  </div>
</template>
