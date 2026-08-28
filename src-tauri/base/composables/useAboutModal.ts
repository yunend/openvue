import { ref, computed } from 'vue'

interface PluginHandler {
  handlerId: string
  status: string
  pluginId?: string
  urlTemplate?: string
  description?: string
  name?: string
}

interface ExtensionConfig {
  handlers: PluginHandler[]
  activeHandlerId?: string | null
}

/** 旧格式（兼容） */
interface LegacyPluginEntry {
  status: string
  pluginId?: string
  urlTemplate?: string
  description?: string
  name?: string
}

interface NormalizedPluginEntry {
  ext: string
  activeHandlerId?: string | null
  activeStatus: string
  activeHandler?: PluginHandler | null
}

export function useAboutModal() {
  const aboutLoading = ref(false)
  const aboutData = ref<AboutData | null>(null)
  const aboutError = ref<string | null>(null)
  const copiedConfig = ref(false)
  // 存储标准化后的扩展名配置（统一为新格式形态）
  const aboutPlugins = ref<Record<string, NormalizedPluginEntry> | null>(null)

  async function loadAboutInfo(): Promise<void> {
    aboutLoading.value = true
    aboutError.value = null
    try {
      const [aboutRes, pluginsRes] = await Promise.all([
        fetch('/api/about'),
        fetch('/api/plugins'),
      ])

      if (!aboutRes.ok) throw new Error('HTTP ' + aboutRes.status)
      aboutData.value = await aboutRes.json()

      if (pluginsRes.ok) {
        const pluginsData: { extensions: Record<string, ExtensionConfig | LegacyPluginEntry> } = await pluginsRes.json()
        const raw = pluginsData.extensions || {}
        const normalized: Record<string, NormalizedPluginEntry> = {}

        for (const ext of Object.keys(raw)) {
          const v = raw[ext]
          let handlers: PluginHandler[] = []
          let activeId: string | null | undefined

          if (Array.isArray((v as ExtensionConfig).handlers)) {
            // 新格式：直接用
            const cfg = v as ExtensionConfig
            handlers = cfg.handlers
            activeId = cfg.activeHandlerId
          } else {
            // 旧格式：包一层
            const old = v as LegacyPluginEntry
            const hId = old.pluginId || 'default'
            handlers = [{
              handlerId: hId,
              status: old.status,
              pluginId: old.pluginId,
              urlTemplate: old.urlTemplate,
              description: old.description,
              name: old.name,
            }]
            activeId = old.status === 'enabled' ? hId : null
          }

          const activeHandler = activeId
            ? handlers.find(h => h.handlerId === activeId)
            : handlers.find(h => h.status === 'enabled')

          normalized[ext] = {
            ext,
            activeHandlerId: activeId,
            activeStatus: activeHandler?.status ?? handlers[0]?.status ?? 'browser-default',
            activeHandler: activeHandler ?? null,
          }
        }
        aboutPlugins.value = normalized
      } else {
        aboutPlugins.value = null
      }
    } catch (e) {
      aboutError.value = e instanceof Error ? e.message : String(e)
    } finally {
      aboutLoading.value = false
    }
  }

  async function copyConfigToClipboard(prettyConfigJson: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(prettyConfigJson)
      copiedConfig.value = true
      setTimeout(() => (copiedConfig.value = false), 1500)
    } catch (e) {
      alert('复制失败，请手动选择文本复制')
    }
  }

  function statusLabel(status: string): string {
    const map: Record<string, string> = {
      enabled: '已启用',
      disabled: '未启用',
      'browser-default': '浏览器默认',
      undeveloped: '未开发',
    }
    return map[status] || status
  }

  function statusBadgeClass(status: string): string {
    const map: Record<string, string> = {
      enabled: 'bg-green-100 text-green-800 border border-green-300',
      disabled: 'bg-red-100 text-red-800 border border-red-300',
      'browser-default': 'bg-blue-100 text-blue-800 border border-blue-300',
      undeveloped: 'bg-gray-200 text-gray-600 border border-gray-300',
    }
    return map[status] || 'bg-gray-100 text-gray-600'
  }

  return {
    aboutLoading,
    aboutData,
    aboutError,
    aboutPlugins,
    loadAboutInfo,
    copiedConfig,
    copyConfigToClipboard,
    statusLabel,
    statusBadgeClass,
  }
}