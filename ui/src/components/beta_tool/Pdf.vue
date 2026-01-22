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
const pdfChartRef = ref(null);
let pdfChart = null;

const fittedPdfMin = computed(() =>
  beta.domain.map((x, i) => ({
    x: x,
    y: beta.fittedPdfMin[i],
  }))
);

const fittedPdfMax = computed(() =>
  beta.domain.map((x, i) => ({
    x: x,
    y: beta.fittedPdfMax[i],
  }))
);

const predictedPdf = computed(() =>
  beta.domain.map((x, i) => ({
    x: x,
    y: beta.predictedPdf[i],
  }))
);

const samplingPdf = computed(() =>
  beta.samplingPdf && beta.samplingPdf.length > 0
    ? beta.domain.map((x, i) => ({
        x: x,
        y: beta.samplingPdf[i],
      }))
    : []
);

let testModePdf = null;
if (beta.testMode) {
  testModePdf = computed(() =>
    beta.domain.map((x, i) => ({
      x: x,
      y: beta.testModePdf[i],
    }))
  );
}

const createChart = () => {
  pdfChart = new Chart(pdfChartRef.value, {
    type: 'scatter',
    data: {
      datasets: [
        {
          type: 'line',
          label: t('beta.pdf.pdf-min'),
          data: fittedPdfMin.value,
          borderColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--pdf-min-color').trim(),
          backgroundColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--pdf-min-color').trim(),
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        {
          type: 'line',
          label: t('beta.pdf.pdf-max'),
          data: fittedPdfMax.value,
          borderColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--pdf-max-color').trim(),
          backgroundColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--pdf-max-color').trim(),
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        {
          type: 'line',
          label: t('beta.pdf.predicted-pdf'),
          data: predictedPdf.value,
          borderColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--pdf-predicted-color').trim(),
          backgroundColor: getComputedStyle(document.documentElement)
          .getPropertyValue('--pdf-predicted-color').trim(),
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        ...(samplingPdf.value && samplingPdf.value.length > 0
          ? [
              {
                type: 'line',
                label: t('beta.pdf.sampling-pdf'),
                data: samplingPdf.value,
                borderColor: getComputedStyle(document.documentElement)
                  .getPropertyValue('--pdf-sampling-color').trim(),
                backgroundColor: getComputedStyle(document.documentElement)
                  .getPropertyValue('--pdf-sampling-color').trim(),
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
                label: t('beta.pdf.test-mode-pdf'),
                data: testModePdf.value,
                borderColor: getComputedStyle(document.documentElement)
                .getPropertyValue('--pdf-testmode-color').trim(),
                backgroundColor: getComputedStyle(document.documentElement)
                .getPropertyValue('--pdf-testmode-color').trim(),
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
          text: t('beta.pdf.chart-title'),
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
          min: 0,
          max: 1,
          ticks: {
            stepSize: 0.1,
          },
          grid: {
            color: getComputedStyle(document.documentElement)
                  .getPropertyValue('--grid-color').trim(),
          },
        },
        y: {
          title: {
            display: true,
            text: '\u03C1(x)',
          },
          ticks: {
            stepSize: 0.1,
          },
          grid: {
            color: getComputedStyle(document.documentElement)
                  .getPropertyValue('--grid-color').trim(),
          },
        },
      },
    },
  });
}

onMounted(() => {
  if (pdfChart === null) {
    createChart();
  }
});

onUnmounted(() => {
  if (pdfChart) {
    pdfChart.destroy();
  }
})

watch(
  () => [
    fittedPdfMin,
    fittedPdfMax,
    predictedPdf,
    samplingPdf,
    testModePdf,
  ],
  () => {
    if (pdfChart) {
      pdfChart.data.datasets[0].data = fittedPdfMin.value;
      pdfChart.data.datasets[1].data = fittedPdfMax.value;
      pdfChart.data.datasets[2].data = predictedPdf.value;
      if (samplingPdf.value && samplingPdf.value.length > 0) {
        pdfChart.data.datasets[3].data = samplingPdf.value;
      }
      if (beta.testMode) {
        pdfChart.data.datasets[4].data = testModePdf.value;
      }
      pdfChart.update();
    }
  },
  { deep: true }
);

watch(
  () => [theme.currentTheme, language.currentLanguage],
  ([newTheme, newLanguage], [oldTheme, oldLanguage]) => {
    if (pdfChart) {
      if (newTheme !== oldTheme) {
        pdfChart.destroy();
        createChart();
      }
      if (newLanguage !== oldLanguage) {
        pdfChart.options.plugins.title.text = t('beta.pdf.chart-title');
        pdfChart.data.datasets[0].label = t('beta.pdf.pdf-min');
        pdfChart.data.datasets[1].label = t('beta.pdf.pdf-max');
        pdfChart.data.datasets[2].label = t('beta.pdf.predicted-pdf');
        if (beta.samplingPdf && beta.samplingPdf.length > 0) {
          pdfChart.data.datasets[3].label = t('beta.pdf.sampling-pdf');
        }
        if (beta.testMode) {
          pdfChart.data.datasets[4].label = t('beta.pdf.test-mode-pdf');
        }
        pdfChart.update();
      }
    }
  }
);

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <canvas ref="pdfChartRef" style="width: 500px; height: 500px;"></canvas>
  </div>
</template>
