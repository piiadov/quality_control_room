<script setup>
import WebSocketService from '../../services/websocketService.js'
import { ref, onMounted } from 'vue';
import { useSettings, useSidebarStore, 
  useQualityProfileInput, useQualityProfileResults } from '../../store/index.js';
import { QuestionMarkCircleIcon } from "@heroicons/vue/24/outline/index.js";
import Results from './Results.vue';

const sidebarStore = useSidebarStore();
const settingsStore = useSettings();
const qualityProfileInputStore = useQualityProfileInput();
const qualityProfileResultsStore = useQualityProfileResults();

const isDisabled = ref(false);
const errorMessage = ref(null);

onMounted(() => {
  sidebarStore.activeTool = "QualityProfile";
});

function stringToNumberArray(str) {
  return str !== "" && str.match(/-?\d*\.?\d+/g) ? str.match(/-?\d*\.?\d+/g).map(Number) : [];
}

const submitData = () => {
  errorMessage.value = null;

  const volume = parseInt(qualityProfileInputStore.batchVolume.toString(), 10);
  if ((volume === NaN || volume < 1) && !qualityProfileInputStore.testMode) {
    errorMessage.value = 'Dscretization: Please enter valid positive integer number';
    qualityProfileResultsStore.showResults = false;
    return;
  } else {
    qualityProfileInputStore.batchVolume = volume;
  }

  const minValue = parseFloat(qualityProfileInputStore.minValue.toString());
  const maxValue = parseFloat(qualityProfileInputStore.maxValue.toString());
  if ((minValue === NaN || maxValue === NaN) && !qualityProfileInputStore.testMode) {
    errorMessage.value = 'Min or Max value: Please enter valid float number';
    qualityProfileResultsStore.showResults = false;
    return;
  } else {
    qualityProfileInputStore.minValue = minValue;
    qualityProfileInputStore.maxValue = maxValue;
  }

  if ((minValue >= maxValue) && !qualityProfileInputStore.testMode) {
    errorMessage.value = 'Min value must be less than max value';
    qualityProfileResultsStore.showResults = false;
    return;
  }

  const data = stringToNumberArray(qualityProfileInputStore.samplingData.toString());
  if (data.length === 0 && !qualityProfileInputStore.testMode) {
    errorMessage.value = 'Sampling data: Please enter valid float numbers';
    qualityProfileResultsStore.showResults = false;
    return;
  } else {
    qualityProfileInputStore.samplingData = data;
  }

  if (volume < data.length && !qualityProfileInputStore.testMode) {
    errorMessage.value = 'Batch volume or discretization factor must be greater than sampling size';
    qualityProfileResultsStore.showResults = false;
    return;
  }

  const areAllValuesInRange = data.every(value => value >= qualityProfileInputStore.minValue && value <= qualityProfileInputStore.maxValue);
  if (!areAllValuesInRange && !qualityProfileInputStore.testMode) {
    errorMessage.value = 'Sampling data: All values must be within the specified range';
    qualityProfileResultsStore.showResults = false;
    return;
  }

  isDisabled.value = true;
  const ws = new WebSocketService(settingsStore.backendUrl, settingsStore.connectTimeout);
  ws.connectAndSendData('calc', qualityProfileInputStore)
    .then(response => {
      if (response.data.error > 0) {
        errorMessage.value = 'Backend error: ' + response.data.info;
        isDisabled.value = false;
        qualityProfileResultsStore.showResults = false;
        return;
      } else {
        errorMessage.value = null;
        isDisabled.value = false;

        qualityProfileInputStore.batchVolume = response.data.population_size;
        qualityProfileInputStore.minValue = response.data.min_value;
        qualityProfileInputStore.maxValue = response.data.max_value;
        qualityProfileInputStore.samplingData = response.data.data;

        qualityProfileResultsStore.info = response.data.info;
        qualityProfileResultsStore.scaledData = response.data.scaled_data;
        qualityProfileResultsStore.cdfMin = response.data.cdf_min;
        qualityProfileResultsStore.cdfMax = response.data.cdf_max;
        qualityProfileResultsStore.q = response.data.q;
        qualityProfileResultsStore.fittedCdfMin = response.data.fitted_cdf_min;
        qualityProfileResultsStore.fittedCdfMax = response.data.fitted_cdf_max;
        qualityProfileResultsStore.fittedPdfMin = response.data.fitted_pdf_min;
        qualityProfileResultsStore.fittedPdfMax = response.data.fitted_pdf_max;
        qualityProfileResultsStore.betaParamsMin = response.data.beta_params_min;
        qualityProfileResultsStore.betaParamsMax = response.data.beta_params_max;
        qualityProfileResultsStore.predictedBetaParams = response.data.predicted_beta_params;
        qualityProfileResultsStore.predictedCdf = response.data.predicted_cdf;
        qualityProfileResultsStore.predictedPdf = response.data.predicted_pdf;
        qualityProfileResultsStore.testModeBetaParams = response.data.test_mode_beta_params;
        qualityProfileResultsStore.testModeCdf = response.data.test_mode_cdf;
        qualityProfileResultsStore.testModePdf = response.data.test_mode_pdf;
        qualityProfileResultsStore.showResults = true;
      }
    })
    .catch(error => {
      errorMessage.value = error.message;
      isDisabled.value = false;
      qualityProfileResultsStore.showResults = false;
    });
};
</script>

