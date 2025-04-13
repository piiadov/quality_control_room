<script setup>
import {betaResultsStore, betaInputStore} from "../../store/index.js";

const betaInputs = betaInputStore();
const betaResults = betaResultsStore();
</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">
    <table class="table-auto border-collapse border w-full table-results">
      <thead>
      <tr>
        <th class="border p-2 text-center" colspan="2">
          <strong>&#967;<sup>2</sup> Tests</strong>
        </th>
      </tr>
      </thead>
      <tbody>
      <tr>
        <td class="border p-2 text-left">
          Bins number
        </td>
        <td class="border p-2 text-center">
          *
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          &#967;<sup>2</sup> for predicted quality
        </td>
        <td class="border p-2 text-center">
          *
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          &#967;<sup>2</sup> for minimum quality
        </td>
        <td class="border p-2 text-center">
          *
        </td>
      </tr>
      <tr>
        <td class="border p-2 text-left">
          &#967;<sup>2</sup> for maximum quality
        </td>
        <td class="border p-2 text-center">
          *
        </td>
      </tr>
      <tr v-if="betaInputs.testMode === true">
        <td class="border p-2 text-left">
          &#967;<sup>2</sup> for true quality (test mode)
        </td>
        <td class="border p-2 text-center">
          *
        </td>
      </tr>
      </tbody>
    </table>
  </div>
</template>

