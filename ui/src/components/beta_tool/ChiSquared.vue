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
          {{ beta.binEdges.length - 1 }}
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          {{ t('beta.chi2.critical') }} &#967;<sup>2</sup> {{ t('beta.chi2.value') }}
        </td>
        <td class="border p-2 text-center" colspan="3">
          {{ beta.chi2Pred.critical_value?.toFixed(4) ?? 'N/A' }}
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          {{ t('beta.chi2.predicted-quality') }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.chi2Pred.chi2?.toFixed(4) ?? 'N/A' }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.chi2Pred.p_value?.toFixed(4) ?? 'N/A' }}
        </td>
        <td class="border p-2 text-center">
          <CheckIcon v-if="!beta.chi2Pred.reject_null" class="h-5 w-5 mx-auto"/>
          <XMarkIcon v-else class="h-5 w-5 mx-auto"/>
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          {{ t('beta.chi2.min-quality') }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.chi2Min.chi2?.toFixed(4) ?? 'N/A' }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.chi2Min.p_value?.toFixed(4) ?? 'N/A' }}
        </td>
        <td class="border p-2 text-center">
          <CheckIcon v-if="!beta.chi2Min.reject_null" class="h-5 w-5 mx-auto"/>
          <XMarkIcon v-else class="h-5 w-5 mx-auto"/>
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          {{ t('beta.chi2.max-quality') }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.chi2Max.chi2?.toFixed(4) ?? 'N/A' }}
        </td>
        <td class="border p-2 text-center">
          {{ beta.chi2Max.p_value?.toFixed(4) ?? 'N/A' }}
        </td>
        <td class="border p-2 text-center">
          <CheckIcon v-if="!beta.chi2Max.reject_null" class="h-5 w-5 mx-auto"/>
          <XMarkIcon v-else class="h-5 w-5 mx-auto"/>
        </td>
      </tr>
      </tbody>
    </table>
  </div>
</template>