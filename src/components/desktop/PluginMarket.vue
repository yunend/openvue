<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border-b border-primary-100 px-[26px] py-[22px]">
      <div class="text-[1.05rem] font-bold text-primary-900 mb-4 pb-[10px] border-b border-primary-50">
        🏪 插件市场
      </div>

      <!-- 加载中 -->
      <div v-if="loading" class="text-primary-500 py-8 text-center">⏳ 正在加载插件市场列表...</div>

      <!-- 加载失败 -->
      <div v-else-if="error" class="text-red-500 py-4">
        ❌ 加载失败: {{ error }}
        <button
          class="ml-3 px-3 py-1 text-sm bg-blue-500 text-white border-none rounded cursor-pointer hover:bg-blue-600"
          @click="fetchPlugins"
        >🔄 重试</button>
      </div>

      <!-- 列表 -->
      <div v-else-if="plugins.length === 0" class="text-primary-400 py-8 text-center">暂无可用插件</div>

      <div v-else class="grid gap-4" style="grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));">
        <div
          v-for="p in plugins"
          :key="p.id"
          class="bg-white border border-primary-100 rounded-[10px] p-4 flex flex-col gap-3"
        >
          <!-- 头部：名称 + 版本 -->
          <div class="flex items-start justify-between gap-2">
            <div class="font-bold text-primary-900 text-[0.95rem] leading-snug">{{ p.name }}</div>
            <span class="text-[0.75rem] text-primary-400 bg-primary-50 px-2 py-0.5 rounded whitespace-nowrap shrink-0">v{{ p.version }}</span>
          </div>

          <!-- 描述 -->
          <div class="text-[0.82rem] text-primary-500 leading-relaxed">{{ p.description || '暂无描述' }}</div>

          <!-- 扩展名标签 -->
          <div class="flex flex-wrap gap-1.5">
            <span
              v-for="ext in (p.extensions || [])"
              :key="ext"
              class="text-[0.72rem] bg-blue-50 text-blue-600 px-2 py-0.5 rounded font-medium"
            >.{{ ext }}</span>
          </div>

          <!-- 大小和格式 -->
          <div class="text-[0.75rem] text-primary-400 flex gap-3">
            <span v-if="p.sizeBytes">📦 {{ formatSize(p.sizeBytes) }}</span>
            <span v-if="p.archiveFormat">📄 {{ p.archiveFormat }}</span>
          </div>

          <!-- 下载按钮 + 状态 -->
          <div class="mt-auto pt-2">
            <button
              class="w-full px-3 py-2 text-[0.85rem] font-semibold border-none rounded-[8px] cursor-pointer transition-colors"
              :class="getButtonClass(p.id)"
              :disabled="installingId === p.id"
              @click="handleDownload(p)"
            >
              <template v-if="installingId === p.id">⏳ 安装中...</template>
              <template v-else-if="installedIds.has(p.id)">✅ 已安装</template>
              <template v-else>⬇️ 下载安装</template>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useToast } from '../../composables/useToast'

const { showToast } = useToast()

defineProps<{ isActive: boolean }>()

interface DownloadSource {
  label: string
  url: string
  priority: number
}

interface PluginMeta {
  id: string
  name: string
  version: string
  description: string
  extensions: string[]
  urlTemplate: string
  homepage?: string
  downloadSources: DownloadSource[]
  sha256?: string
  archiveFormat?: string
  sizeBytes?: number
}

const loading = ref(false)
const error = ref('')
const plugins = ref<PluginMeta[]>([])
const installingId = ref('')
const installedIds = ref(new Set<string>())

const INDEX_URL = 'https://gh-proxy.com/https://raw.githubusercontent.com/yunend/openvue-plugins/plugins-data/plugins-index.json'

async function fetchPlugins() {
  loading.value = true
  error.value = ''
  try {
    const { invoke } = window.__TAURI__.core
    const data = await invoke('fetch_plugins_index', { url: INDEX_URL }) as PluginMeta[]
    plugins.value = data
  } catch (e: any) {
    error.value = e.message || String(e)
  } finally {
    loading.value = false
  }
}

async function handleDownload(p: PluginMeta) {
  installingId.value = p.id
  try {
    const { invoke } = window.__TAURI__.core
    await invoke('install_plugin_from_market', { pluginJson: p })
    installedIds.value.add(p.id)
    window.dispatchEvent(new CustomEvent('plugin-installed'))
    showToast(`✅ 插件 [${p.name}] 安装成功！`, 'success')
  } catch (e: any) {
    showToast(`❌ 安装失败: ${e.message || String(e)}`, 'error')
  } finally {
    installingId.value = ''
  }
}

function getButtonClass(id: string) {
  if (installingId.value === id) return 'bg-blue-100 text-blue-400 cursor-wait'
  if (installedIds.value.has(id)) return 'bg-green-50 text-green-600 cursor-default'
  return 'bg-blue-500 text-white hover:bg-blue-600'
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

onMounted(() => {
  fetchPlugins()
})
</script>