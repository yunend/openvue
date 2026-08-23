/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

// Tauri window 类型扩展
interface Window {
  __TAURI__: {
    core: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<any>
    }
  }
}