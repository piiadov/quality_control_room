<script setup>
import { onMounted, onUnmounted } from 'vue';
import { sidebarStore, defectsResultsStore } from '../../store/index.js';

const defectsResults = defectsResultsStore();
const sidebar = sidebarStore();

onMounted(() => {
  sidebar.activeTool = "DefectsRate";
  if (defectsResults.showResults === true) {
    sidebar.sidebarResults = true;
  }
});

onUnmounted(() => {
  sidebar.sidebarResults = false;
});


</script>

<template>
  <div>
    <h1 class="text-2xl font-bold">Defects Rate Analyzer</h1>
    <p>Analyze defect rates efficiently.</p>
  </div>
</template>
