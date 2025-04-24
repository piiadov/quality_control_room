<script setup>
import { sidebarStore, betaStore, normalStore, defectsStore } from "../store/index.js";
import {WrenchScrewdriverIcon, DocumentChartBarIcon} from "@heroicons/vue/24/outline/index.js";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const sidebar = sidebarStore();
const beta = betaStore();
const normal = normalStore();
const defects = defectsStore();
</script>

<template>
  <div class="flex flex-grow">
    <!-- Toolbox -->
    <aside class="w-64 min-w-[16rem] p-4 bg-backgroundSecondary text-text">

      <div class="p-4 text-lg font-bold border-b border-border flex items-center justify-left">
        <WrenchScrewdriverIcon class="h-5 w-5 mr-2" />
        <span>{{ t('sidebar.tools') }}</span>
      </div>

      <div class="space-y-4 p-4">
        <router-link to="/tools/normal-profile">
          <button class="secondary-button mb-4 min-w-full"
                  :class="[sidebar.activeTool === normal? 'active-tool': '']">
            {{ t('home.normal-profile-btn') }}
          </button>
        </router-link>

        <router-link to="/tools/beta-profile">
          <button class="secondary-button mb-4 min-w-full"
                  :class="[sidebar.activeTool === beta? 'active-tool': '']">
            {{ t('home.beta-profile-btn') }}
          </button>
        </router-link>

        <router-link to="/tools/defects-rate">
          <button class="secondary-button mb-4 min-w-full"
                  :class="[sidebar.activeTool === defects? 'active-tool': '']">
            {{ t('home.defect-rate-btn') }}
          </button>
        </router-link>
      </div>

      <div v-if="sidebar.activeTool !== undefined && sidebar.activeTool.showResults">
        <div class="p-4 text-lg font-bold border-b border-border flex items-center justify-left">
          <DocumentChartBarIcon class="h-5 w-5 mr-2" />
          <span>{{ t('sidebar.results') }}</span>
        </div>
        <div class="space-y-4 p-4">
            <button class="results-button mb-0 min-w-full" @click="sidebar.activeTool.resetState">
              {{ t('sidebar.clean') }}
            </button>
          <button class="results-button mb-4 min-w-full" @click="sidebar.activeTool.exportResults">
            {{ t('sidebar.export') }}
          </button>
          <button class="results-button mb-4 min-w-full">
            {{ t('sidebar.gen-report') }}
          </button>
          <button class="results-button mb-4 min-w-full">
            {{ t('sidebar.virt-engineer') }}
          </button>
        </div>
      </div>

    </aside>

    <div class="flex-1 p-6">
      <router-view />
    </div>
  </div>
</template>

<style scoped>
</style>