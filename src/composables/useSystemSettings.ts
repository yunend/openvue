import { ref, type Ref } from 'vue'
import { useToast } from './useToast'

interface UseSystemSettingsReturn {
  autoStartEnabled: Ref<boolean>
  initAutostartStatus: () => Promise<void>
  toggleAutostart: (shouldEnable: boolean) => Promise<void>
  hideToTray: () => Promise<void>
  quitApp: () => Promise<boolean>
}

interface UseLocalServerControlReturn {
  isRunning: Ref<boolean>
}

const autoStartEnabled = ref(false)

export function useSystemSettings(): UseSystemSettingsReturn {
  const { showToast } = useToast()
  const { isRunning } = useServerControl()

  async function initAutostartStatus(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      autoStartEnabled.value = await invoke('plugin:autostart|is_enabled') as boolean
    } catch (e) {
      console.error('获取自启动状态失败:', e)
    }
  }

  async function toggleAutostart(shouldEnable: boolean): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      if (shouldEnable) {
        await invoke('plugin:autostart|enable')
        showToast('✅ 已开启开机自启动', 'success')
      } else {
        await invoke('plugin:autostart|disable')
        showToast('❌ 已关闭开机自启动', 'success')
      }
      autoStartEnabled.value = shouldEnable
    } catch (e) {
      showToast('操作失败: ' + e, 'error')
      throw e
    }
  }

  async function hideToTray(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      await invoke('hide_window')
      showToast('📦 已最小化到系统托盘', 'info')
    } catch (e) {
      showToast('隐藏失败: ' + e, 'error')
    }
  }

  async function quitApp(): Promise<boolean> {
    if (isRunning.value && !confirm('⚠️ HTTP 服务正在运行，确定要退出吗？')) {
      return false
    }

    try {
      const { invoke } = window.__TAURI__.core
      await invoke('quit_app')
      return true
    } catch (e) {
      console.error(e)
      return false
    }
  }

  return {
    autoStartEnabled,
    initAutostartStatus,
    toggleAutostart,
    hideToTray,
    quitApp
  }
}

function useServerControl(): UseLocalServerControlReturn {
  return { isRunning: ref(false) }
}