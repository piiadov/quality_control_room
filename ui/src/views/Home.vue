<script setup>
import { useI18n, } from 'vue-i18n';
const { t } = useI18n();
</script>

<template>
  <div class="text-center">
    <h3 class="text-4xl font-semibold text-text mb-8">{{ t('home.title') }}:</h3>

    <div class="space-x-4">
      <router-link to="/tools/defects-rate">
        <button class="primary-button">{{ t('home.defect-rate-btn') }}</button>
      </router-link>

      <router-link to="/tools/beta-profile">
        <button class="primary-button">{{ t('home.beta-profile-btn') }}</button>
      </router-link>
      <router-link to="/tools/normal-profile">
        <button class="primary-button">{{ t('home.normal-profile-btn') }}</button>
      </router-link>
    </div>

    <div class="m-4 text-mutedText space-x-4">
      <p>
        {{ t('home.help-link.begin-text') }}
        <router-link to="/about">
          <span class="theme-link">{{ t('home.help-link.link-text') }}</span>
        </router-link>
        {{ t('home.help-link.end-text') }}
      </p>
    </div>
  </div>
</template>

<style scoped>
</style>
