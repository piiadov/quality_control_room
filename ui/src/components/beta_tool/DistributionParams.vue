<script setup>
import {betaStore} from "../../store/index.js";

const beta = betaStore();
</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">

    <table class="table-auto border-collapse border w-full table-results">
      <thead>
        <tr>
          <th class="border p-2 text-center">
            <strong>Beta - Distribution</strong>
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
            Minimum Quality
          </td>
          <td class="border p-2 text-center">
            {{ beta.betaParamsMin[0].toFixed(1) }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.betaParamsMin[1].toFixed(1) }}
          </td>
        </tr>
        <tr>
          <td class="border p-2 text-left">
            Maximum Quality
          </td>
          <td class="border p-2 text-center">
            {{ beta.betaParamsMax[0].toFixed(1) }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.betaParamsMax[1].toFixed(1) }}
          </td>
        </tr>
        <tr>
          <td class="border p-2 text-left">
            Predicted Quality
          </td>
          <td class="border p-2 text-center">
            {{ beta.predictedBetaParams[0].toFixed(1) }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.predictedBetaParams[1].toFixed(1) }}
          </td>
        </tr>
        <tr v-if="beta.testMode === true">
          <td class="border p-2 text-left">
            True Quality (test mode)
          </td>
          <td class="border p-2 text-center">
            {{ beta.testModeBetaParams[0].toFixed(1) }}
          </td>
          <td class="border p-2 text-center">
            {{ beta.testModeBetaParams[1].toFixed(1) }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
