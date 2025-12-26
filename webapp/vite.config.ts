import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'node:path'

export default defineConfig({
  base: '/app/',
  plugins: [react()],
  build: {
    outDir: '../static',
    emptyOutDir: false, // Don't delete favicon.ico, 404.html etc
    rollupOptions: {
      output: {
        entryFileNames: 'app.bundle.js',
        chunkFileNames: '[name].bundle.js',
        assetFileNames: '[name].[ext]'
      }
    }
  },
  resolve: {
    alias: {
      'signaler-client': path.resolve(__dirname, '../client-lib/src')
    }
  },
  server: {
    proxy: {
      '/ws': {
        target: 'ws://localhost:8080',
        ws: true
      }
    }
  }
})