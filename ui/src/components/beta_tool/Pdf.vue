<script setup>

import { onMounted, onUnmounted, ref, computed, watch } from 'vue';
import { betaStore } from "../../store/index.js";
import { Chart, registerables } from 'chart.js';

const beta = betaStore();

Chart.register(...registerables);
const pdfChartRef = ref(null);
let pdfChart = null;

const fittedPdfMin = computed(() =>
  beta.q.map((x, i) => ({
    x: x,
    y: beta.fittedPdfMin[i],
  }))
);

const fittedPdfMax = computed(() =>
  beta.q.map((x, i) => ({
    x: x,
    y: beta.fittedPdfMax[i],
  }))
);

const predictedPdf = computed(() =>
  beta.q.map((x, i) => ({
    x: x,
    y: beta.predictedPdf[i],
  }))
);

let testModePdf = null;
if (beta.testMode) {
  testModePdf = computed(() =>
    beta.q.map((x, i) => ({
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
          label: 'PDF Min',
          data: fittedPdfMin.value,
          borderColor: '#8B0000',
          backgroundColor: '#8B0000',
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        {
          type: 'line',
          label: 'PDF Max',
          data: fittedPdfMax.value,
          borderColor: '#8B0000',
          backgroundColor: '#8B0000',
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        {
          type: 'line',
          label: 'Predicted PDF',
          data: predictedPdf.value,
          borderColor: '#00FF00',
          backgroundColor: '#00FF00',
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        ...(beta.testMode
          ? [
              {
                type: 'line',
                label: 'True PDF (test mode)',
                data: testModePdf.value,
                borderColor: '#1E90FF',
                backgroundColor: '#1E90FF',
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
          text: 'PDF',
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
            color: '#5c5c5c',
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
            color: '#5c5c5c',
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
    testModePdf,
  ],
  () => {
    if (pdfChart) {
      pdfChart.data.datasets[0].data = fittedPdfMin.value;
      pdfChart.data.datasets[1].data = fittedPdfMax.value;
      pdfChart.data.datasets[2].data = predictedPdf.value;
      if (beta.testMode) {
        pdfChart.data.datasets[3].data = testModePdf.value;
      }
      pdfChart.update();
    }
  },
  { deep: true }
);

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <canvas ref="pdfChartRef" style="width: 500px; height: 500px;"></canvas>
  </div>
</template>
