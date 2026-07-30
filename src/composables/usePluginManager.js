import { ref, computed } from 'vue'
import { useToast } from './useToast'

const pluginsCache = ref({ extensions: {} })
const pluginsFilter = ref('all')

export function usePluginManager() {
  const { showToast } = useToast()

  const filteredPlugins = computed(() => {
    const exts = Object.keys(pluginsCache.value.extensions || {}).sort()
    
    if (pluginsFilter.value === 'all') {
      return exts.map(ext => ({ ext, ...pluginsCache.value.extensions[ext] }))
    }
    
    return exts
      .filter(ext => {
        const entry = pluginsCache.value.extensions[ext]
        const statusKey = mapStatus(entry.status)
        return statusKey === pluginsFilter.value
      })
      .map(ext => ({ ext, ...pluginsCache.value.extensions[ext] }))
  })

  async function loadPluginsConfig() {
    try {
      const { invoke } = window.__TAURI__.core
      pluginsCache.value = await invoke('get_plugins_config')
      showToast('🧩 插件配置已加载', 'info')
    } catch (e) {
      console.error('加载插件配置失败:', e)
      showToast('加载插件配置失败: ' + e, 'error')
    }
  }

  async function togglePlugin(ext, newStatus) {
    if (!newStatus || newStatus === '__') return
    
    try {
      const { invoke } = window.__TAURI__.core
      showToast(`💾 正在更新 .${ext} 状态...`, 'info')
      const msg = await invoke('save_plugin_extension_status', { ext, status: newStatus })
      await loadPluginsConfig()
      showToast(msg, 'success')
    } catch (e) {
      console.error(e)
      showToast('❌ 切换失败: ' + e, 'error')
    }
  }

  function filterPlugins(type) {
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

function mapStatus(status) {
  const map = {
    'BrowserDefault': 'browser-default',
    'Enabled': 'enabled',
    'Disabled': 'disabled',
    'Undeveloped': 'undeveloped'
  }
  return map[status] || status || 'browser-default'
}

export function fileExtIcon(ext) {
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