import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// The Tauri shell (`../desktop`) is configured to consume this dev server
// (port 5173) and the production bundle at `dist/`.
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: 'esbuild',
    sourcemap: false,
  },
})
