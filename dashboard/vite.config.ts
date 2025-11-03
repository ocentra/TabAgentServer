import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// Dynamic port configuration from environment
const VITE_PORT = parseInt(process.env.VITE_PORT || '5173')
const RUST_PORT = parseInt(process.env.VITE_RUST_PORT || process.env.TABAGENT_RUST_PORT || '3000')

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react({
      babel: {
        plugins: [['babel-plugin-react-compiler']],
      },
    }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@/components': path.resolve(__dirname, './src/components'),
      '@/hooks': path.resolve(__dirname, './src/hooks'),
      '@/lib': path.resolve(__dirname, './src/lib'),
      '@/types': path.resolve(__dirname, './src/types'),
      '@/stores': path.resolve(__dirname, './src/stores'),
      '@/pages': path.resolve(__dirname, './src/pages'),
    },
  },
  server: {
    port: VITE_PORT,
    strictPort: false, // Allow fallback ports
    proxy: {
      '/v1': {
        target: `http://localhost:${RUST_PORT}`,
        changeOrigin: true,
      },
      '/ws': {
        target: `ws://localhost:${RUST_PORT}`,
        ws: true,
      },
    },
  },
})
