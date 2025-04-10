<script setup>

import { onMounted, onUnmounted, ref, computed, watch } from 'vue';
import { sidebarStore, betaInputStore, betaResultsStore }
        from "../../store/index.js";
import { Chart, registerables } from 'chart.js';

const sidebar = sidebarStore();
const betaResults = betaResultsStore();
const betaInputs = betaInputStore();

Chart.register(...registerables);
const pdfChartRef = ref(null);
let pdfChart = null;

const fittedPdfMin = computed(() =>
  betaResults.q.map((x, i) => ({
    x: x,
    y: betaResults.fittedPdfMin[i],
  }))
);

const fittedPdfMax = computed(() =>
  betaResults.q.map((x, i) => ({
    x: x,
    y: betaResults.fittedPdfMax[i],
  }))
);

const predictedPdf = computed(() =>
  betaResults.q.map((x, i) => ({
    x: x,
    y: betaResults.predictedPdf[i],
  }))
);

let testModePdf = null;
if (betaInputs.testMode) {
  testModePdf = computed(() =>
    betaResults.q.map((x, i) => ({
      x: x,
      y: betaResults.testModePdf[i],
    }))
  );
}

onMounted(() => {
  sidebar.sidebarResults = true;

  if (pdfChartRef.value) {
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
          ...(betaInputs.testMode
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
              text: 'rho(x)',
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

});

watch(
  [fittedPdfMin, fittedPdfMax, predictedPdf, testModePdf],
  ([newFittedPdfMin, newFittedPdfMax, newPredictedPdf, newTestModePdf]) => {
    if (pdfChart) {
      pdfChart.data.datasets[0].data = newFittedPdfMin;
      pdfChart.data.datasets[1].data = newFittedPdfMax;
      pdfChart.data.datasets[2].data = newPredictedPdf;
      if (betaInputs.testMode) {
        pdfChart.data.datasets[3].data = newTestModePdf;
      }
      pdfChart.update();
    }
  }
);

onUnmounted(() => {
  sidebar.sidebarResults = false;
  if (pdfChart) {
    pdfChart.destroy();
  }
})

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <canvas ref="pdfChartRef" style="width: 500px; height: 500px;"></canvas>
  </div>
</template>
