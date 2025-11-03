import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Dynamic port configuration from environment
const VITE_PORT = parseInt(process.env.VITE_PORT || '5175')
const RUST_PORT = parseInt(process.env.VITE_RUST_PORT || process.env.TABAGENT_BUILDER_PORT || '3000')

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
  server: {
    port: VITE_PORT,
    strictPort: false, // Allow fallback ports
    proxy: {
      '/api': {
        target: `http://localhost:${RUST_PORT}`,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, '/v1')
      }
    }
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
})
