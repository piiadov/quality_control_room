<script setup>
import WebSocketService from '../../services/websocketService.js'
import { ref, computed, watch } from 'vue';
import { settingsStore, betaStore, sidebarStore} from '../../store/index.js';

const settings = settingsStore();
const beta = betaStore();
const sidebar = sidebarStore();

const fileInput = ref(null);

const stringToNumberArray = (str) => {
  return str !== "" && str.match(/-?\d*\.?\d+/g) ? str.match(/-?\d*\.?\d+/g).map(Number) : [];
};

const loadFile = () => {
  fileInput.value.click();
};

const handleFileUpload = (event) => {
  const file = event.target.files[0];
  if (file) {
    const reader = new FileReader();
    reader.onload = (e) => {
      beta.samplingData = e.target.result;
    };
    reader.readAsText(file);
  }
};

const batchVolumeInput = ref(null);
const batchVolumeInputTestMode = computed(() =>
  !isNaN(beta.batchVolume) ? beta.batchVolume : ''
);

const minValueInput = ref(null);
const minValueInputTestMode = computed(() =>
  !isNaN(beta.minValue) ? beta.minValue : ''
);
const maxValueInput = ref(null);
const maxValueInputTestMode = computed(() =>
  !isNaN(beta.maxValue) ? beta.maxValue : ''
);

const submitData = () => {
  beta.errorMessage = "";
  if (!beta.testMode) {
    const volume = parseInt(batchVolumeInput.value, 10);
    if (isNaN(volume) || volume < 1) {
      beta.errorMessage = 'Dscretization: Please enter valid positive integer number';
      beta.batchVolume = NaN;
      beta.showResults = false;
      return;
    } else {
      beta.batchVolume = volume;
    }

    const minValue = parseFloat(minValueInput.value);
    const maxValue = parseFloat(maxValueInput.value);
    if (isNaN(minValue) || isNaN(maxValue)) {
      beta.errorMessage = 'Min or Max value: Please enter valid float number';
      beta.minValue = NaN;
      beta.maxValue = NaN;
      beta.showResults = false;
      return;
    } else {
      beta.minValue = minValue;
      beta.maxValue = maxValue;
    }

    if (minValue >= maxValue) {
      beta.errorMessage = 'Min value must be less than max value';
      beta.minValue = NaN;
      beta.maxValue = NaN;
      beta.showResults = false;
      return;
    }

    const data = stringToNumberArray(beta.samplingData.toString());
    if (data.length === 0) {
      beta.errorMessage = 'Sampling data: Please enter valid float numbers';
      beta.showResults = false;
      return;
    } else {
      beta.samplingData = data;
    }

    if (volume < data.length) {
      beta.errorMessage = 'Batch volume or discretization factor must be greater than sampling size';
      beta.showResults = false;
      return;
    }

    const areAllValuesInRange = data.every(value => value >= beta.minValue && value <= beta.maxValue);
    if (!areAllValuesInRange) {
      beta.errorMessage = 'Sampling data: All values must be within the specified range';
      beta.showResults = false;
      return;
    }
  }

  beta.inputDisabled = true;
  const ws = new WebSocketService(settings.backendUrl, settings.connectTimeout);
  ws.connectAndSendData('calc', beta)
    .then(response => {
      if (response.data.error > 0) {
        beta.errorMessage = 'Backend error: ' + response.data.info;
        beta.inputDisabled = false;
        beta.showResults = false;
      } else {
        beta.errorMessage = "";
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
        beta.binsNumber = response.data.bins.length - 1;
        beta.predictedChi2 = response.data.predicted_chi2;
        beta.minChi2 = response.data.min_chi2;
        beta.maxChi2 = response.data.max_chi2;
        beta.testModeChi2 = response.data.test_mode_chi2;
        beta.predictedPval = response.data.predicted_pval;
        beta.minPval = response.data.min_pval;
        beta.maxPval = response.data.max_pval;
        beta.testModePval = response.data.test_mode_pval;
        beta.critVal = response.data.crit_val;
        beta.predictedDecision = response.data.predicted_decision;
        beta.minDecision = response.data.min_decision;
        beta.maxDecision = response.data.max_decision;
        beta.testModeDecision = response.data.test_mode_decision;
        beta.showResults = true;
        sidebar.sidebarResults = true;
      }
    })
    .catch(error => {
      beta.errorMessage = error.message;
      beta.inputDisabled = false;
      beta.showResults = false;
    });
};

watch(() => beta.testMode, (newValue) => {
  // Think on:
  // is there a better way to resetState on testMode changed? 
  // may it cause a trigger loop with v-model in checkbox?
  beta.resetState();
  beta.testMode = newValue; 
});

</script>

<template>
    <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
      <!-- Test mode -->
      <div class="h-2">
          <div class="flex items-center justify-center space-x-2">
            <input type="checkbox" id="test-mode" v-model="beta.testMode"
                   :disabled="beta.inputDisabled"/>
            <label for="test-mode" class="label-text">Test Mode</label>
          </div>
      </div>

      <!--Error message-->
      <div v-if="beta.errorMessage.length > 0" class="error-message text-sm h-4">
        {{ beta.errorMessage }}
      </div>

        <!-- Discretization and Min/Max values -->
      <div class="flex space-x-4">
        <!-- Batch volume/discretization -->
        <div class="flex-1">
          <label for="batch-volume" class="label-text">Discretization</label>
          <input v-if="beta.testMode" v-model="batchVolumeInputTestMode" type="text" id="batch-volume" class="mt-2 w-full input-text"
                placeholder="Enter an integer" :disabled="beta.inputDisabled" readonly/>
          <input v-else v-model="batchVolumeInput" type="text" id="batch-volume" class="mt-2 w-full input-text"
                placeholder="Enter an integer" :disabled="beta.inputDisabled"/>

        </div>

        <!-- Min Value -->
        <div class="flex-1">
          <label for="min-value" class="label-text">Min Value</label>
          <input v-if="beta.testMode" v-model="minValueInputTestMode" type="text" id="min-value" class="mt-2 w-full input-text"
                 placeholder="Enter a value" :disabled="beta.inputDisabled" readonly/>
          <input v-else v-model="minValueInput" type="text" id="min-value" class="mt-2 w-full input-text"
                 placeholder="Enter a value" :disabled="beta.inputDisabled"/>
        </div>

        <!-- Max Value -->
        <div class="flex-1">
          <label for="max-value" class="label-text">Max Value</label>
          <input v-if="beta.testMode" v-model="maxValueInputTestMode" type="text" id="max-value" class="mt-2 w-full input-text"
              placeholder="Enter a value" :disabled="beta.inputDisabled" readonly/>
          <input v-else v-model="maxValueInput" type="text" id="max-value" class="mt-2 w-full input-text"
              placeholder="Enter a value" :disabled="beta.inputDisabled"/>
        </div>
      </div>

      <div>
        <!-- Sampling data -->
        <div class="flex items-center justify-between space-x-2">
          <label for="sampling-data" class="label-text">
            Sampling Data
          </label>
          <span class="w-auto muted-link text-xs p-0" @click="loadFile">
              load from file
          </span>
          <input
            ref="fileInput"
            type="file"
            accept=".txt"
            class="hidden"
            @change="handleFileUpload"
          />
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