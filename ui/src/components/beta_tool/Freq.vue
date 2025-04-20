<script setup>
  import { ref, onMounted, onUnmounted, computed, watch } from "vue";
  import { betaStore } from "../../store/index.js";
  import { Chart, registerables } from 'chart.js';
  import {ArrowPathIcon} from "@heroicons/vue/24/outline/index.js";
  import WebSocketService from '../../services/websocketService.js'
  import { settingsStore } from "../../store/index.js";

  const settings = settingsStore();
  const beta = betaStore();

  Chart.register(...registerables);
  const freqRef = ref(null);
  let freqChart = null;
  const newBinsNumber = ref(null);

  const updateFreq = () => {
    const binsNumber = parseInt(newBinsNumber.value, 10);
    if (isNaN(binsNumber) || binsNumber < 1 || binsNumber > 50) {
      beta.errorMessage = "Please enter a valid number of bins (1-50)";
    } else {
      const savedBinsNumber = beta.binsNumber;
      beta.errorMessage = "";
      beta.inputDisabled = true;
      beta.binsNumber = binsNumber;
      const ws = new WebSocketService(settings.backendUrl, settings.connectTimeout);
      ws.connectAndSendData('update_bins', beta)
      .then(response => {
        if (response.data.error > 0) {
          beta.errorMessage = 'Backend error: ' + response.data.info;
          beta.inputDisabled = false;
          beta.binsNumber = savedBinsNumber;
        } else {
          beta.errorMessage = "";
          beta.inputDisabled = false;
          beta.bins = response.data.bins;
          beta.freq = response.data.freq;
          beta.binsNumber = response.data.bins.length - 1;
          beta.predictedChi2 = response.data.predicted_chi2;
          beta.minChi2 = response.data.min_chi2;
          beta.maxChi2 = response.data.max_chi2;
          beta.testModeChi2 = response.data.test_mode_chi2;
          beta.predictedPval = response.data.predicted_pval;
          beta.minPval = response.data.min_pval;
          beta.maxPval = response.data.max_pval;
          beta.testModePval = response.data.test_mode_pval;
          beta.predictedDecision = response.data.predicted_decision;
          beta.minDecision = response.data.min_decision;
          beta.maxDecision = response.data.max_decision;
          beta.testModeDecision = response.data.test_mode_decision;
          beta.critVal = response.data.crit_val;
        }
      })
      .catch(error => {
        beta.errorMessage = error.message;
        beta.inputDisabled = false;
        beta.binsNumber = savedBinsNumber;
      });

    }
  };

  const binsLabels = computed(() => 
    beta.bins.map(bin => bin.toFixed(2).toString())
  );

  const createHistogram = () => {
    freqChart = new Chart(freqRef.value, {
      type: "bar",
      data: {
        labels: binsLabels.value,
        datasets: [
          {
            label: "Frequency",
            data: beta.freq,
            backgroundColor: "rgba(75, 192, 192, 0.2)",
            borderColor: "rgba(75, 192, 192, 1)",
            borderWidth: 1,
          },
        ],
      },
      options: {
        responsive: true,
        plugins: {
          legend: {
            display: false,
          },
          title: {
            display: true,
            text: "Sampling data counts per bin",
            font: {
              size: 14,
            },
          },
        },
        scales: {
          x: {
            type: "category",
            title: {
              display: true,
              text: "Bins",
            },
            offset: true,
            barPercentage: 1.0,
            categoryPercentage: 1.0,
          },
          y: {
            title: {
              display: true,
              text: "Counts",
            },
            beginAtZero: true,
          },
        },
      },
    });
  };

  onMounted(() => {
    newBinsNumber.value = beta.bins.length - 1;
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
    () => beta,
    () => {
      newBinsNumber.value = beta.bins.length - 1;
      if (freqChart) {
        freqChart.data.labels = binsLabels.value;
        freqChart.data.datasets[0].data = beta.freq;
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
        id="inputBinsNumber"
        type="text"
        v-model="newBinsNumber"
        class="w-20 input-text"
        :disabled="beta.inputDisabled"
      />
      <ArrowPathIcon
        class="h-5 w-5 label-text"
        :class="{ 'muted-link': !beta.inputDisabled, 'muted-link-disabled': beta.inputDisabled }"
        :disabled="beta.inputDisabled"
        @click="() => {beta.inputDisabled ? {} : updateFreq();}"
      />
    </div>
    <div style="position: relative; height: 100%; width: 100%;">
      <canvas ref="freqRef" style="display: block; width: 100%; height: 90%;"></canvas>
    </div>
  </div>
</template>
