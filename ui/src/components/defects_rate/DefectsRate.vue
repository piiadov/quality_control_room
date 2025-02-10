<script setup>
import { onMounted } from 'vue';
import { useSidebarStore } from '../../store/index.js';

const sidebarStore = useSidebarStore();

onMounted(() => {
  sidebarStore.activeTool = "DefectsRate";
});

</script>

<template>
  <div>
    <h1 class="text-2xl font-bold">Defects Rate Analyzer</h1>
    <p>Analyze defect rates efficiently.</p>
  </div>
</template>

<style scoped>
</style>