import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import fs from 'fs'
import path from 'path'

// Plugin to embed public/help.md as a virtual module
function helpMarkdownPlugin() {
  const virtualModuleId = 'virtual:help-markdown'
  const resolvedVirtualModuleId = '\0' + virtualModuleId

  return {
    name: 'help-markdown-plugin',
    resolveId(id) {
      if (id === virtualModuleId) {
        return resolvedVirtualModuleId
      }
    },
    load(id) {
      if (id === resolvedVirtualModuleId) {
        const helpPath = path.resolve(__dirname, 'public/help.md')
        const content = fs.readFileSync(helpPath, 'utf-8')
        return `export default ${JSON.stringify(content)}`
      }
    }
  }
}

export default defineConfig(({ command }) => {
  const config = {
    plugins: [vue(), helpMarkdownPlugin()],
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
