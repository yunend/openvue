import { ref } from 'vue'
import { useToast } from './useToast'

interface ServerStatus {
  isRunning: boolean
  port: number | string
  staticFolder: string
  urls: string[]
}

const isRunning = ref(false)
const status = ref<ServerStatus>({
  isRunning: false,
  port: '-',
  staticFolder: '-',
  urls: []
})

export function useServerControl() {
  const { showToast } = useToast()

  async function refreshStatus(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      const result = await invoke('get_server_status') as ServerStatus
      status.value = result
      isRunning.value = result.isRunning
    } catch (e) {
      console.error('刷新状态失败:', e)
      showToast('刷新状态失败: ' + e, 'error')
    }
  }

  async function startServer(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      const msg = await invoke('start_server') as string
      showToast('✅ ' + msg, 'success')
      setTimeout(refreshStatus, 300)
    } catch (e) {
      showToast('❌ ' + e, 'error')
    }
  }

  async function stopServer(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      const msg = await invoke('stop_server') as string
      showToast('🛑 ' + msg, 'success')
      setTimeout(refreshStatus, 200)
    } catch (e) {
      showToast('❌ ' + e, 'error')
    }
  }

  async function restartHttpService(): Promise<void> {
    try {
      showToast('🔄 正在重启 HTTP 服务...', 'info')
      if (isRunning.value) {
        try {
          const { invoke } = window.__TAURI__.core
          await invoke('stop_server')
          await new Promise(resolve => setTimeout(resolve, 500))
        } catch (e) {
          console.warn('停止服务时出错:', e)
        }
      }
      await startServer()
      setTimeout(refreshStatus, 300)
    } catch (e) {
      showToast('❌ 重启 HTTP 服务失败: ' + e, 'error')
    }
  }

  return {
    isRunning,
    status,
    refreshStatus,
    startServer,
    stopServer,
    restartHttpService
  }
}