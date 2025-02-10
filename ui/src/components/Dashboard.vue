<script setup>
import { useI18n, } from 'vue-i18n';
import { useSidebarStore } from '../store/index';
import QualityProfileSidebarResults from './quality_profile/SidebarResults.vue';
import DefectsRateSidebarResults from './defects_rate/SidebarResults.vue';

const { t } = useI18n();
const sidebarStore = useSidebarStore();
</script>

<template>
  <div class="flex flex-grow">
    <!-- Toolbox -->
    <aside class="w-64 p-4 bg-backgroundSecondary text-text">
      <div class="p-4 text-lg font-bold border-b border-border">
        {{ t('sidebar.tools') }}
      </div>

      <div class="space-y-4 p-4">
        <router-link to="/tools/defects-rate">
          <button class="secondary-button mb-4 min-w-full"
                  :class="[sidebarStore.activeTool === 'DefectsRate'? 'active-tool': '']">
            {{ t('home.defect-rate-btn') }}
          </button>
        </router-link>

        <router-link to="/tools/quality-profile">
          <button class="secondary-button mb-4 min-w-full"
                  :class="[sidebarStore.activeTool === 'QualityProfile'? 'active-tool': '']">
            {{ t('home.quality-profile-btn') }}
          </button>
        </router-link>
      </div>

      <QualityProfileSidebarResults
          v-if="sidebarStore.activeTool === 'QualityProfile' && sidebarStore.sidebarResults" />
      <DefectsRateSidebarResults
          v-if="sidebarStore.activeTool === 'DefectsRate' && sidebarStore.sidebarResults" />
    </aside>

    <div class="flex-1 p-6">
      <router-view />
    </div>
  </div>
</template>

<style scoped>
</style>