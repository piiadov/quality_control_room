<script setup>
import {CheckIcon, XMarkIcon} from "@heroicons/vue/24/outline/index.js";
import { betaStore } from "../../store/index.js";
import { useI18n } from "vue-i18n";

const { t } = useI18n()
const beta = betaStore();
</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <table class="table-auto border-collapse border w-full table-results">
      <colgroup>
        <col style="width: 55%;" />
        <col style="width: 14%;" />
        <col style="width: 17%;" />
        <col style="width: 14%;" />
      </colgroup>
      <thead>
      <tr>
        <th class="border p-2 text-center">
          <strong>{{ t('beta.chi2.title') }}</strong>
        </th>
        <th class="border p-2 text-center">
          <strong>&#967;<sup>2</sup></strong>
        </th>
        <th class="border p-2 text-center">
          <strong>{{ t('beta.chi2.p-value') }}</strong>
        </th>
        <th class="border p-2 text-center">
          <strong>{{ t('beta.chi2.decision') }}</strong>
        </th>
      </tr>
      </thead>
      <tbody>
      <tr>
        <td class="border p-2 text-left">
          {{ t('beta.chi2.bins-number') }}
        </td>
        <td class="border p-2 text-center" colspan="3">
          {{ beta.bins.length - 1 }}
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          {{ t('beta.chi2.critical') }} &#967;<sup>2</sup> {{ t('beta.chi2.value') }}
        </td>
        <td class="border p-2 text-center" colspan="3">
          {{ beta.critVal.toFixed(4) }}
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          {{ t('beta.chi2.predicted-quality') }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.predictedChi2.toFixed(4) }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.predictedPval.toFixed(4) }}
        </td>
        <td class="border p-2 text-center">
          <CheckIcon v-if="beta.predictedDecision" class="h-5 w-5 mx-auto"/>
          <XMarkIcon v-else class="h-5 w-5 mx-auto"/>
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          {{ t('beta.chi2.min-quality') }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.minChi2.toFixed(4) }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.minPval.toFixed(4) }}
        </td>
        <td class="border p-2 text-center">
          <CheckIcon v-if="beta.minDecision" class="h-5 w-5 mx-auto"/>
          <XMarkIcon v-else class="h-5 w-5 mx-auto"/>
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          {{ t('beta.chi2.max-quality') }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.maxChi2.toFixed(4) }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.maxPval.toFixed(4) }}
        </td>
        <td class="border p-2 text-center">
          <CheckIcon v-if="beta.maxDecision" class="h-5 w-5 mx-auto"/>
          <XMarkIcon v-else class="h-5 w-5 mx-auto"/>
        </td>
      </tr>
      <tr v-if="beta.testMode === true">
        <td class="border p-2 text-left">
          {{ t('beta.chi2.test-mode-quality') }}
        </td>
        <td class="border p-2 text-center">
          {{  beta.testModeChi2.toFixed(4) }}
        </td>
        <td class="border p-2 text-center">
          {{  beta.testModePval.toFixed(4) }}
        </td>
        <td class="border p-2 text-center">
          <CheckIcon v-if="beta.testModeDecision" class="h-5 w-5 mx-auto"/>
          <XMarkIcon v-else class="h-5 w-5 mx-auto"/>
        </td>
      </tr>
      </tbody>
    </table>
  </div>
</template>