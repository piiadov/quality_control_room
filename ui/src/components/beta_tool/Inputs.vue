<script setup>
import ApiService from '../../services/api.js'
import { ref, computed, watch } from 'vue';
import { settingsStore, betaStore, sidebarStore} from '../../store/index.js';
import { useI18n, } from 'vue-i18n';

const { t } = useI18n();
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

const populationSizeInput = ref(null);
const populationSizeInputTestMode = computed(() =>
  !isNaN(beta.populationSize) ? beta.populationSize : ''
);

const minValueInput = ref(null);
const minValueInputTestMode = computed(() =>
  !isNaN(beta.minValue) ? beta.minValue : ''
);
const maxValueInput = ref(null);
const maxValueInputTestMode = computed(() =>
  !isNaN(beta.maxValue) ? beta.maxValue : ''
);

const submitData = async () => {
  beta.errorMessage = "";
  if (!beta.testMode) {
    const popSize = parseInt(populationSizeInput.value, 10);
    if (isNaN(popSize) || popSize < 1) {
      beta.errorMessage = 'Population size: Please enter valid positive integer number';
      beta.populationSize = NaN;
      beta.showResults = false;
      return;
    } else {
      beta.populationSize = popSize;
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

    if (popSize < data.length) {
      beta.errorMessage = 'Population size must be greater than sample size';
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
  const api = new ApiService(settings.backendUrl, settings.connectTimeout);
  
  try {
    // Call fullAnalysis which handles all split commands
    const result = await api.fullAnalysis(
      beta.distribution,
      beta.samplingData,
      beta.minValue,
      beta.maxValue,
      beta.populationSize,
      beta.binsNumber
    );
    
    api.close();
    
    if (!result.success) {
      beta.errorMessage = 'Backend error: ' + result.message;
      beta.inputDisabled = false;
      beta.showResults = false;
      return;
    }
    
    // Update store with results
    beta.errorMessage = "";
    beta.inputDisabled = false;
    
    // From analyze command
    beta.sampleSize = result.sample_size;
    beta.populationSize = result.population_size;
    beta.minValue = result.min_value;
    beta.maxValue = result.max_value;
    beta.scaledData = result.scaled_data;
    beta.paramsMin = result.params_min;
    beta.paramsMax = result.params_max;
    beta.predictedParams = result.predicted_params;
    beta.samplingParams = result.sampling_params;
    beta.chi2Min = result.chi2_min;
    beta.chi2Max = result.chi2_max;
    beta.chi2Pred = result.chi2_pred;
    
    // From get_intervals
    beta.cdfMin = result.cdf_min;
    beta.cdfMax = result.cdf_max;
    
    // From get_cdf
    beta.domain = result.domain;
    beta.fittedCdfMin = result.fitted_cdf_min;
    beta.fittedCdfMax = result.fitted_cdf_max;
    beta.predictedCdf = result.predicted_cdf;
    beta.samplingCdf = result.sampling_cdf;
    
    // From get_pdf
    beta.fittedPdfMin = result.fitted_pdf_min;
    beta.fittedPdfMax = result.fitted_pdf_max;
    beta.predictedPdf = result.predicted_pdf;
    beta.samplingPdf = result.sampling_pdf;
    
    // From get_histogram
    beta.binEdges = result.bin_edges;
    beta.observedFreq = result.observed_freq;
    beta.expectedFreqMin = result.expected_freq_min;
    beta.expectedFreqMax = result.expected_freq_max;
    beta.expectedFreqPred = result.expected_freq_pred;
    
    beta.showResults = true;
    sidebar.sidebarResults = true;
    
  } catch (error) {
    api.close();
    beta.errorMessage = error.message;
    beta.inputDisabled = false;
    beta.showResults = false;
  }
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
          <div class="flex items-center justify-left space-x-2">
            <input type="checkbox" id="test-mode" v-model="beta.testMode"
                   :disabled="beta.inputDisabled"/>
            <label for="test-mode" class="label-text">{{ t('beta.inputs.test-mode') }}</label>
          </div>
      </div>

      <!--Error message-->
      <div v-if="beta.errorMessage.length > 0" class="error-message text-sm h-4">
        {{ beta.errorMessage }}
      </div>

        <!-- Discretization and Min/Max values -->
      <div class="flex space-x-4">
        <!-- Population size -->
        <div class="flex-1">
          <label for="population-size" class="label-text">{{ t('beta.inputs.discretization') }}</label>
          <input v-if="beta.testMode" v-model="populationSizeInputTestMode" type="text" id="population-size" class="mt-2 w-full input-text"
                :placeholder="t('beta.inputs.discretization-placeholder')" :disabled="beta.inputDisabled" readonly/>
          <input v-else v-model="populationSizeInput" type="text" id="population-size" class="mt-2 w-full input-text"
                :placeholder="t('beta.inputs.discretization-placeholder')" :disabled="beta.inputDisabled"/>

        </div>

        <!-- Min Value -->
        <div class="flex-1">
          <label for="min-value" class="label-text">{{ t('beta.inputs.min-value') }}</label>
          <input v-if="beta.testMode" v-model="minValueInputTestMode" type="text" id="min-value" class="mt-2 w-full input-text"
                 :placeholder="t('beta.inputs.min-value-placeholder')" :disabled="beta.inputDisabled" readonly/>
          <input v-else v-model="minValueInput" type="text" id="min-value" class="mt-2 w-full input-text"
                 :placeholder="t('beta.inputs.min-value-placeholder')" :disabled="beta.inputDisabled"/>
        </div>

        <!-- Max Value -->
        <div class="flex-1">
          <label for="max-value" class="label-text">{{ t('beta.inputs.max-value') }}</label>
          <input v-if="beta.testMode" v-model="maxValueInputTestMode" type="text" id="max-value" class="mt-2 w-full input-text"
              :placeholder="t('beta.inputs.max-value-placeholder')" :disabled="beta.inputDisabled" readonly/>
          <input v-else v-model="maxValueInput" type="text" id="max-value" class="mt-2 w-full input-text"
              :placeholder="t('beta.inputs.max-value-placeholder')" :disabled="beta.inputDisabled"/>
        </div>
      </div>

      <div>
        <!-- Sampling data -->
        <div class="flex items-center justify-between space-x-2">
          <label for="sampling-data" class="label-text">
            {{ t('beta.inputs.sampling-data') }}
          </label>
          <span
            v-if="!beta.testMode"
            class="w-auto muted-link text-xs p-0"
            @click="loadFile"
          >
            {{ t('beta.inputs.load-data') }}
          </span>
          <input
            ref="fileInput"
            type="file"
            accept=".txt, .csv"
            class="hidden"
            @change="handleFileUpload"
          />
        </div>
        <textarea v-if="beta.testMode" v-model="beta.samplingData" id="sampling-data" rows="4"
          class="mt-2 w-full h-[12rem] input-text"
          :placeholder="t('beta.inputs.sampling-data-placeholder')" 
          :disabled="beta.inputDisabled" readonly>
        </textarea>
        <textarea v-else v-model="beta.samplingData" id="sampling-data" rows="4"
          class="mt-2 w-full h-[12rem] input-text"
          :placeholder="t('beta.inputs.sampling-data-placeholder')" 
          :disabled="beta.inputDisabled">
        </textarea>
      </div>
        <!-- Submit Button -->
      <div>
        <div class="text-center">
          <button @click="submitData" class="primary-button" 
            :disabled="beta.inputDisabled">{{ t('beta.inputs.analyze') }}
          </button>
        </div>
      </div>
    </div>
</template>
