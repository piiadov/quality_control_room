<script setup>
import WebSocketService from '../../services/websocketService.js'
import { ref, onMounted } from 'vue';
import { useSettings, useSidebarStore, useQualityProfile } from '../../store/index.js';
import { QuestionMarkCircleIcon } from "@heroicons/vue/24/outline/index.js";
import Results from './Results.vue';

const sidebarStore = useSidebarStore();
const settingsStore = useSettings();
const qualityProfileStore = useQualityProfile();

const isDisabled = ref(false);
const batchVolume = ref('');
const samplingData = ref('');
const errorBatchVolume = ref(null);
const errorSamplingData = ref(null);
const errorServerResponse = ref(null);

onMounted(() => {
  sidebarStore.activeTool = "QualityProfile";

  if (qualityProfileStore.batchVolume > 0) {
    batchVolume.value = qualityProfileStore.batchVolume.toString();
  }

  if (qualityProfileStore.samplingData.length > 0) {
    samplingData.value = qualityProfileStore.samplingData.join(', ');
  }

});

function stringToNumberArray(str) {
  return str !== "" && str.match(/-?\d*\.?\d+/g) ? str.match(/-?\d*\.?\d+/g).map(Number) : [];
}

function stringToPositiveInteger(str) {
  return /^[1-9]\d*$/.test(str) ? parseInt(str, 10) :null;
}

const submitData = () => {
  errorBatchVolume.value = null;
  errorSamplingData.value = null;

  const volume = stringToPositiveInteger(batchVolume.value.toString());
  if (!volume) {
    errorBatchVolume.value = 'Please enter valid positive integer number';
    qualityProfileStore.showResults = false;
    return;
  }

  const data = stringToNumberArray(samplingData.value.toString());
  if (data.length === 0) {
    errorSamplingData.value = 'Please enter valid float numbers';
    qualityProfileStore.showResults = false;
    return;
  }

  if (volume < data.length) {
    errorBatchVolume.value = 'Batch volume or discretization factor must be greater than sampling size';
    qualityProfileStore.showResults = false;
    return;
  }

  // alert("Sending volume " + volume + " and data " + data);
  isDisabled.value = true;
  const ws = new WebSocketService(settingsStore.backendUrl, settingsStore.connectTimeout);
  ws.connectAndSendData('calc', data)
      .then(response => {
        errorServerResponse.value = null;
        isDisabled.value = false;
        qualityProfileStore.batchVolume = volume;
        qualityProfileStore.samplingData = data;
        qualityProfileStore.info = response.data.info;
        qualityProfileStore.x = response.data.x;
        qualityProfileStore.q = response.data.q;
        qualityProfileStore.showResults = true;
      })
      .catch(error => {
        errorServerResponse.value = error.message;
        isDisabled.value = false;
        qualityProfileStore.showResults = false;
      });
};
</script>

<template>
  <header class="text-left mb-2 text-3xl font-semibold text-text p-4">
    <h1>Quantitative Quality Profiler</h1>
  </header>
  <main class="flex flex-1 justify-start gap-4 p-4">
    <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-6">
      <div>
        <div class="flex items-center justify-between space-x-2">
          <label for="batch-volume" class="block text-lg text-text">Batch volume/discretization</label>
          <router-link to="/about">
            <QuestionMarkCircleIcon class="h-5 w-5 muted-link " />
          </router-link>
        </div>
        <input
            v-model="batchVolume"
            type="text"
            id="batch-volume"
            class="mt-2 w-full p-3"
            placeholder="Enter an integer"
        />
        <div class="h-2 error-message text-sm mt-2">{{ errorBatchVolume }}</div>
      </div>

      <div>
        <!-- Textarea Field -->
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
            v-model="samplingData"
            id="sampling-data"
            rows="4"
            class="mt-2 w-full p-3 border border-border-color"
            placeholder="Enter numbers separated with new line, comma or space"
        ></textarea>
        <div class="h-2 error-message text-sm mt-2">{{ errorSamplingData }}</div>
      </div>

      <div>
        <div class="text-center">
          <button @click="submitData" class="primary-button" :disabled="isDisabled">Analyze</button>
        </div>
        <div class="h-2 error-message text-sm mt-2">{{ errorServerResponse }}</div>
      </div>
    </div>
    <Results v-if="qualityProfileStore.showResults"/>
  </main>
</template>

<style scoped>
</style>