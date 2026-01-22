<script setup>
import { useBetaStore } from "../../store";
import { useI18n } from "vue-i18n";

const { t } = useI18n()
const beta = useBetaStore();
</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">

    <table class="table-auto border-collapse border w-full mb-4 table-results">
      <thead>
        <tr>
          <th class="border p-2 text-center">
            <strong>{{ t('beta.params.title') }}</strong>
          </th>
          <th class="border p-2 text-center">
            <strong>&#945;</strong>
          </th>
          <th class="border p-2 text-center">
            <strong>&#946;</strong>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td class="border p-2 text-left">
            {{ t('beta.params.min-quality') }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.paramsMin[0]?.toFixed(2) ?? 'N/A' }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.paramsMin[1]?.toFixed(2) ?? 'N/A' }}
          </td>
        </tr>
        <tr>
          <td class="border p-2 text-left">
            {{ t('beta.params.max-quality') }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.paramsMax[0]?.toFixed(2) ?? 'N/A' }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.paramsMax[1]?.toFixed(2) ?? 'N/A' }}
          </td>
        </tr>
        <tr>
          <td class="border p-2 text-left">
            {{ t('beta.params.predicted-quality') }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.predictedParams[0]?.toFixed(2) ?? 'N/A' }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.predictedParams[1]?.toFixed(2) ?? 'N/A' }}
          </td>
        </tr>
        <tr>
          <td class="border p-2 text-left">
            {{ t('beta.params.sampling-quality') }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.samplingParams[0]?.toFixed(2) ?? 'N/A' }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.samplingParams[1]?.toFixed(2) ?? 'N/A' }}
          </td>
        </tr>
        <tr v-if="beta.testMode === true">
          <td class="border p-2 text-left">
            {{ t('beta.params.test-mode-quality') }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.testModeParams[0]?.toFixed(2) ?? 'N/A' }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.testModeParams[1]?.toFixed(2) ?? 'N/A' }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
