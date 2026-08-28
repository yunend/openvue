import { ref } from 'vue'

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

interface PluginsData {
  extensions: Record<string, ExtensionConfig>
}

/** 旧格式（兼容旧 plugins.json 的条目形态） */
interface LegacyPluginEntry {
  status: string
  pluginId?: string
  urlTemplate?: string
  description?: string
  name?: string
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
          const v = raw[ext] as unknown as ExtensionConfig | LegacyPluginEntry
          if (Array.isArray((v as ExtensionConfig).handlers)) {
            normalized[ext] = v as ExtensionConfig
          } else {
            // 兼容旧格式：包一层，handlerId 复用 pluginId，没有就用 'default'
            const old = v as LegacyPluginEntry
            const hId = old.pluginId || 'default'
            normalized[ext] = {
              handlers: [{
                handlerId: hId,
                status: old.status,
                pluginId: old.pluginId,
                urlTemplate: old.urlTemplate,
                description: old.description,
                name: old.name,
              }],
              activeHandlerId: old.status === 'enabled' ? hId : null,
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
   * 同一扩展名多个处理器时，只有 activeHandlerId 指向且 status=enabled 的才生效；
   * 否则回退到第一个 status=enabled 的条目；都没有返回 null 走浏览器默认
   * @param ext 扩展名（不带点，会自动转小写）
   * @param publicPath 文件在服务端的公开路径
   * @returns 应打开的插件完整 URL；null = 走浏览器默认
   */
  function resolvePluginUrl(ext: string, publicPath: string): string | null {
    if (!ext) return null

    const key = ext.toLowerCase()
    const cfg = pluginsMap.value[key]
    if (!cfg || !cfg.handlers?.length) return null

    // 1. 优先：activeHandlerId 指定的处理器
    let handler: PluginHandler | undefined = cfg.activeHandlerId
      ? cfg.handlers.find(h => h.handlerId === cfg.activeHandlerId)
      : undefined

    // 2. 若 activeHandlerId 指向的条目状态不是 enabled，退回第一个 enabled
    if (!handler || handler.status !== 'enabled') {
      handler = cfg.handlers.find(h => h.status === 'enabled')
    }
    if (!handler) return null

    // 3. 必须有 urlTemplate + pluginId
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