import { ref } from 'vue'
import { useToast } from './useToast'

const autoStartEnabled = ref(false)

export function useSystemSettings() {
  const { showToast } = useToast()
  const { isRunning } = useServerControl()

  async function initAutostartStatus() {
    try {
      const { invoke } = window.__TAURI__.core
      autoStartEnabled.value = await invoke('plugin:autostart|is_enabled')
    } catch (e) {
      console.error('获取自启动状态失败:', e)
    }
  }

  async function toggleAutostart(shouldEnable) {
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

  async function hideToTray() {
    try {
      const { invoke } = window.__TAURI__.core
      await invoke('hide_window')
      showToast('📦 已最小化到系统托盘', 'info')
    } catch (e) {
      showToast('隐藏失败: ' + e, 'error')
    }
  }

  async function quitApp() {
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

function useServerControl() {
  return { isRunning: ref(false) }
}