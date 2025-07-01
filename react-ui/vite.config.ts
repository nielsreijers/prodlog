import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: '../src/ui/static/react',
    emptyOutDir: true,
  },
  server: {
    proxy: {
      '/api': 'http://localhost:8080',
      '/redact': 'http://localhost:8080',
      '/diffcontent': 'http://localhost:8080'
    }
  }
}) 