<template>
  <header class="text-left mb-2 text-3xl font-semibold text-text p-4">
    <h1>Quantitative Quality Profiler</h1>
  </header>

  <main class="flex flex-1 justify-start gap-4 p-4">
    <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">

      <!-- Message if test mode -->
      <div v-if="qualityProfileInputStore.testMode" class="h-2 info-message text-sm">
        Test mode
      </div>

      <!--Error message-->
      <div v-if="errorMessage" class="error-message text-sm h-4">{{ errorMessage }}</div>
      
      <!-- Discretization and Min/Max values -->
      <div class="flex space-x-4">
        <!-- Batch volume/discretization -->
        <div class="flex-1">
          <div class="flex items-center justify-between space-x-2">
        <label for="batch-volume" class="block text-lg text-text">Discretization</label>
        <router-link to="/about">
          <QuestionMarkCircleIcon class="h-5 w-5 muted-link" />
        </router-link>
          </div>
          <input
            v-model="qualityProfileInputStore.batchVolume"
            type="text"
            id="batch-volume"
            class="mt-2 w-full p-3"
            placeholder="Enter an integer"
          />
        </div>

        <!-- Min Value -->
        <div class="flex-1">
          <div class="flex items-center justify-between space-x-2">
        <label for="min-value" class="block text-lg text-text">Min Value</label>
        <router-link to="/about">
          <QuestionMarkCircleIcon class="h-5 w-5 muted-link" />
        </router-link>
          </div>
          <input
            v-model="qualityProfileInputStore.minValue"
            type="text"
            id="min-value"
            class="mt-2 w-full p-3"
            placeholder="Enter a value"
          />
        </div>

        <!-- Max Value -->
        <div class="flex-1">
          <div class="flex items-center justify-between space-x-2">
        <label for="max-value" class="block text-lg text-text">Max Value</label>
        <router-link to="/about">
          <QuestionMarkCircleIcon class="h-5 w-5 muted-link" />
        </router-link>
          </div>
          <input
            v-model="qualityProfileInputStore.maxValue"
            type="text"
            id="max-value"
            class="mt-2 w-full p-3"
            placeholder="Enter a value"
          />
        </div>
      </div>

      <div>
        <!-- Sampling data -->
        <div class="flex items-center justify-between space-x-2">

          <label for="sampling-data" class="block text-lg text-text">
            Sampling Data
            <span class="w-auto muted-link text-xs">
            load from file
          </span>

          </label>

          <router-link to="/about">
            <QuestionMarkCircleIcon class="h-5 w-5 muted-link" />
          </router-link>
        </div>
        <textarea
            v-model="qualityProfileInputStore.samplingData"
            id="sampling-data"
            rows="4"
            class="mt-2 w-full p-3 border border-border-color"
            placeholder="Enter numbers separated with new line, comma or space"
        ></textarea>
      </div>

      <!-- Submit Button -->
      <div>
        <div class="text-center">
          <button @click="submitData" class="primary-button" :disabled="isDisabled">Analyze</button>
        </div>
      </div>
    </div>
    <Results v-if="qualityProfileResultsStore.showResults"/>
  </main>
</template>

<style scoped>
</style>