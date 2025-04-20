<script setup>
import {CheckIcon, XMarkIcon} from "@heroicons/vue/24/outline/index.js";
import { betaStore } from "../../store/index.js";

const beta = betaStore();
</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <table class="table-auto border-collapse border w-full table-results">
      <thead>
      <tr>
        <th class="border p-2 text-center">
          <strong>Tests</strong>
        </th>
        <th class="border p-2 text-center">
          <strong>&#967;<sup>2</sup></strong>
        </th>
        <th class="border p-2 text-center">
          <strong>p-value</strong>
        </th>
        <th class="border p-2 text-center">
          <strong>Decision</strong>
        </th>
      </tr>
      </thead>
      <tbody>
      <tr>
        <td class="border p-2 text-left">
          Bins number
        </td>
        <td class="border p-2 text-center" colspan="3">
          {{ beta.bins.length - 1 }}
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          Critical &#967;<sup>2</sup>-value
        </td>
        <td class="border p-2 text-center" colspan="3">
          {{ beta.critVal.toFixed(4) }}
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          Predicted quality
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
          Minimum quality
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
          Maximum quality
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
          True quality (test mode)
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