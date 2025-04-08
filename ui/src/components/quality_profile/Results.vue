<script setup>

import { onMounted, onUnmounted } from 'vue';
import {useSidebarStore, useQualityProfileResults} from "../../store/index.js";

const sidebarStore = useSidebarStore();
const qualityProfileResults = useQualityProfileResults();

onMounted(() => {
  sidebarStore.sidebarResults = true;
});

onUnmounted(() => {
  sidebarStore.sidebarResults = false;
})

// const chartsData = ref(null);
//
// const chartOptions = {
//   chart: {
//     height: 350,
//     type: 'line',
//     zoom: {
//       enabled: false,
//     },
//   },
//   xaxis: {
//     categories: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct'],
//   },
// };

// const analyzeData = () => {
//   if (!inputNumber.value) {
//     alert('Please enter a valid number');
//     return;
//   }
//
//   chartsData.value = {
//     chart1: [
//       {
//         name: 'Dataset 1',
//         data: [34, 44, 54, 21, 12, 43, 25, 16, 50, 20],
//       },
//     ],
//     chart2: [
//       {
//         name: 'Dataset 2',
//         data: [20, 50, 70, 40, 30, 60, 80, 90, 60, 20],
//       },
//     ],
//   };
// };

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-6">


    <p>{{ qualityProfileResults.q}}</p>


  </div>


  <!-- Chart Section (hidden until analyzed) -->
  <!--    <div v-if="chartsData" class="mt-12">-->
  <!--      <div class="grid grid-cols-1 md:grid-cols-2 gap-8">-->
  <!--        <div class="bg-background-secondary p-4 rounded-lg shadow-md">-->
  <!--          <apexchart type="line" :options="chartOptions" :series="chartsData.chart1" />-->
  <!--        </div>-->
  <!--        <div class="bg-background-secondary p-4 rounded-lg shadow-md">-->
  <!--          <apexchart type="bar" :options="chartOptions" :series="chartsData.chart2" />-->
  <!--        </div>-->
  <!--      </div>-->
  <!--    </div>-->

  <!--  <div>-->
  <!--    <h1 class="text-2xl font-bold">Quantitative Quality Profile</h1>-->
  <!--    <p>Evaluate quantitative quality metrics for your samples.</p>-->
  <!--  </div>-->

  <!--  <div>-->
  <!--    <label for="numbers" class="block text-gray-700 font-bold mb-2">Enter Numbers:</label>-->
  <!--    <textarea-->
  <!--        id="numbers"-->
  <!--        class="w-full p-2 border border-gray-300 rounded focus:outline-none focus:ring focus:border-blue-500"-->
  <!--        rows="5"-->
  <!--        v-model="numbersInput"-->
  <!--        placeholder="Enter float values separated by spaces, commas, or new lines"-->
  <!--    ></textarea>-->
  <!--    <div v-if="errorBatchVolume" class="text-red-500 text-sm mt-2">{{ errorBatchVolume }}</div>-->
  <!--    <button-->
  <!--        class="mt-4 px-4 py-2 bg-teal-500 text-white rounded hover:bg-teal-600"-->
  <!--        @click="processNumbers"-->
  <!--    >-->
  <!--      Submit-->
  <!--    </button>-->
  <!--    <div v-if="processedNumbers.length" class="mt-4">-->
  <!--      <h3 class="text-gray-700 font-bold mb-2">Processed Numbers:</h3>-->
  <!--      <p class="text-gray-600">{{ processedNumbers }}</p>-->
  <!--    </div>-->
  <!--  </div>-->

</template>

<style scoped>

</style>
