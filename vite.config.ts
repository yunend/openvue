import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// 需要加一个类型扩展
export default defineConfig(({ mode }: { mode: string }) => {
  const isDesktop = mode === 'desktop'
  const isWeb = mode === 'web'

  return {
    plugins: [vue()],
    root: isDesktop ? 'src' : (isWeb ? 'src-tauri/base' : '.'),
    build: {
      outDir: isDesktop ? '../dist-desktop' : (isWeb ? '../dist-web' : '../dist'),
      emptyOutDir: true
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
        '@web': resolve(__dirname, 'src-tauri/base')
      }
    },
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_']
  }
})