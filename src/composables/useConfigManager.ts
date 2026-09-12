import { ref } from 'vue'
import { useToast } from './useToast'
import { i18n } from '../i18n'

export interface AppConfig {
  port: number
  publicFolder: string
  enableUpload: boolean
  /** 用户自定义插件根目录（后端总返回，无配置时为 null） */
  pluginsFolder: string | null
  /** 实际生效的插件目录（由后端计算返回，只读展示用） */
  effectivePluginsDir: string
}

const config = ref<AppConfig>({
  port: 8005,
  publicFolder: 'public',
  enableUpload: false,
  pluginsFolder: null,
  effectivePluginsDir: ''
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
        enableUpload: !!cfg.enableUpload,
        pluginsFolder: cfg.pluginsFolder ?? null,
        effectivePluginsDir: cfg.effectivePluginsDir || ''
      }
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
      const msg = await invoke('save_config', {
        port: parseInt(String(port), 10),
        publicFolder,
        enableUpload
      }) as string
      if (msg.startsWith('__RESTART_FAILED__')) {
        const err = msg.slice('__RESTART_FAILED__'.length)
        showToast(i18n.global.t('toast.restartFailed', { err }), 'error')
      }
      config.value = {
        port: parseInt(String(port), 10),
        publicFolder,
        enableUpload: !!enableUpload,
        pluginsFolder: newConfig.pluginsFolder,
        effectivePluginsDir: config.value.effectivePluginsDir
      }
      return true
    } catch (e) {
      showToast(i18n.global.t('toast.saveFailed', { err: String(e) }), 'error')
    }
    return false
  }

  /** 设置插件根目录（独立命令，不通过 save_config） */
  async function setPluginsFolder(folderPath: string): Promise<boolean> {
    try {
      const { invoke } = window.__TAURI__.core
      const msg = await invoke('set_plugins_folder', { folderPath }) as string
      if (msg.startsWith('__RESTART_FAILED__')) {
        const err = msg.slice('__RESTART_FAILED__'.length)
        showToast(i18n.global.t('toast.restartFailed', { err }), 'error')
      }
      await loadConfig()
      return true
    } catch (e) {
      showToast(i18n.global.t('toast.saveFailed', { err: String(e) }), 'error')
    }
    return false
  }

  /** 重置插件根目录为默认 */
  async function resetPluginsFolder(): Promise<boolean> {
    try {
      const { invoke } = window.__TAURI__.core
      const msg = await invoke('reset_plugins_folder') as string
      if (msg.startsWith('__RESTART_FAILED__')) {
        const err = msg.slice('__RESTART_FAILED__'.length)
        showToast(i18n.global.t('toast.restartFailed', { err }), 'error')
      }
      await loadConfig()
      return true
    } catch (e) {
      showToast(i18n.global.t('toast.saveFailed', { err: String(e) }), 'error')
    }
    return false
  }

  /** 用系统文件管理器打开当前插件目录 */
  async function openPluginsFolder(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      await invoke('open_plugins_folder')
    } catch (e) {
      showToast(i18n.global.t('toast.browseFailed', { err: String(e) }), 'error')
    }
  }

  async function browseFolder(initialDir?: string | null, title?: string): Promise<string | null> {
    try {
      const { invoke } = window.__TAURI__.core
      const chosen = await invoke('choose_folder', {
        initialDir: initialDir || null,
        title: title || null
      }) as string | null
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
    browseFolder,
    setPluginsFolder,
    resetPluginsFolder,
    openPluginsFolder
  }
}