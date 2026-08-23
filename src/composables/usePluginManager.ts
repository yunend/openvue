import { ref, computed } from 'vue'
import { useToast } from './useToast'

type PluginStatus = 'browser-default' | 'enabled' | 'disabled' | 'undeveloped'

interface PluginEntry {
  status: string
  name?: string
  description?: string
  pluginId?: string
  urlTemplate?: string
}

interface PluginsData {
  extensions: Record<string, PluginEntry>
}

type PluginFilter = 'all' | 'enabled' | 'disabled' | 'browser-default' | 'undeveloped'

interface PluginItem {
  ext: string
  status: string
  name?: string
  description?: string
  pluginId?: string
  urlTemplate?: string
}

const pluginsCache = ref<PluginsData>({ extensions: {} })
const pluginsFilter = ref<PluginFilter>('all')

function mapStatus(status: string): PluginStatus {
  const map: Record<string, PluginStatus> = {
    'BrowserDefault': 'browser-default',
    'Enabled': 'enabled',
    'Disabled': 'disabled',
    'Undeveloped': 'undeveloped'
  }
  return map[status] || (status as PluginStatus) || 'browser-default'
}

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
      .filter(ext => {
        const entry = pluginsCache.value.extensions[ext]
        if (!entry) return false
        const statusKey = mapStatus(entry.status)
        return pluginsFilter.value === 'all' || statusKey === pluginsFilter.value
      })
      .map(ext => ({ ext, ...pluginsCache.value.extensions[ext]! }))
  })

  async function loadPluginsConfig(): Promise<void> {
    try {
      const { invoke } = window.__TAURI__.core
      pluginsCache.value = await invoke('get_plugins_config') as PluginsData
      showToast('🧩 插件配置已加载', 'info')
    } catch (e) {
      console.error('加载插件配置失败:', e)
      showToast('加载插件配置失败: ' + e, 'error')
    }
  }

  async function togglePlugin(ext: string, newStatus: string): Promise<void> {
    if (!newStatus || newStatus === '__') return

    try {
      const { invoke } = window.__TAURI__.core
      showToast(`💾 正在更新 .${ext} 状态...`, 'info')
      const msg = await invoke('save_plugin_extension_status', { ext, status: newStatus }) as string
      await loadPluginsConfig()
      showToast(msg, 'success')
    } catch (e) {
      console.error(e)
      showToast('❌ 切换失败: ' + e, 'error')
    }
  }

  function filterPlugins(type: PluginFilter): void {
    pluginsFilter.value = type
  }

  return {
    pluginsCache,
    pluginsFilter,
    filteredPlugins,
    loadPluginsConfig,
    togglePlugin,
    filterPlugins
  }
}