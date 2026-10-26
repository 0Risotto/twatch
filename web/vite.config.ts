import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: { '@': path.resolve(fileURLToPath(new URL('.', import.meta.url)), 'src') },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:7878',
        changeOrigin: false,
        ws: true,
      },
    },
  },
  build: {
    target: 'es2022',
    sourcemap: true,
  },
})
