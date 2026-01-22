<script setup>
  import { ref, computed, onMounted, watch } from "vue";
  import { useBetaStore, useSettingsStore } from "../../store";
  import { useChart, getCssVar, getHistogramOptions } from "../../composables";
  import { validateBinsNumber } from "../../utils/validation";
  import { ArrowPathIcon } from "@heroicons/vue/24/outline/index.js";
  import ApiService from "../../services/api.js";
  import { useI18n } from "vue-i18n";

  const { t } = useI18n();
  const settings = useSettingsStore();
  const beta = useBetaStore();

  const freqRef = ref(null);
  const newBinsNumber = ref(null);

  const updateFreq = async () => {
    const result = validateBinsNumber(newBinsNumber.value);
    if (!result.valid) {
      beta.errorMessage = result.error;
      return;
    }
    
    const savedBinsNumber = beta.binsNumber;
    beta.errorMessage = "";
    beta.inputDisabled = true;
    beta.binsNumber = result.value;
    
    const api = new ApiService(settings.backendUrl, settings.connectTimeout);
    
    try {
      const histResult = await api.getHistogram(
        beta.distribution,
        beta.scaledData,
        result.value,
        beta.paramsMin,
        beta.paramsMax,
        beta.predictedParams
      );
      
      api.close();
      
      if (!histResult.success) {
        beta.errorMessage = 'Backend error: ' + histResult.message;
        beta.inputDisabled = false;
        beta.binsNumber = savedBinsNumber;
        return;
      }
      
      // Use store action to update histogram data
      beta.updateHistogram(histResult);
      
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

  const createChart = () => ({
    type: "bar",
    data: {
      labels: binsLabels.value,
      datasets: [
        {
          label: t('beta.freq.chart-y-label'),
          data: beta.observedFreq,
          backgroundColor: getCssVar('--bar-color'),
          borderColor: getCssVar('--bar-border-color'),
          borderWidth: 1,
        },
      ],
    },
    options: getHistogramOptions(
      t('beta.freq.chart-title'),
      t('beta.freq.chart-x-label'),
      t('beta.freq.chart-y-label')
    ),
  });

  const updateChartData = (chart) => {
    chart.data.labels = binsLabels.value;
    chart.data.datasets[0].data = beta.observedFreq;
    chart.update();
  };

  const updateChartLabels = (chart) => {
    chart.options.plugins.title.text = t('beta.freq.chart-title');
    chart.options.scales.x.title.text = t('beta.freq.chart-x-label');
    chart.options.scales.y.title.text = t('beta.freq.chart-y-label');
    chart.data.datasets[0].label = t('beta.freq.chart-y-label');
    chart.update();
  };

  // Initialize binsNumber on mount
  onMounted(() => {
    newBinsNumber.value = beta.binEdges.length - 1;
  });

  // Update input when bins change externally
  watch(() => beta.binEdges, () => {
    newBinsNumber.value = beta.binEdges.length - 1;
  }, { deep: true });

  // Use composable for chart lifecycle
  useChart(freqRef, createChart, {
    watchData: [binsLabels, () => beta.observedFreq],
    onDataChange: updateChartData,
    onLanguageChange: updateChartLabels,
  });

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
