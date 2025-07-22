import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import fs from 'fs'

export default defineConfig({
  plugins: [vue()],
  server: {
    host: 'quality-control.io',
    port: '5725',
    https: {
      key: fs.readFileSync('/etc/ssl/private/quality-control.io.key'),
      cert: fs.readFileSync('/etc/ssl/certs/quality-control.io-fullchain.crt'),
    },
  }
})
