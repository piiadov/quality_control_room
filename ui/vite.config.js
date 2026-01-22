import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import fs from 'fs'

export default defineConfig(({ command }) => {
  const config = {
    plugins: [vue()],
    server: {
      host: 'localhost',
      port: 5725,
    }
  }

  // Only add HTTPS in dev mode if certificates exist
  if (command === 'serve') {
    const keyPath = '/etc/ssl/private/quality-control.io.key'
    const certPath = '/etc/ssl/certs/quality-control.io-fullchain.crt'
    
    if (fs.existsSync(keyPath) && fs.existsSync(certPath)) {
      config.server.https = {
        key: fs.readFileSync(keyPath),
        cert: fs.readFileSync(certPath),
      }
    }
  }

  return config
})
