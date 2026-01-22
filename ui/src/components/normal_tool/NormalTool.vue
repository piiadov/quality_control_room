<script setup>
import { onMounted } from 'vue';
import { useSidebarStore, useNormalStore } from '../../store';

const normal = useNormalStore();
const sidebar = useSidebarStore();

onMounted(() => {
  sidebar.activeTool = normal;
});

</script>

<template>
  <div>
    <h1 class="text-2xl font-bold">Quality Profile using Normal Distribution</h1>
  </div>

  <div class="mt-6 p-4 bg-yellow-100 border-l-4 border-yellow-500 text-yellow-700 rounded">
    <p>
      <strong>Coming soon!</strong> This tool is under development.<br>
      In the meantime, try <a href="/tools/beta-profile" class="text-blue-600 underline">Beta Profiling Tool</a>.
    </p>
  </div>

</template>
