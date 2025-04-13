<script setup>
  import {betaResultsStore, betaInputStore} from "../../store/index.js";
  import { Chart, registerables } from 'chart.js';
  import {ref} from "vue";

  const betaInputs = betaInputStore();
  const betaResults = betaResultsStore();

  Chart.register(...registerables);
  const freqRef = ref(null);
  let freq = null;




</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <canvas ref="freqRef" style="width: 500px; height: 500px;"></canvas>
  </div>
</template>
