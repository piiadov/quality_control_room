<script setup>
import { useI18n, } from 'vue-i18n';
import { sidebarStore } from '../store/index';
import DefectsRateSidebarResults from './defects_rate/SidebarResults.vue';
import BetaSidebarResults from './beta_tool/SidebarResults.vue';
import NormalSidebarResults from './normal_tool/SidebarResults.vue';

const { t } = useI18n();
const sidebar = sidebarStore();
</script>

<template>
  <div class="flex flex-grow">
    <!-- Toolbox -->
    <aside class="w-64 min-w-[16rem] p-4 bg-backgroundSecondary text-text">
      <div class="p-4 text-lg font-bold border-b border-border">
        {{ t('sidebar.tools') }}
      </div>

      <div class="space-y-4 p-4">
        <router-link to="/tools/defects-rate">
          <button class="secondary-button mb-4 min-w-full"
                  :class="[sidebar.activeTool === 'DefectsRate'? 'active-tool': '']">
            {{ t('home.defect-rate-btn') }}
          </button>
        </router-link>

        <router-link to="/tools/beta-profile">
          <button class="secondary-button mb-4 min-w-full"
                  :class="[sidebar.activeTool === 'BetaProfile'? 'active-tool': '']">
            {{ t('home.beta-profile-btn') }}
          </button>
        </router-link>

        <router-link to="/tools/normal-profile">
          <button class="secondary-button mb-4 min-w-full"
                  :class="[sidebar.activeTool === 'NormalProfile'? 'active-tool': '']">
            {{ t('home.normal-profile-btn') }}
          </button>
        </router-link>
      </div>

      <DefectsRateSidebarResults
          v-if="sidebar.activeTool === 'DefectsRate' && sidebar.sidebarResults" />
      <BetaSidebarResults
          v-if="sidebar.activeTool === 'BetaProfile' && sidebar.sidebarResults" />
      <NormalSidebarResults
          v-if="sidebar.activeTool === 'NormalProfile' && sidebar.sidebarResults" />
    </aside>

    <div class="flex-1 p-6">
      <router-view />
    </div>
  </div>
</template>

<style scoped>
</style>