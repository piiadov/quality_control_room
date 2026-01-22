<script setup>
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useSettingsStore, useBetaStore, useSidebarStore } from '../../store';
import ApiService from '../../services/api.js';
import { validatePopulationSize, validateRange, validateSamplingData } from '../../utils/validation';

const { t } = useI18n();
const settings = useSettingsStore();
const beta = useBetaStore();
const sidebar = useSidebarStore();

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
    // Validate population size
    const popResult = validatePopulationSize(populationSizeInput.value);
    if (!popResult.valid) {
      beta.errorMessage = popResult.error;
      beta.populationSize = NaN;
      beta.showResults = false;
      return;
    }
    beta.populationSize = popResult.value;

    // Validate min/max range
    const rangeResult = validateRange(minValueInput.value, maxValueInput.value);
    if (!rangeResult.valid) {
      beta.errorMessage = rangeResult.error;
      beta.minValue = NaN;
      beta.maxValue = NaN;
      beta.showResults = false;
      return;
    }
    beta.minValue = rangeResult.min;
    beta.maxValue = rangeResult.max;

    // Validate sampling data
    const dataResult = validateSamplingData(
      beta.samplingData.toString(),
      rangeResult.min,
      rangeResult.max,
      popResult.value
    );
    if (!dataResult.valid) {
      beta.errorMessage = dataResult.error;
      beta.showResults = false;
      return;
    }
    beta.samplingData = dataResult.data;
  }

  beta.inputDisabled = true;
  const api = new ApiService(settings.backendUrl, settings.connectTimeout);
  
  try {
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
    
    // Use store action to update all fields
    beta.updateFromResult(result);
    beta.inputDisabled = false;
    beta.showResults = true;
    sidebar.sidebarResults = true;
    
  } catch (error) {
    api.close();
    beta.errorMessage = error.message;
    beta.inputDisabled = false;
    beta.showResults = false;
  }
};

// Test data configuration (hardcoded for now)
const TEST_CONFIG = {
  distribution: 0,  // Beta
  params: [2.0, 3.0],  // alpha=2, beta=3 (asymmetric)
  sampleSize: 50,
  populationSize: 1000,
  minValue: 0,
  maxValue: 100,
};

// Load test data from server
const loadTestData = async () => {
  const api = new ApiService(settings.backendUrl, settings.connectTimeout);
  
  try {
    const result = await api.generateTestData(
      TEST_CONFIG.distribution,
      TEST_CONFIG.params,
      TEST_CONFIG.sampleSize,
      TEST_CONFIG.minValue,
      TEST_CONFIG.maxValue
    );
    
    api.close();
    
    if (result.success) {
      beta.populationSize = TEST_CONFIG.populationSize;
      beta.minValue = result.min_value;
      beta.maxValue = result.max_value;
      beta.samplingData = result.test_data;
      
      // Store true distribution data for charts
      beta.testModeParams = result.test_params;
      beta.testModeCdf = result.test_cdf;
      beta.testModePdf = result.test_pdf;
      beta.domain = result.domain;
    } else {
      beta.errorMessage = 'Failed to load test data: ' + (result.message || 'Unknown error');
    }
  } catch (error) {
    api.close();
    beta.errorMessage = 'Failed to load test data: ' + error.message;
  }
};

watch(() => beta.testMode, (newValue) => {
  beta.resetState();
  beta.testMode = newValue;
  
  if (newValue) {
    // Load test data from server when test mode is enabled
    loadTestData();
  }
}, { immediate: true });  // Run immediately on mount to handle default testMode: true

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
