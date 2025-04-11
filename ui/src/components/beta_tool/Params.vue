<script setup>
import {betaResultsStore} from "../../store/index.js";

</script>

<template>
  <div class="min-w-lg bg-backgroundSecondary p-8 rounded-lg shadow-lg space-y-4">

    <div v-if="info" class="text-blue-500 min-h-8">
      info
    </div>

    <table class="table-auto border-collapse border border-gray-300 w-full">
      <tbody>
      <tr v-for="row in 4" :key="row">
        <td v-for="col in 2" :key="col" class="border border-gray-300 p-2 text-center">
        Row {{ row }}, Col {{ col }}
        </td>
      </tr>
      </tbody>
    </table>

  </div>
</template>
