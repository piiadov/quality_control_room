<script setup>
import { onMounted, onUnmounted } from 'vue';
import { sidebarStore, betaStore } from '../../store/index.js';
import Inputs from './Inputs.vue';
import Cdf from './Cdf.vue';
import Pdf from './Pdf.vue';
import DistributionParams from './DistributionParams.vue';
import ChiSquared from './ChiSquared.vue';
import Freq from './Freq.vue';

const sidebar = sidebarStore();
const beta = betaStore();

onMounted(() => {
  sidebar.activeTool = "BetaProfile";
  if (beta.showResults === true) {
    sidebar.sidebarResults = true;
  }
});

onUnmounted(() => {
  sidebar.sidebarResults = false;
});

</script>

<template>
  <header class="text-left mb-2 text-xl font-semibold text-text p-4">
    <h1>Quality Profile using Beta-Distribution</h1>
  </header>
  <main class="flex flex-1 flex-wrap justify-start gap-4 p-4">
    <Inputs style="width: 500px; height: 500px;"/>
    <Freq v-if="beta.showResults" style="width: 500px; height: 500px;"/>
    <Cdf v-if="beta.showResults" style="width: 500px; height: 500px;"/>
    <Pdf v-if="beta.showResults" style="width: 500px; height: 500px;"/>
    <DistributionParams v-if="beta.showResults" style="width: 500px; height: 500px;"/>
    <ChiSquared v-if="beta.showResults" style="width: 500px; height: 500px;"/>
  </main>
</template>
