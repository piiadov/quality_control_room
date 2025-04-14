<script setup>
  import { onMounted, onUnmounted, computed, watch } from "vue";
  import { betaResultsStore } from "../../store/index.js";
  import { Chart, registerables } from 'chart.js';
  import {ref} from "vue";

  const betaResults = betaResultsStore();

  Chart.register(...registerables);
  const freqRef = ref(null);
  let freqChart = null;

  const binsNumber = computed(() => {
    return betaResults.bins.length - 1;
  });

  const createHistogram = () => {
    freqChart = new Chart(freqRef.value, {
      type: "bar",
      data: {
        labels: betaResults.bins,
        datasets: [
          {
            label: "Frequency",
            data: betaResults.freq,
            backgroundColor: "rgba(75, 192, 192, 0.2)",
            borderColor: "rgba(75, 192, 192, 1)",
            borderWidth: 1,
          },
        ],
      },
      options: {
        responsive: true,
        scales: {
          x: {
            title: {
              display: true,
              text: "Bins",
            },
          },
          y: {
            title: {
              display: true,
              text: "Frequency",
            },
            beginAtZero: true,
          },
        },
      },
    });
  };

  onMounted(() => {
    if (freqChart === null) {
      createHistogram();
    }
  });

  onUnmounted(() => {
    if (freqChart) {
      freqChart.destroy();
    }
  })

  watch(
    () => betaResults,
    () => {
      if (freqChart) {
      freqChart.data.labels = betaResults.bins;
      freqChart.data.datasets[0].data = betaResults.freq;
      freqChart.update();
      }
    },
    { deep: true }
  );

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <div class="flex items-center space-x-4">
      <label for="binsNumber" class="label-text">Number of Bins:</label>
      <input
        id="binsNumber"
        type="text"
        v-model="binsNumber"
        class="w-20"
      />
      <button
        class="btn btn-primary"
        @click="alert"
      >
      Update
      </button>
    </div>
    <canvas ref="freqRef" style="width: 500px; height: 300px"></canvas>
  </div>
</template>
