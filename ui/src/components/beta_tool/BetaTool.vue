<script setup>
import { onMounted } from 'vue';
import { sidebarStore, betaResultsStore } from '../../store/index.js';
import Inputs from './Inputs.vue';
import Cdf from './Cdf.vue';
import Pdf from './Pdf.vue';
import Params from './Params.vue';

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
  <main class="flex flex-1 flex-wrap justify-start gap-4 p-4">
    <Inputs style="width: 500px; height: 500px;"/>
    <Cdf v-if="betaResults.showResults" style="width: 500px; height: 500px;"/>
    <Pdf v-if="betaResults.showResults" style="width: 500px; height: 500px;"/>
    <Params v-if="betaResults.showResults" style="width: 500px; height: 300px;"/>
  </main>
</template>
