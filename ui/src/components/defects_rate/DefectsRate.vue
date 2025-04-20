<script setup>
import { onMounted } from 'vue';
import { sidebarStore, defectsStore } from '../../store/index.js';

const defects = defectsStore();
const sidebar = sidebarStore();

onMounted(() => {
  sidebar.activeTool = defects;
});

</script>

<template>
  <div>
    <h1 class="text-2xl font-bold">Defects Rate Analyzer</h1>
    <p>Analyze defect rates efficiently.</p>
  </div>
</template>
