import { ref } from 'vue'
import { useToast } from './useToast'

const config = ref({
  port: 8005,
  staticFolder: 'static',
  enableUpload: false
})

export function useConfigManager() {
  const { showToast } = useToast()

  async function loadConfig() {
    try {
      const { invoke } = window.__TAURI__.core
      const cfg = await invoke('get_config')
      config.value = {
        port: cfg.port || 8005,
        staticFolder: cfg.staticFolder || 'static',
        enableUpload: !!cfg.enableUpload
      }
      showToast('📥 配置已读取', 'info')
    } catch (e) {
      showToast('读取配置失败: ' + e, 'error')
    }
  }

  async function saveConfig(newConfig) {
    const { port, staticFolder, enableUpload } = newConfig
    
    if (!port || port < 1 || port > 65535) {
      showToast('请输入有效的端口号 (1-65535)', 'error')
      return false
    }
    
    if (!staticFolder) {
      showToast('请输入公共文件目录路径', 'error')
      return false
    }

    try {
      const { invoke } = window.__TAURI__.core
      const msg = await invoke('save_config', { 
        port: parseInt(port, 10), 
        staticFolder, 
        enableUpload 
      })
      config.value = { port: parseInt(port, 10), staticFolder, enableUpload: !!enableUpload }
      showToast('💾 ' + msg, 'success')
      return true
    } catch (e) {
      showToast('保存失败: ' + e, 'error')
      return false
    }
  }

  async function browseFolder(initialDir) {
    try {
      const { invoke } = window.__TAURI__.core
      const chosen = await invoke('choose_folder', { initialDir: initialDir || null })
      if (chosen) {
        showToast('📁 已选择目录: ' + chosen, 'info')
      }
      return chosen
    } catch (e) {
      showToast('打开文件夹选择失败: ' + e, 'error')
      return null
    }
  }

  return {
    config,
    loadConfig,
    saveConfig,
    browseFolder
  }
}