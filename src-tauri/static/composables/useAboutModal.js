import { ref, computed } from 'vue'

export function useAboutModal() {
  const aboutLoading = ref(false)
  const aboutData = ref(null)
  const aboutError = ref(null)
  const copiedConfig = ref(false)
  const aboutPlugins = ref(null)

  async function loadAboutInfo() {
    aboutLoading.value = true
    aboutError.value = null
    try {
      const [aboutRes, pluginsRes] = await Promise.all([
        fetch('/api/about'),
        fetch('/api/plugins')
      ])
      
      if (!aboutRes.ok) throw new Error('HTTP ' + aboutRes.status)
      aboutData.value = await aboutRes.json()

      if (pluginsRes.ok) {
        const pluginsData = await pluginsRes.json()
        aboutPlugins.value = pluginsData.extensions || {}
      } else {
        aboutPlugins.value = null
      }
    } catch (e) {
      aboutError.value = e.message || String(e)
    } finally {
      aboutLoading.value = false
    }
  }

  async function copyConfigToClipboard(prettyConfigJson) {
    try {
      await navigator.clipboard.writeText(prettyConfigJson)
      copiedConfig.value = true
      setTimeout(() => (copiedConfig.value = false), 1500)
    } catch (e) {
      alert('复制失败，请手动选择文本复制')
    }
  }

  function statusLabel(status) {
    const map = {
      'enabled': '已启用',
      'disabled': '未启用',
      'browser-default': '浏览器默认',
      'undeveloped': '未开发'
    }
    return map[status] || status
  }

  function statusBadgeClass(status) {
    const map = {
      'enabled': 'bg-green-100 text-green-800 border border-green-300',
      'disabled': 'bg-red-100 text-red-800 border border-red-300',
      'browser-default': 'bg-blue-100 text-blue-800 border border-blue-300',
      'undeveloped': 'bg-gray-200 text-gray-600 border border-gray-300'
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
    statusBadgeClass
  }
}