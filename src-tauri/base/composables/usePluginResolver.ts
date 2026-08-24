import { ref } from 'vue'

export function usePluginResolver() {
  const pluginsMap = ref<Record<string, PluginEntry>>({})

  async function loadPluginsMap(): Promise<void> {
    try {
      const response = await fetch('/api/plugins')
      if (response.ok) {
        const data: PluginsData = await response.json()
        pluginsMap.value = data.extensions || {}
        console.log('🧩 插件映射表已加载，共', Object.keys(pluginsMap.value).length, '个扩展名')
      } else {
        console.warn('获取 /api/plugins 失败:', response.status)
      }
    } catch (e) {
      console.warn('加载插件配置失败（不影响使用，降级为浏览器默认打开）:', e)
    }
  }

  /**
   * 根据扩展名 + publicPath 算出插件打开 URL
   * @param ext 扩展名（不带点，会自动转小写）
   * @param publicPath 文件在服务端的公开路径
   * @returns 应打开的插件完整 URL；null = 走浏览器默认
   */
  function resolvePluginUrl(ext: string, publicPath: string): string | null {
    if (!ext) return null

    const key = ext.toLowerCase()
    const entry = pluginsMap.value[key]
    if (!entry) return null

    // 只有 status === "enabled"（已启用）才用插件
    if (entry.status !== 'enabled') return null

    // 已启用但没有 urlTemplate 或 pluginId，同样无效 → 回默认
    const tpl = entry.urlTemplate
    const pid = entry.pluginId
    if (!tpl || !pid) return null

    // 占位符替换：{pluginId} 和 {publicPath}
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