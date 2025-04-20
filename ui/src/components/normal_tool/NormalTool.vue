<script setup>
import { onMounted } from 'vue';
import { sidebarStore, normalStore } from '../../store/index.js';

const normal = normalStore();
const sidebar = sidebarStore();

onMounted(() => {
  sidebar.activeTool = normal;
});

</script>

<template>
  <div>
    <h1 class="text-2xl font-bold">Normal Distribution</h1>
  </div>
</template>
