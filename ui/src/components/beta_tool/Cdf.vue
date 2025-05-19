<script setup>

import { onMounted, onUnmounted, ref, computed, watch } from "vue";
import { betaStore, themeStore, languageStore } from "../../store/index.js";
import { Chart, registerables } from "chart.js";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const beta = betaStore();
const theme = themeStore();
const language = languageStore();

Chart.register(...registerables);
const cdfChartRef = ref(null);
let cdfChart = null;

const cdfMin = computed(() =>
  beta.scaledData.map((x, i) => ({
    x: x,
    y: beta.cdfMin[i],
  }))
);

const cdfMax = computed(() =>
  beta.scaledData.map((x, i) => ({
    x: x,
    y: beta.cdfMax[i],
  }))
);

const fittedCdfMin = computed(() =>
  beta.q.map((x, i) => ({
    x: x,
    y: beta.fittedCdfMin[i],
  }))
);

const fittedCdfMax = computed(() =>
  beta.q.map((x, i) => ({
    x: x,
    y: beta.fittedCdfMax[i],
  }))
);

const predictedCdf = computed(() =>
  beta.q.map((x, i) => ({
    x: x,
    y: beta.predictedCdf[i],
  }))
);

const samplingCdf = computed(() =>
  beta.samplingCdf && beta.samplingCdf.length > 0
    ? beta.q.map((x, i) => ({
        x: x,
        y: beta.samplingCdf[i],
      }))
    : []
);

let testModeCdf = null;
if (beta.testMode) {
  testModeCdf = computed(() =>
    beta.q.map((x, i) => ({
      x: x,
      y: beta.testModeCdf[i],
    }))
  );
}

const createChart = () => {
  cdfChart = new Chart(cdfChartRef.value, {
    type: 'scatter',
    data: {
      datasets: [
        {
          type: 'scatter',
          label: t('beta.cdf.est-cdf-min'),
          data: cdfMin.value,
          borderColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--est-cdf-min-color').trim(),
          pointRadius: 2,
        },
        {
          type: 'scatter',
          label: t('beta.cdf.est-cdf-max'),
          data: cdfMax.value,
          borderColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--est-cdf-max-color').trim(),
          pointRadius: 2,
        },
        {
          type: 'line',
          label: t('beta.cdf.cdf-min'),
          data: fittedCdfMin.value,
          borderColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--cdf-min-color').trim(),
          backgroundColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--cdf-min-color').trim(),
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        {
          type: 'line',
          label: t('beta.cdf.cdf-max'),
          data: fittedCdfMax.value,
          borderColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--cdf-max-color').trim(),
          backgroundColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--cdf-max-color').trim(),
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        {
          type: 'line',
          label: t('beta.cdf.predicted-cdf'),
          data: predictedCdf.value,
          borderColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--cdf-predicted-color').trim(),
          backgroundColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--cdf-predicted-color').trim(),
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        ...(samplingCdf.value && samplingCdf.value.length > 0
          ? [
              {
                type: 'line',
                label: t('beta.cdf.sampling-cdf'),
                data: samplingCdf.value,
                borderColor: getComputedStyle(document.documentElement)
                  .getPropertyValue('--cdf-sampling-color').trim(),
                backgroundColor: getComputedStyle(document.documentElement)
                  .getPropertyValue('--cdf-sampling-color').trim(),
                borderWidth: 2,
                fill: false,
                pointRadius: 0,
              },
            ]
          : []
        ),
        ...(beta.testMode
          ? [
              {
                type: 'line',
                label: t('beta.cdf.test-mode-cdf'),
                data: testModeCdf.value,
                borderColor: getComputedStyle(document.documentElement)
                .getPropertyValue('--cdf-testmode-color').trim(),
                backgroundColor: getComputedStyle(document.documentElement)
                .getPropertyValue('--cdf-testmode-color').trim(),
                borderWidth: 2,
                fill: false,
                pointRadius: 0,
              },
            ]
          : []
        ),
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: {
          position: 'bottom',
          labels: {
            font: {
              size: 12,
            },
          },
        },
        title: {
          display: true,
          text: t('beta.cdf.chart-title'),
          font: {
            size: 14,
          },
        },
      },
      scales: {
        x: {
          type: 'linear',
          position: 'bottom',
          title: {
            display: true,
            text: 'x',
          },
          grid: {
            color: getComputedStyle(document.documentElement)
                  .getPropertyValue('--grid-color').trim(),
          },
          min: 0,
          max: 1,
          ticks: {
            stepSize: 0.1,
          },
        },
        y: {
          title: {
            display: true,
            text: '1 - P(ξ ≤ x)',
          },
          grid: {
            color: getComputedStyle(document.documentElement)
                  .getPropertyValue('--grid-color').trim(),
          },
          min: 0,
          max: 1,
          ticks: {
            stepSize: 0.1,
          },
        },
      },
    },
  });
}

onMounted(() => {
  if (cdfChart === null) {
    createChart();
  }
});

onUnmounted(() => {
  if (cdfChart) {
    cdfChart.destroy();
  }
})

watch(
  () => [
    cdfMin,
    cdfMax,
    fittedCdfMin,
    fittedCdfMax,
    predictedCdf,
    samplingCdf,
    testModeCdf,
  ],
  () => {
    if (cdfChart) {
      cdfChart.data.datasets[0].data = cdfMin.value;
      cdfChart.data.datasets[1].data = cdfMax.value;
      cdfChart.data.datasets[2].data = fittedCdfMin.value;
      cdfChart.data.datasets[3].data = fittedCdfMax.value;
      cdfChart.data.datasets[4].data = predictedCdf.value;
      if (samplingCdf.value && samplingCdf.value.length > 0) {
        cdfChart.data.datasets[5].data = samplingCdf.value;
      }
      if (beta.testMode) {
        cdfChart.data.datasets[6].data = testModeCdf.value;
      }
      cdfChart.update();
    }
  },
  { deep: true }
);

watch(
  () => [theme.currentTheme, language.currentLanguage],
  ([newTheme, newLanguage], [oldTheme, oldLanguage]) => {
    if (cdfChart) {
      if (newTheme !== oldTheme) {
        cdfChart.destroy();
        createChart();
      }
      if (newLanguage !== oldLanguage) {
        cdfChart.options.plugins.title.text = t('beta.cdf.chart-title');
        cdfChart.data.datasets[0].label = t('beta.cdf.est-cdf-min');
        cdfChart.data.datasets[1].label = t('beta.cdf.est-cdf-max');
        cdfChart.data.datasets[2].label = t('beta.cdf.cdf-min');
        cdfChart.data.datasets[3].label = t('beta.cdf.cdf-max');
        cdfChart.data.datasets[4].label = t('beta.cdf.predicted-cdf');
        if (beta.samplingCdf && beta.samplingCdf.length > 0) {
          cdfChart.data.datasets[5].label = t('beta.cdf.sampling-pdf');
        }
        if (beta.testMode) {
          cdfChart.data.datasets[6].label = t('beta.cdf.test-mode-cdf');
        }
        cdfChart.update();
      }
    }
  }
);

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <canvas ref="cdfChartRef" style="width: 500px; height: 500px;"></canvas>
  </div>
</template>
