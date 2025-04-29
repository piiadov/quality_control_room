import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import cssnano from 'cssnano';

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  css: {
    postcss: {
      plugins: [
        cssnano({
          preset: 'default',
        }),
      ],
    },
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    assets: {
      css: {
        mimeType: 'text/css',
      },
      js: {
        mimeType: 'application/javascript',
      },
      html: {
        mimeType: 'text/html',
      },
    },
  },
})
