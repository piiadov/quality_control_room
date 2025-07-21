import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import fs from 'fs'

export default defineConfig({
  plugins: [vue()],
  server: {
    host: '191.252.60.9',
    port: '5725',
    https: {
      key: fs.readFileSync('/home/vp/letsencrypt-copy/quality-control.io/privkey1.pem'),
      cert: fs.readFileSync('/home/vp/letsencrypt-copy/quality-control.io/fullchain1.pem'),
    },
  }
})
