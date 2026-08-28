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
    // web 模式下将 plugins 文件夹作为公共资源
    publicDir: isWeb ? resolve(__dirname, 'plugins') : undefined,
    build: {
      outDir: isDesktop ? '../dist-desktop' : (isWeb ? '../dist-web' : '../dist'),
      emptyOutDir: true,
      rollupOptions: isWeb ? {
        input: {
          main: resolve(__dirname, 'src-tauri/base/index.html'),
          '404': resolve(__dirname, 'src-tauri/base/404.html')
        }
      } : undefined
    },
    server: {
      port: 5173,
      strictPort: true,
      headers: {
        'X-Frame-Options': 'DENY',
        'Content-Security-Policy': "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' ipc.localhost https://api.github.com; frame-ancestors 'none'",
        'Referrer-Policy': 'same-origin',
      },
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