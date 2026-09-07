import { ref } from 'vue'

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

interface PluginsData {
  extensions: Record<string, ExtensionConfig>
}

export function usePluginResolver() {
  const pluginsMap = ref<Record<string, ExtensionConfig>>({})

  async function loadPluginsMap(): Promise<void> {
    try {
      const response = await fetch('/api/plugins')
      if (response.ok) {
        const data: PluginsData = await response.json()
        const raw = data.extensions || ({} as Record<string, ExtensionConfig>)
        const normalized: Record<string, ExtensionConfig> = {}

        for (const ext of Object.keys(raw)) {
          const v = raw[ext]
          if (Array.isArray((v as ExtensionConfig).handlers)) {
            normalized[ext] = v as ExtensionConfig
          } else {
            // 兼容旧格式（理论上已不存在，但保留容错）
            const anyV = v as Record<string, unknown>
            const hId = (anyV.pluginId as string) || 'default'
            normalized[ext] = {
              handlers: [{
                handlerId: hId,
                pluginId: anyV.pluginId as string | undefined,
                urlTemplate: anyV.urlTemplate as string | undefined,
                description: anyV.description as string | undefined,
                name: anyV.name as string | undefined,
              }],
              activeHandlerId: null,
            }
          }
        }

        pluginsMap.value = normalized
        const totalHandlers = Object.values(normalized).reduce((a, c) => a + c.handlers.length, 0)
        console.log('🧩 插件映射表已加载，共', Object.keys(normalized).length, '个扩展名 /', totalHandlers, '个处理器')
      } else {
        console.warn('获取 /api/plugins 失败:', response.status)
      }
    } catch (e) {
      console.warn('加载插件配置失败（不影响使用，降级为浏览器默认打开）:', e)
    }
  }

  /**
   * 根据扩展名 + publicPath 算出【当前激活处理器】对应的插件打开 URL
   * activeHandlerId === null → 走浏览器默认
   * activeHandlerId 有值 → 找对应 handler，有 urlTemplate+pluginId 才返回 URL
   * @param ext 扩展名（不带点，会自动转小写）
   * @param publicPath 文件在服务端的公开路径
   * @returns 应打开的插件完整 URL；null = 走浏览器默认
   */
  function resolvePluginUrl(ext: string, publicPath: string): string | null {
    if (!ext) return null

    const key = ext.toLowerCase()
    const cfg = pluginsMap.value[key]
    if (!cfg || !cfg.handlers?.length) return null

    // activeHandlerId 为 null → 浏览器默认
    const activeId = cfg.activeHandlerId
    if (!activeId) return null

    // 找 activeHandlerId 对应的 handler
    const handler = cfg.handlers.find(h => h.handlerId === activeId)
    if (!handler) return null

    // 必须有 urlTemplate + pluginId
    const tpl = handler.urlTemplate
    const pid = handler.pluginId
    if (!tpl || !pid) return null

    return tpl
      .replace(/\{pluginId\}/g, pid)
      .replace(/\{publicPath\}/g, publicPath)
  }

  return {
    pluginsMap,
    loadPluginsMap,
    resolvePluginUrl,
  }
}