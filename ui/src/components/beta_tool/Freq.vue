<script setup>
  import { ref, onMounted, onUnmounted, computed, watch } from "vue";
  import { betaStore, settingsStore, themeStore, languageStore } from "../../store/index.js";
  import { Chart, registerables } from "chart.js";
  import {ArrowPathIcon} from "@heroicons/vue/24/outline/index.js";
  import ApiService from "../../services/api.js";
  import { useI18n } from "vue-i18n";

  const { t } = useI18n();
  const language = languageStore();
  const theme = themeStore();
  const settings = settingsStore();
  const beta = betaStore();

  Chart.register(...registerables);
  const freqRef = ref(null);
  let freqChart = null;
  const newBinsNumber = ref(null);

  const updateFreq = async () => {
    const binsNumber = parseInt(newBinsNumber.value, 10);
    if (isNaN(binsNumber) || binsNumber < 1 || binsNumber > 50) {
      beta.errorMessage = "Please enter a valid number of bins (1-50)";
      return;
    }
    
    const savedBinsNumber = beta.binsNumber;
    beta.errorMessage = "";
    beta.inputDisabled = true;
    beta.binsNumber = binsNumber;
    
    const api = new ApiService(settings.backendUrl, settings.connectTimeout);
    
    try {
      const result = await api.getHistogram(
        beta.distribution,
        beta.scaledData,
        binsNumber,
        beta.paramsMin,
        beta.paramsMax,
        beta.predictedParams
      );
      
      api.close();
      
      if (!result.success) {
        beta.errorMessage = 'Backend error: ' + result.message;
        beta.inputDisabled = false;
        beta.binsNumber = savedBinsNumber;
        return;
      }
      
      beta.errorMessage = "";
      beta.inputDisabled = false;
      beta.binEdges = result.bin_edges;
      beta.observedFreq = result.observed_freq;
      beta.expectedFreqMin = result.expected_freq_min;
      beta.expectedFreqMax = result.expected_freq_max;
      beta.expectedFreqPred = result.expected_freq_pred;
      beta.binsNumber = result.bin_edges.length - 1;
      
      // Chi2 results from histogram
      if (result.chi2_min) beta.chi2Min = result.chi2_min;
      if (result.chi2_max) beta.chi2Max = result.chi2_max;
      if (result.chi2_pred) beta.chi2Pred = result.chi2_pred;
      
    } catch (error) {
      api.close();
      beta.errorMessage = error.message;
      beta.inputDisabled = false;
      beta.binsNumber = savedBinsNumber;
    }
  };

  const binsLabels = computed(() => 
    beta.binEdges.map(bin => bin.toFixed(2).toString())
  );

  const createHistogram = () => {
    freqChart = new Chart(freqRef.value, {
      type: "bar",
      data: {
        labels: binsLabels.value,
        datasets: [
          {
            label: t('beta.freq.chart-y-label'),
            data: beta.observedFreq,
            backgroundColor: getComputedStyle(document.documentElement)
                            .getPropertyValue('--bar-color').trim(),
            borderColor: getComputedStyle(document.documentElement)
                        .getPropertyValue('--bar-border-color').trim(),
            borderWidth: 1,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            display: false,
          },
          title: {
            display: true,
            text: t('beta.freq.chart-title'),
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
              text: t('beta.freq.chart-x-label'),
            },
            grid: {
              color: getComputedStyle(document.documentElement)
                    .getPropertyValue('--grid-color').trim(),
            },
            offset: true,
            barPercentage: 1.0,
            categoryPercentage: 1.0,
          },
          y: {
            title: {
              display: true,
              text: t('beta.freq.chart-y-label'),
            },
            grid: {
              color: getComputedStyle(document.documentElement)
                    .getPropertyValue('--grid-color').trim(),
            },
            beginAtZero: true,
          },
        },
      },
    });
  };

  onMounted(() => {
    newBinsNumber.value = beta.binEdges.length - 1;
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
      newBinsNumber.value = beta.binEdges.length - 1;
      if (freqChart) {
        freqChart.data.labels = binsLabels.value;
        freqChart.data.datasets[0].data = beta.observedFreq;
        freqChart.update();
      }
    },
    { deep: true }
  );

watch(
  () => [theme.currentTheme, language.currentLanguage],
  ([newTheme, newLanguage], [oldTheme, oldLanguage]) => {
    if (freqChart) {
      if (newTheme !== oldTheme) {
        freqChart.destroy();
        createHistogram();
      }
      if (newLanguage !== oldLanguage) {
        freqChart.options.plugins.title.text = t('beta.freq.chart-title');
        freqChart.options.scales.x.title.text = t('beta.freq.chart-x-label');
        freqChart.options.scales.y.title.text = t('beta.freq.chart-y-label');
        freqChart.data.datasets[0].label = t('beta.freq.chart-y-label');
        freqChart.update();
      }
    }
  }
);

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <div class="flex items-center space-x-4">
      <label for="binsNumber" class="label-text">{{ t('beta.freq.bins-number') }}</label>
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
    <div class="chart-container">
      <canvas ref="freqRef"></canvas>
    </div>
  </div>
</template>
