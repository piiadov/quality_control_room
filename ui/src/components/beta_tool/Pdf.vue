<script setup>

import { ref, computed } from "vue";
import { useBetaStore } from "../../store";
import { useChart, getCssVar, mapToXY, lineDataset, getDefaultChartOptions } from "../../composables";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const beta = useBetaStore();

const pdfChartRef = ref(null);

// Computed data mappings
const fittedPdfMin = computed(() => mapToXY(beta.domain, beta.fittedPdfMin));
const fittedPdfMax = computed(() => mapToXY(beta.domain, beta.fittedPdfMax));
const predictedPdf = computed(() => mapToXY(beta.domain, beta.predictedPdf));
const samplingPdf = computed(() => 
  beta.samplingPdf?.length > 0 ? mapToXY(beta.domain, beta.samplingPdf) : []
);
const testModePdf = computed(() => 
  beta.testMode ? mapToXY(beta.domain, beta.testModePdf) : []
);

const createChart = () => ({
  type: 'scatter',
  data: {
    datasets: [
      lineDataset(t('beta.pdf.pdf-min'), fittedPdfMin.value, getCssVar('--pdf-min-color')),
      lineDataset(t('beta.pdf.pdf-max'), fittedPdfMax.value, getCssVar('--pdf-max-color')),
      lineDataset(t('beta.pdf.predicted-pdf'), predictedPdf.value, getCssVar('--pdf-predicted-color')),
      ...(samplingPdf.value.length > 0
        ? [lineDataset(t('beta.pdf.sampling-pdf'), samplingPdf.value, getCssVar('--pdf-sampling-color'))]
        : []),
      ...(beta.testMode
        ? [lineDataset(t('beta.pdf.test-mode-pdf'), testModePdf.value, getCssVar('--pdf-testmode-color'))]
        : []),
    ],
  },
  options: {
    ...getDefaultChartOptions(t('beta.pdf.chart-title'), 'x', '\u03C1(x)'),
    scales: {
      ...getDefaultChartOptions(t('beta.pdf.chart-title'), 'x', '\u03C1(x)').scales,
      y: {
        title: { display: true, text: '\u03C1(x)' },
        ticks: { stepSize: 0.1 },
        grid: { color: getCssVar('--grid-color') },
      },
    },
  },
});

const updateChartData = (chart) => {
  chart.data.datasets[0].data = fittedPdfMin.value;
  chart.data.datasets[1].data = fittedPdfMax.value;
  chart.data.datasets[2].data = predictedPdf.value;
  if (samplingPdf.value.length > 0 && chart.data.datasets[3]) {
    chart.data.datasets[3].data = samplingPdf.value;
  }
  if (beta.testMode && chart.data.datasets[4]) {
    chart.data.datasets[4].data = testModePdf.value;
  }
  chart.update();
};

const updateChartLabels = (chart) => {
  chart.options.plugins.title.text = t('beta.pdf.chart-title');
  chart.data.datasets[0].label = t('beta.pdf.pdf-min');
  chart.data.datasets[1].label = t('beta.pdf.pdf-max');
  chart.data.datasets[2].label = t('beta.pdf.predicted-pdf');
  if (beta.samplingPdf?.length > 0 && chart.data.datasets[3]) {
    chart.data.datasets[3].label = t('beta.pdf.sampling-pdf');
  }
  if (beta.testMode && chart.data.datasets[4]) {
    chart.data.datasets[4].label = t('beta.pdf.test-mode-pdf');
  }
  chart.update();
};

// Use composable for chart lifecycle
useChart(pdfChartRef, createChart, {
  watchData: [fittedPdfMin, fittedPdfMax, predictedPdf, samplingPdf, testModePdf],
  onDataChange: updateChartData,
  onLanguageChange: updateChartLabels,
});

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <canvas ref="pdfChartRef" style="width: 500px; height: 500px;"></canvas>
  </div>
</template>
