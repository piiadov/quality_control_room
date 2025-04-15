<script setup>
import WebSocketService from '../../services/websocketService.js'
import { ref, computed } from 'vue';
import { settingsStore, betaStore, sidebarStore} from '../../store/index.js';
import { QuestionMarkCircleIcon } from "@heroicons/vue/24/outline/index.js";

const settings = settingsStore();
const beta = betaStore();
const sidebar = sidebarStore();

const errorMessage = ref(null);

function stringToNumberArray(str) {
  return str !== "" && str.match(/-?\d*\.?\d+/g) ? str.match(/-?\d*\.?\d+/g).map(Number) : [];
}

const batchVolumeInput = computed(() =>
  !isNaN(beta.batchVolume) ? beta.batchVolume : ''
);

const minValueInput = computed(() =>
  !isNaN(beta.minValue) ? beta.minValue : ''
);

const maxValueInput = computed(() =>
  !isNaN(beta.maxValue) ? beta.maxValue : ''
);

const submitData = () => {
  errorMessage.value = null;

  const volume = parseInt(beta.batchVolume.toString(), 10);
  if ((isNaN(volume) || volume < 1) && !beta.testMode) {
    errorMessage.value = 'Dscretization: Please enter valid positive integer number';
    beta.showResults = false;
    return;
  } else {
    beta.batchVolume = volume;
  }

  const minValue = parseFloat(beta.minValue.toString());
  const maxValue = parseFloat(beta.maxValue.toString());
  if ((isNaN(minValue) || isNaN(maxValue)) && !beta.testMode) {
    errorMessage.value = 'Min or Max value: Please enter valid float number';
    beta.showResults = false;
    return;
  } else {
    beta.minValue = minValue;
    beta.maxValue = maxValue;
  }

  if ((minValue >= maxValue) && !beta.testMode) {
    errorMessage.value = 'Min value must be less than max value';
    beta.showResults = false;
    return;
  }

  const data = stringToNumberArray(beta.samplingData.toString());
  if (data.length === 0 && !beta.testMode) {
    errorMessage.value = 'Sampling data: Please enter valid float numbers';
    beta.showResults = false;
    return;
  } else {
    beta.samplingData = data;
  }

  if (volume < data.length && !beta.testMode) {
    errorMessage.value = 'Batch volume or discretization factor must be greater than sampling size';
    beta.showResults = false;
    return;
  }

  const areAllValuesInRange = data.every(value => value >= beta.minValue && value <= beta.maxValue);
  if (!areAllValuesInRange && !beta.testMode) {
    errorMessage.value = 'Sampling data: All values must be within the specified range';
    beta.showResults = false;
    return;
  }

  beta.inputDisabled = true;
  const ws = new WebSocketService(settings.backendUrl, settings.connectTimeout);
  ws.connectAndSendData('calc', beta)
    .then(response => {
      if (response.data.error > 0) {
        errorMessage.value = 'Backend error: ' + response.data.info;
        beta.inputDisabled = false;
        beta.showResults = false;
      } else {
        errorMessage.value = null;
        beta.inputDisabled = false;

        beta.batchVolume = response.data.population_size;
        beta.minValue = response.data.min_value;
        beta.maxValue = response.data.max_value;
        beta.samplingData = response.data.data;

        beta.info = response.data.info;
        beta.scaledData = response.data.scaled_data;
        beta.cdfMin = response.data.cdf_min;
        beta.cdfMax = response.data.cdf_max;
        beta.q = response.data.q;
        beta.fittedCdfMin = response.data.fitted_cdf_min;
        beta.fittedCdfMax = response.data.fitted_cdf_max;
        beta.fittedPdfMin = response.data.fitted_pdf_min;
        beta.fittedPdfMax = response.data.fitted_pdf_max;
        beta.betaParamsMin = response.data.beta_params_min;
        beta.betaParamsMax = response.data.beta_params_max;
        beta.predictedBetaParams = response.data.predicted_beta_params;
        beta.predictedCdf = response.data.predicted_cdf;
        beta.predictedPdf = response.data.predicted_pdf;
        beta.testModeBetaParams = response.data.test_mode_beta_params;
        beta.testModeCdf = response.data.test_mode_cdf;
        beta.testModePdf = response.data.test_mode_pdf;
        beta.bins = response.data.bins;
        beta.freq = response.data.freq;
        beta.showResults = true;
        sidebar.sidebarResults = true;
      }
    })
    .catch(error => {
      errorMessage.value = error.message;
      beta.inputDisabled = false;
      beta.showResults = false;
    });
};

</script>

<template>
    <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
      <!-- Message if test mode -->
      <div v-if="beta.testMode" class="h-2 info-message text-sm">
          Test mode
      </div>

      <!--Error message-->
      <div v-if="errorMessage" class="error-message text-sm h-4">
        {{ errorMessage }}
      </div>

        <!-- Discretization and Min/Max values -->
      <div class="flex space-x-4">
        <!-- Batch volume/discretization -->
        <div class="flex-1">
          <div class="flex items-center justify-between space-x-2">
            <label for="batch-volume" class="label-text">Discretization</label>
            <router-link to="/about">
              <QuestionMarkCircleIcon class="h-5 w-5 muted-link" />
            </router-link>
          </div>
          <input v-model="batchVolumeInput" type="text" id="batch-volume" class="mt-2 w-full input-text"
              placeholder="Enter an integer" :disabled="beta.inputDisabled" />
        </div>

        <!-- Min Value -->
        <div class="flex-1">
          <div class="flex items-center justify-between space-x-2">
            <label for="min-value" class="label-text">Min Value</label>
            <router-link to="/about">
              <QuestionMarkCircleIcon class="h-5 w-5 muted-link" />
            </router-link>
          </div>
          <input v-model="minValueInput" type="text" id="min-value" class="mt-2 w-full input-text"
                 placeholder="Enter a value" :disabled="beta.inputDisabled" />
        </div>

        <!-- Max Value -->
        <div class="flex-1">
          <div class="flex items-center justify-between space-x-2">
            <label for="max-value" class="label-text">Max Value</label>
            <router-link to="/about">
              <QuestionMarkCircleIcon class="h-5 w-5 muted-link" />
            </router-link>
          </div>
          <input v-model="maxValueInput" type="text" id="max-value" class="mt-2 w-full input-text"
              placeholder="Enter a value" :disabled="beta.inputDisabled" />
        </div>
      </div>

      <div>
        <!-- Sampling data -->
        <div class="flex items-center justify-between space-x-2">
          <label for="sampling-data" class="label-text">
            Sampling Data
            <span class="w-auto muted-link text-xs p-4">
              load from file
            </span>
          </label>
          <router-link to="/about">
            <QuestionMarkCircleIcon class="h-5 w-5 muted-link" />
          </router-link>
        </div>
        <textarea v-model="beta.samplingData" id="sampling-data" rows="4"
          class="mt-2 w-full h-[12rem] input-text"
          placeholder="Enter numbers separated with new line, comma or space" 
          :disabled="beta.inputDisabled">
        </textarea>
      </div>

        <!-- Submit Button -->
      <div>
        <div class="text-center">
          <button @click="submitData" class="primary-button" 
            :disabled="beta.inputDisabled">Analyze
          </button>
        </div>
      </div>
    </div>
</template>