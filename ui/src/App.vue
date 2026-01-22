<script setup>
  import { HomeIcon, InformationCircleIcon, SunIcon, MoonIcon } from "@heroicons/vue/24/outline";
  import { useThemeStore, useLanguageStore } from "./store";
  import { useI18n } from "vue-i18n";
  import { ref } from "vue";

  const { t } = useI18n();
  const theme = useThemeStore();
  const language = useLanguageStore();
  const showBanner = ref(true);
</script>

<template>
  <div id="app" class="flex flex-col min-h-screen">

    <div v-if="showBanner" class="flex items-center justify-center mt-2 mb-2">
      <span class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-full bg-yellow-50 border border-yellow-300 text-yellow-700 font-semibold">
      🚧 This application is in active development. Features may change or break unexpectedly. 🚧
        <button
        @click="showBanner = false"
        class="ml-2 px-2 py-0.5 rounded-full bg-yellow-100 border border-yellow-300 text-yellow-700 hover:bg-yellow-200 transition"
        aria-label="Close"
      >
        <strong>&times;</strong>
      </button>
    </span>
      
    </div>

    <header class="h-16 shadow flex items-center px-4 bg-backgroundSecondary text-text border-b border-border">
      <!-- Logo and App Name -->
      <div class="flex items-center space-x-2">
        <!-- SVG Logo -->
        <div class="flex items-center justify-center h-16 w-16">
          <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 100 100"
              preserveAspectRatio="xMidYMid meet"
              class="h-full w-full"
          >
            <!-- Background -->
            <rect width="100" height="100" fill="none" />
            <!-- Logo Placeholder -->
            <rect x="10" y="10" width="80" height="80" fill="gray" stroke="gray" stroke-width="3" />
            <text x="50" y="55" text-anchor="middle" fill="black" font-size="18" font-family="Arial, sans-serif">
              Logo
            </text>
          </svg>
        </div>

        <!-- App Name -->
        <span class="text-xl font-bold"> {{t('header.name')}} </span>
      </div>

      <!-- Spacer -->
      <div class="flex-1"></div>

      <!-- Navigation Buttons -->
      <div class="flex items-center space-x-4">

        <router-link to="/" class="flex items-center px-3 py-2 hover:opacity-80">
          <HomeIcon class="h-6 w-6 text-current" />
          <span class="ml-2">{{ t('header.home') }}</span>
        </router-link>

        <router-link to="/help" class="flex items-center px-3 py-2 hover:opacity-80">
          <InformationCircleIcon class="h-6 w-6 text-current" />
          <span class="ml-2">{{ t('header.help') }}</span>
        </router-link>

        <!-- Theme Toggle Button -->
        <button @click="theme.toggleTheme" class="flex items-center px-3 py-2 hover:opacity-80">
          <template v-if="theme.currentTheme === 'light'">
            <MoonIcon class="h-6 w-6 text-current" />
          </template>
          <template v-else>
            <SunIcon class="h-6 w-6 text-current" />
          </template>
        </button>

        <!-- Language Toggle Button -->
        <button
            @click="language.toggleLanguage"
            class="flex items-center justify-center h-10 w-10 hover:opacity-80"
        >
          <template v-if="language.currentLanguage === 'en-us'">
            <span class="fi fi-us text-lg grayscale"></span>
          </template>
          <template v-else>
            <span class="fi fi-br text-lg grayscale"></span>
          </template>
        </button>
      </div>
    </header>

    <main class="flex-grow flex-col bg-background flex justify-center">
      <router-view />
    </main>

    <footer class="h-24 bg-backgroundSecondary text-text border-t border-border p-4 flex flex-col items-center">
      <!-- Links -->
      <div class="flex items-center space-x-4">
        <!-- GitHub Link -->
        <a
            href="https://github.com/piiadov/quality_control_room"
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center space-x-2 hover:opacity-80 text-base"
        >
          <i class="fab fa-github text-lg"></i>
          <span>GitHub</span>
        </a>

        <!-- LinkedIn Link -->
        <a
            href="https://www.linkedin.com/in/vasilii-piiadov/"
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center space-x-2 hover:opacity-80 text-base"
        >
          <i class="fab fa-linkedin text-lg"></i>
          <span>LinkedIn</span>
        </a>

        <!-- Email for Feedback -->
        <a
            href="mailto:piyadov@alumni.usp.br"
            class="flex items-center space-x-2 hover:opacity-80 text-base"
        >
          <i class="fas fa-envelope text-lg"></i>
          <span>Feedback</span>
        </a>
      </div>

      <div class="mt-2 text-sm text-center" style="display: inline-flex; align-items: center; gap: 5px;">
        <span><img src="https://img.shields.io/badge/license-MIT-6C757D.svg?style=flat-square" alt="License: MIT"></span>
      </div>

    </footer>
  </div>
</template>

<style scoped>
</style>
