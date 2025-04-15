<script setup>

import { onMounted, onUnmounted, ref, computed, watch } from 'vue';
import { betaStore } from "../../store/index.js";
import { Chart, registerables } from 'chart.js';

const beta = betaStore();

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
          label: 'Estimated CDF Min',
          data: cdfMin.value,
          borderColor: '#2E8B57',
          pointRadius: 2,
        },
        {
          type: 'scatter',
          label: 'Estimated CDF Max',
          data: cdfMax.value,
          borderColor: '#2E8B57',
          pointRadius: 2,
        },
        {
          type: 'line',
          label: 'CDF Min',
          data: fittedCdfMin.value,
          borderColor: '#8B0000',
          backgroundColor: '#8B0000',
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        {
          type: 'line',
          label: 'CDF Max',
          data: fittedCdfMax.value,
          borderColor: '#8B0000',
          backgroundColor: '#8B0000',
          borderWidth: 2,
          fill: false,
          pointRadius: 0,
        },
        {
          type: 'line',
          label: 'Predicted CDF',
          data: predictedCdf.value,
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
                label: 'True CDF (test mode)',
                data: testModeCdf.value,
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
            // filter: (legendItem, _) => {
            //   return legendItem.datasetIndex !== 1 
            //       && legendItem.datasetIndex !== 3;
            // },
          },
        },
        title: {
          display: true,
          text: 'Complementary CDF',
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
            text: '1 - P(ξ ≤ x)',
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
    testModeCdf,
  ],
  () => {
    if (cdfChart) {
      cdfChart.data.datasets[0].data = cdfMin.value;
      cdfChart.data.datasets[1].data = cdfMax.value;
      cdfChart.data.datasets[2].data = fittedCdfMin.value;
      cdfChart.data.datasets[3].data = fittedCdfMax.value;
      cdfChart.data.datasets[4].data = predictedCdf.value;
      if (beta.testMode) {
        cdfChart.data.datasets[5].data = testModeCdf.value;
      }
      cdfChart.update();
    }
  },
  { deep: true }
);

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <canvas ref="cdfChartRef" style="width: 500px; height: 500px;"></canvas>
  </div>
</template>
