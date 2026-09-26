import { ref, computed } from 'vue'
import { useToast } from './useToast'
import { i18n } from '../i18n'

/** 单个处理器（一个扩展名可以有多个） */
export interface PluginHandler {
  handlerId: string
  name?: string
  description?: string
  pluginId?: string
  urlTemplate?: string
}

/** 一个扩展名的配置：多个处理器 + 激活项 id */
export interface ExtensionConfig {
  handlers: PluginHandler[]
  activeHandlerId?: string | null
}

interface PluginsData {
  extensions: Record<string, ExtensionConfig>
}

/** 前端列表渲染用：一个扩展 + 该扩展所有处理器 + 激活 id */
export interface PluginItem {
  ext: string
  handlers: PluginHandler[]
  activeHandlerId?: string | null
}

const pluginsCache = ref<PluginsData>({ extensions: {} })

export function fileExtIcon(ext: string): string {
  const e = ext.toLowerCase()
  if (['jpg','jpeg','png','gif','svg','bmp','webp'].includes(e)) return '🖼️'
  if (['mp4','avi','mov','mkv','webm','flv'].includes(e)) return '🎬'
  if (['mp3','wav','flac','aac','ogg'].includes(e)) return '🎵'
  if (['pdf'].includes(e)) return '📕'
  if (['doc','docx'].includes(e)) return '📘'
  if (['xls','xlsx','csv'].includes(e)) return '📗'
  if (['ppt','pptx'].includes(e)) return '📙'
  if (['zip','rar','7z','tar','gz'].includes(e)) return '🗜️'
  if (['md'].includes(e)) return '📝'
  if (['txt','log'].includes(e)) return '📄'
  if (['html','htm'].includes(e)) return '🌐'
  if (['ggb'].includes(e)) return '📐'
  return '📎'
}

export function usePluginManager() {
  const { showToast } = useToast()

  const filteredPlugins = computed<PluginItem[]>(() => {
    const exts = Object.keys(pluginsCache.value.extensions || {}).sort()

    return exts
      .map(ext => {
        const cfg = pluginsCache.value.extensions[ext] as ExtensionConfig | undefined
        if (!cfg || !cfg.handlers?.length) return null

        // 兼容旧格式：若后端因缓存仍返回老数据结构，自动包一层
        const handlers: PluginHandler[] = Array.isArray(cfg.handlers)
          ? cfg.handlers
          : [{ ...(cfg as unknown as Record<string, unknown>), handlerId: 'default' } as unknown as PluginHandler]

        const activeId = cfg.activeHandlerId ?? null
        return {
          ext,
          handlers,
          activeHandlerId: activeId ?? null
        } as PluginItem
      })
      .filter((item): item is PluginItem => !!item)
  })

  async function loadPluginsConfig(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      pluginsCache.value = await invoke('get_plugins_config') as PluginsData
    } catch (e) {
      console.error('load plugins failed:', e)
      showToast(i18n.global.t('toast.pluginsLoadFailed', { err: String(e) }), 'error')
    }
  }

  /** 切回浏览器默认（activeHandlerId 设为 null） */
  async function setBrowserDefault(ext: string): Promise<void> {
    if (!ext) return
    try {
      const { invoke } = window.__TAURI__.core
      await invoke('set_plugin_browser_default', { ext }) as string
      await loadPluginsConfig()
    } catch (e) {
      console.error(e)
      showToast(i18n.global.t('toast.handlerActivateFailed', { err: String(e) }), 'error')
    }
  }

  /** 把某扩展的指定处理器设为激活 */
  async function activateHandler(ext: string, handlerId: string): Promise<void> {
    if (!ext || !handlerId) return
    try {
      const { invoke } = window.__TAURI__.core
      await invoke('activate_plugin_handler', { ext, handlerId }) as string
      await loadPluginsConfig()
    } catch (e) {
      console.error(e)
      showToast(i18n.global.t('toast.handlerActivateFailed', { err: String(e) }), 'error')
    }
  }

  async function getPluginsDir(): Promise<string> {
    const { invoke } = window.__TAURI__.core
    return await invoke('get_plugins_dir') as string
  }

  async function addCustomPlugin(ext: string, folderPath: string): Promise<void> {
    if (!ext || !folderPath) {
      showToast(i18n.global.t('toast.customPluginEmpty'), 'error')
      return
    }
    try {
      const { invoke } = window.__TAURI__.core
      await invoke('add_custom_plugin', { ext, folderPath }) as string
      await loadPluginsConfig()
      showToast(i18n.global.t('toast.customPluginAdded', { ext }), 'success')
    } catch (e) {
      showToast(i18n.global.t('toast.customPluginFailed', { err: String(e) }), 'error')
    }
  }

  async function removeCustomPlugin(ext: string, folderPath: string): Promise<void> {
    if (!ext || !folderPath) {
      showToast(i18n.global.t('toast.customPluginRemoveEmpty'), 'error')
      return
    }
    try {
      const { invoke } = window.__TAURI__.core
      const result = await invoke('remove_custom_plugin', { ext, folderPath }) as string
      await loadPluginsConfig()
      if (result.startsWith('__RESTART_FAILED__')) {
        showToast(i18n.global.t('toast.customPluginRemoved', { ext }), 'success')
        showToast(i18n.global.t('toast.restartFailed', { err: result.slice('__RESTART_FAILED__'.length) }), 'error')
      } else {
        showToast(i18n.global.t('toast.customPluginRemoved', { ext }), 'success')
      }
    } catch (e) {
      showToast(i18n.global.t('toast.customPluginRemoveFailed', { err: String(e) }), 'error')
    }
  }

  return {
    filteredPlugins,
    loadPluginsConfig,
    activateHandler,
    setBrowserDefault,
    getPluginsDir,
    addCustomPlugin,
    removeCustomPlugin
  }
}