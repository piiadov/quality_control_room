<script setup>
import { WrenchScrewdriverIcon, DocumentChartBarIcon } from "@heroicons/vue/24/outline/index.js";
import {useI18n} from "vue-i18n";

const { t } = useI18n();
</script>

<template>
  <div class="p-4 text-lg font-bold border-b border-border flex items-center justify-left">
    <DocumentChartBarIcon class="h-5 w-5 mr-2" />
    <span>{{ t('sidebar.results') }}</span>
  </div>
  <div class="space-y-4 p-4">
    <button class="results-button mb-0 min-w-full">
      Clean
    </button>
    <button class="results-button mb-4 min-w-full">
      Export
    </button>
    <button class="results-button mb-4 min-w-full">
      Generate report
    </button>
    <button class="results-button mb-4 min-w-full">
      Virtual engineer
    </button>
  </div>
</template>

<style scoped>

</style>