<script setup>
import { onMounted } from 'vue';
import { sidebarStore, betaResultsStore } from '../../store/index.js';
import Inputs from './Inputs.vue';
import Cdf from './Cdf.vue';
import Pdf from './Pdf.vue';

const sidebar = sidebarStore();
const betaResults = betaResultsStore();

onMounted(() => {
  sidebar.activeTool = "BetaProfile";
});

</script>

<template>
  <header class="text-left mb-2 text-xl font-semibold text-text p-4">
    <h1>Quality Profile using Beta-Distribution</h1>
  </header>

  <main class="flex flex-1 justify-start gap-4 p-4">
    <Inputs />
    <Cdf v-if="betaResults.showResults"/>
    <Pdf v-if="betaResults.showResults"/>
  </main>
</template>
