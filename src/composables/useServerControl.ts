import { ref } from 'vue'
import { useToast } from './useToast'
import { i18n } from '../i18n'

interface ServerStatus {
  isRunning: boolean
  port: number | string
  publicFolder: string
  urls: string[]
}

const isRunning = ref(false)
const status = ref<ServerStatus>({
  isRunning: false,
  port: '-',
  publicFolder: '-',
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
      console.error('refresh status failed:', e)
      showToast(i18n.global.t('toast.refreshFailed', { err: String(e) }), 'error')
    }
  }

  async function startServer(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      await invoke('start_server')
      showToast(i18n.global.t('toast.started'), 'success')
      setTimeout(refreshStatus, 300)
    } catch (e) {
      showToast(i18n.global.t('toast.startFailed', { err: String(e) }), 'error')
    }
  }

  async function stopServer(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      await invoke('stop_server')
      showToast(i18n.global.t('toast.stopped'), 'success')
      setTimeout(refreshStatus, 200)
    } catch (e) {
      showToast(i18n.global.t('toast.stopFailed', { err: String(e) }), 'error')
    }
  }

  async function restartHttpService(): Promise<void> {
    try {
      showToast(i18n.global.t('toast.restarting'), 'info')
      if (isRunning.value) {
        try {
          const { invoke } = window.__TAURI__.core
          await invoke('stop_server')
          await new Promise(resolve => setTimeout(resolve, 500))
        } catch (e) {
          console.warn('stop during restart failed:', e)
        }
      }
      await startServer()
      setTimeout(refreshStatus, 300)
    } catch (e) {
      showToast(i18n.global.t('toast.restartFailed', { err: String(e) }), 'error')
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