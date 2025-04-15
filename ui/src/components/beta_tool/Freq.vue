<script setup>
  import { ref, onMounted, onUnmounted, computed, watch } from "vue";
  import { betaStore } from "../../store/index.js";
  import { Chart, registerables } from 'chart.js';
  import { ArrowPathIcon } from "@heroicons/vue/24/outline/index.js";
  import WebSocketService from '../../services/websocketService.js'
  import { settingsStore } from "../../store/index.js";

  const settings = settingsStore();
  const errorMessage = ref(null);
  const beta = betaStore();

  Chart.register(...registerables);
  const freqRef = ref(null);
  let freqChart = null;
  const newBinsNumber = ref(null);

  const updateFreq = () => {
    const binsNumber = parseInt(newBinsNumber.value, 10);
    if (isNaN(binsNumber) || binsNumber < 1 || binsNumber > 50) {
      errorMessage.value = "Please enter a valid number of bins (1-50)";
      return;
    } else {
      const savedBinsNumber = beta.binsNumber;
      errorMessage.value = null;
      beta.inputDisabled = true;
      beta.binsNumber = binsNumber;
      const ws = new WebSocketService(settings.backendUrl, settings.connectTimeout);
      ws.connectAndSendData('update_bins', beta)
      .then(response => {
        if (response.data.error > 0) {
          errorMessage.value = 'Backend error: ' + response.data.info;
          beta.inputDisabled = false;
          beta.binsNumber = savedBinsNumber;
        } else {
          errorMessage.value = null;
          beta.inputDisabled = false;
          beta.bins = response.data.bins;
          beta.freq = response.data.freq;
          beta.binsNumber = response.data.bins.length - 1;

          console.log(beta.binsNumber, ' bins, beta.bins: ', JSON.stringify(beta.bins));

        }
      })
      .catch(error => {
        errorMessage.value = error.message;
        beta.inputDisabled = false;
        beta.binsNumber = savedBinsNumber;
      });

    }
  };

  const createHistogram = () => {
    freqChart = new Chart(freqRef.value, {
      type: "bar",
      data: {
        labels: beta.bins,
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
      freqChart.data.labels = beta.bins;
      freqChart.data.datasets[0].data = beta.freq;
      freqChart.update();
      }
    },
    { deep: true }
  );

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <div v-if="errorMessage" class="error-message text-sm h-4">{{ errorMessage }}</div>
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
    <canvas ref="freqRef" style="width: 500px; height: 300px"></canvas>
  </div>
</template>
