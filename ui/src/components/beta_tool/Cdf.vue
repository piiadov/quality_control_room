<script setup>

import { ref, computed } from "vue";
import { useBetaStore } from "../../store";
import { useChart, getCssVar, mapToXY, lineDataset, scatterDataset, getDefaultChartOptions } from "../../composables";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const beta = useBetaStore();

const cdfChartRef = ref(null);

// Computed data mappings
const cdfMin = computed(() => mapToXY(beta.scaledData, beta.cdfMin));
const cdfMax = computed(() => mapToXY(beta.scaledData, beta.cdfMax));
const fittedCdfMin = computed(() => mapToXY(beta.domain, beta.fittedCdfMin));
const fittedCdfMax = computed(() => mapToXY(beta.domain, beta.fittedCdfMax));
const predictedCdf = computed(() => mapToXY(beta.domain, beta.predictedCdf));
const samplingCdf = computed(() => 
  beta.samplingCdf?.length > 0 ? mapToXY(beta.domain, beta.samplingCdf) : []
);
const testModeCdf = computed(() => 
  beta.testMode ? mapToXY(beta.domain, beta.testModeCdf) : []
);

const createChart = () => ({
  type: 'scatter',
  data: {
    datasets: [
      scatterDataset(t('beta.cdf.est-cdf-min'), cdfMin.value, getCssVar('--est-cdf-min-color')),
      scatterDataset(t('beta.cdf.est-cdf-max'), cdfMax.value, getCssVar('--est-cdf-max-color')),
      lineDataset(t('beta.cdf.cdf-min'), fittedCdfMin.value, getCssVar('--cdf-min-color')),
      lineDataset(t('beta.cdf.cdf-max'), fittedCdfMax.value, getCssVar('--cdf-max-color')),
      lineDataset(t('beta.cdf.predicted-cdf'), predictedCdf.value, getCssVar('--cdf-predicted-color')),
      ...(samplingCdf.value.length > 0
        ? [lineDataset(t('beta.cdf.sampling-cdf'), samplingCdf.value, getCssVar('--cdf-sampling-color'))]
        : []),
      ...(beta.testMode
        ? [lineDataset(t('beta.cdf.test-mode-cdf'), testModeCdf.value, getCssVar('--cdf-testmode-color'))]
        : []),
    ],
  },
  options: getDefaultChartOptions(t('beta.cdf.chart-title'), 'x', '1 - P(ξ ≤ x)'),
});

const updateChartData = (chart) => {
  chart.data.datasets[0].data = cdfMin.value;
  chart.data.datasets[1].data = cdfMax.value;
  chart.data.datasets[2].data = fittedCdfMin.value;
  chart.data.datasets[3].data = fittedCdfMax.value;
  chart.data.datasets[4].data = predictedCdf.value;
  if (samplingCdf.value.length > 0 && chart.data.datasets[5]) {
    chart.data.datasets[5].data = samplingCdf.value;
  }
  if (beta.testMode && chart.data.datasets[6]) {
    chart.data.datasets[6].data = testModeCdf.value;
  }
  chart.update();
};

const updateChartLabels = (chart) => {
  chart.options.plugins.title.text = t('beta.cdf.chart-title');
  chart.data.datasets[0].label = t('beta.cdf.est-cdf-min');
  chart.data.datasets[1].label = t('beta.cdf.est-cdf-max');
  chart.data.datasets[2].label = t('beta.cdf.cdf-min');
  chart.data.datasets[3].label = t('beta.cdf.cdf-max');
  chart.data.datasets[4].label = t('beta.cdf.predicted-cdf');
  if (beta.samplingCdf?.length > 0 && chart.data.datasets[5]) {
    chart.data.datasets[5].label = t('beta.cdf.sampling-cdf');
  }
  if (beta.testMode && chart.data.datasets[6]) {
    chart.data.datasets[6].label = t('beta.cdf.test-mode-cdf');
  }
  chart.update();
};

// Use composable for chart lifecycle
useChart(cdfChartRef, createChart, {
  watchData: [cdfMin, cdfMax, fittedCdfMin, fittedCdfMax, predictedCdf, samplingCdf, testModeCdf],
  onDataChange: updateChartData,
  onLanguageChange: updateChartLabels,
});

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <canvas ref="cdfChartRef" style="width: 500px; height: 500px;"></canvas>
  </div>
</template>
