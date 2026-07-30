import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig(({ mode }) => {
  const isDesktop = mode === 'desktop'
  const isWeb = mode === 'web'

  return {
    plugins: [vue()],

    root: isDesktop ? 'src' : (isWeb ? 'src-tauri/static' : '.'),

    build: {
      outDir: isDesktop ? '../dist-desktop' : (isWeb ? '../dist-web' : '../dist'),
      emptyOutDir: false
    },

    server: {
      port: 5173,
      strictPort: true,
      proxy: {
        '/api': {
          target: 'http://localhost:3000',
          changeOrigin: true
        }
      }
    },

    resolve: {
      alias: {
        '@': resolve(__dirname, 'src'),
        '@web': resolve(__dirname, 'src-tauri/static')
      }
    },

    // 确保 Tauri dev 能正确识别
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_']
  }
})