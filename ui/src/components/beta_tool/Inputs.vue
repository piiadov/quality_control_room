<script setup>
import WebSocketService from '../../services/websocketService.js'
import { ref, computed } from 'vue';
import { settingsStore, betaInputStore, betaResultsStore, sidebarStore} from '../../store/index.js';
import { QuestionMarkCircleIcon } from "@heroicons/vue/24/outline/index.js";

const settings = settingsStore();
const betaInput = betaInputStore();
const betaResults = betaResultsStore();
const sidebar = sidebarStore();

const isDisabled = ref(false);
const errorMessage = ref(null);

function stringToNumberArray(str) {
  return str !== "" && str.match(/-?\d*\.?\d+/g) ? str.match(/-?\d*\.?\d+/g).map(Number) : [];
}

const batchVolumeInput = computed(() =>
  !isNaN(betaInput.batchVolume) ? betaInput.batchVolume : ''
);

const minValueInput = computed(() =>
  !isNaN(betaInput.minValue) ? betaInput.minValue : ''
);

const maxValueInput = computed(() =>
  !isNaN(betaInput.maxValue) ? betaInput.maxValue : ''
);

const submitData = () => {
  errorMessage.value = null;

  const volume = parseInt(betaInput.batchVolume.toString(), 10);
  if ((isNaN(volume) || volume < 1) && !betaInput.testMode) {
    errorMessage.value = 'Dscretization: Please enter valid positive integer number';
    betaResults.showResults = false;
    return;
  } else {
    betaInput.batchVolume = volume;
  }

  const minValue = parseFloat(betaInput.minValue.toString());
  const maxValue = parseFloat(betaInput.maxValue.toString());
  if ((isNaN(minValue) || isNaN(maxValue)) && !betaInput.testMode) {
    errorMessage.value = 'Min or Max value: Please enter valid float number';
    betaResults.showResults = false;
    return;
  } else {
    betaInput.minValue = minValue;
    betaInput.maxValue = maxValue;
  }

  if ((minValue >= maxValue) && !betaInput.testMode) {
    errorMessage.value = 'Min value must be less than max value';
    betaResults.showResults = false;
    return;
  }

  const data = stringToNumberArray(betaInput.samplingData.toString());
  if (data.length === 0 && !betaInput.testMode) {
    errorMessage.value = 'Sampling data: Please enter valid float numbers';
    betaResults.showResults = false;
    return;
  } else {
    betaInput.samplingData = data;
  }

  if (volume < data.length && !betaInput.testMode) {
    errorMessage.value = 'Batch volume or discretization factor must be greater than sampling size';
    betaResults.showResults = false;
    return;
  }

  const areAllValuesInRange = data.every(value => value >= betaInput.minValue && value <= betaInput.maxValue);
  if (!areAllValuesInRange && !betaInput.testMode) {
    errorMessage.value = 'Sampling data: All values must be within the specified range';
    betaResults.showResults = false;
    return;
  }

  isDisabled.value = true;
  const ws = new WebSocketService(settings.backendUrl, settings.connectTimeout);
  ws.connectAndSendData('calc', betaInput)
    .then(response => {
      if (response.data.error > 0) {
        errorMessage.value = 'Backend error: ' + response.data.info;
        isDisabled.value = false;
        betaResults.showResults = false;
      } else {
        errorMessage.value = null;
        isDisabled.value = false;

        betaInput.batchVolume = response.data.population_size;
        betaInput.minValue = response.data.min_value;
        betaInput.maxValue = response.data.max_value;
        betaInput.samplingData = response.data.data;

        betaResults.info = response.data.info;
        betaResults.scaledData = response.data.scaled_data;
        betaResults.cdfMin = response.data.cdf_min;
        betaResults.cdfMax = response.data.cdf_max;
        betaResults.q = response.data.q;
        betaResults.fittedCdfMin = response.data.fitted_cdf_min;
        betaResults.fittedCdfMax = response.data.fitted_cdf_max;
        betaResults.fittedPdfMin = response.data.fitted_pdf_min;
        betaResults.fittedPdfMax = response.data.fitted_pdf_max;
        betaResults.betaParamsMin = response.data.beta_params_min;
        betaResults.betaParamsMax = response.data.beta_params_max;
        betaResults.predictedBetaParams = response.data.predicted_beta_params;
        betaResults.predictedCdf = response.data.predicted_cdf;
        betaResults.predictedPdf = response.data.predicted_pdf;
        betaResults.testModeBetaParams = response.data.test_mode_beta_params;
        betaResults.testModeCdf = response.data.test_mode_cdf;
        betaResults.testModePdf = response.data.test_mode_pdf;
        betaResults.bins = response.data.bins;
        betaResults.freq = response.data.freq;
        betaResults.showResults = true;
        sidebar.sidebarResults = true;
      }
    })
    .catch(error => {
      errorMessage.value = error.message;
      isDisabled.value = false;
      betaResults.showResults = false;
    });
};

</script>

<template>
    <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">

        <!-- Message if test mode -->
        <div v-if="betaInput.testMode" class="h-2 info-message text-sm">
            Test mode
        </div>

        <!--Error message-->
        <div v-if="errorMessage" class="error-message text-sm h-4">{{ errorMessage }}</div>

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
                    placeholder="Enter an integer" />
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
                    placeholder="Enter a value" />
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
                    placeholder="Enter a value" />
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
            <textarea v-model="betaInput.samplingData" id="sampling-data" rows="4"
                class="mt-2 w-full h-[12rem] input-text"
                placeholder="Enter numbers separated with new line, comma or space"></textarea>
        </div>

        <!-- Submit Button -->
        <div>
            <div class="text-center">
                <button @click="submitData" class="primary-button" :disabled="isDisabled">Analyze</button>
            </div>
        </div>
    </div>
</template>