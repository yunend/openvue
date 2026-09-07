import { ref, computed } from 'vue'

interface PluginHandler {
  handlerId: string
  pluginId?: string
  urlTemplate?: string
  description?: string
  name?: string
}

interface ExtensionConfig {
  handlers: PluginHandler[]
  activeHandlerId?: string | null
}

interface NormalizedPluginEntry {
  ext: string
  activeHandlerId?: string | null
  activeHandler?: PluginHandler | null
}

export function useAboutModal() {
  const aboutLoading = ref(false)
  const aboutData = ref<AboutData | null>(null)
  const aboutError = ref<string | null>(null)
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
        const pluginsData: { extensions: Record<string, ExtensionConfig> } = await pluginsRes.json()
        const raw = pluginsData.extensions || {}
        const normalized: Record<string, NormalizedPluginEntry> = {}

        for (const ext of Object.keys(raw)) {
          const cfg = raw[ext]
          const handlers = cfg.handlers || []
          const activeId = cfg.activeHandlerId ?? null

          const activeHandler = activeId
            ? handlers.find(h => h.handlerId === activeId)
            : undefined

          normalized[ext] = {
            ext,
            activeHandlerId: activeId,
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

  return {
    aboutLoading,
    aboutData,
    aboutError,
    aboutPlugins,
    loadAboutInfo,
  }
}