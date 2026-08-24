import { ref } from 'vue'
import { useToast } from './useToast'
import { i18n } from '../i18n'

interface AppConfig {
  port: number
  publicFolder: string
  enableUpload: boolean
}

const config = ref<AppConfig>({
  port: 8005,
  publicFolder: 'public',
  enableUpload: false
})

export function useConfigManager() {
  const { showToast } = useToast()

  async function loadConfig(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      const cfg = await invoke('get_config') as AppConfig
      config.value = {
        port: cfg.port || 8005,
        publicFolder: cfg.publicFolder || 'public',
        enableUpload: !!cfg.enableUpload
      }
      showToast(i18n.global.t('toast.configLoaded'), 'info')
    } catch (e) {
      showToast(i18n.global.t('toast.configLoadFailed', { err: String(e) }), 'error')
    }
  }

  async function saveConfig(newConfig: AppConfig): Promise<boolean> {
    const { port, publicFolder, enableUpload } = newConfig

    if (!port || port < 1 || port > 65535) {
      showToast(i18n.global.t('toast.invalidPort'), 'error')
      return false
    }

    if (!publicFolder) {
      showToast(i18n.global.t('toast.emptyFolder'), 'error')
      return false
    }

    try {
      const { invoke } = window.__TAURI__.core
      await invoke('save_config', {
        port: parseInt(String(port), 10),
        publicFolder,
        enableUpload
      })
      config.value = { port: parseInt(String(port), 10), publicFolder, enableUpload: !!enableUpload }
      showToast(i18n.global.t('toast.saved'), 'success')
      return true
    } catch (e) {
      showToast(i18n.global.t('toast.saveFailed', { err: String(e) }), 'error')
      return false
    }
  }

  async function browseFolder(initialDir?: string | null): Promise<string | null> {
    try {
      const { invoke } = window.__TAURI__.core
      const chosen = await invoke('choose_folder', { initialDir: initialDir || null }) as string | null
      if (chosen) {
        showToast(i18n.global.t('toast.folderSelected', { path: chosen }), 'info')
      }
      return chosen
    } catch (e) {
      showToast(i18n.global.t('toast.browseFailed', { err: String(e) }), 'error')
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