<script setup>
import { onMounted, onUnmounted } from 'vue';
import { sidebarStore, normalResultsStore } from '../../store/index.js';

const normalResults = normalResultsStore();
const sidebar = sidebarStore();

onMounted(() => {
  sidebar.activeTool = "NormalProfile";
  if (normalResults.showResults === true) {
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
