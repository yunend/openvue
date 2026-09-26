<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border-b border-primary-100 px-[26px] py-[22px]">
      <div class="text-[1.05rem] font-bold text-primary-900 mb-4 pb-[10px] border-b border-primary-50">
        🏪 插件市场
      </div>

      <!-- 下载源偏好切换 -->
      <div class="mb-4 px-1 flex items-center gap-3">
        <span class="text-[0.82rem] text-primary-500 font-medium">🌐 全局下载源:</span>
        <div class="flex gap-1 bg-white border border-primary-200 rounded-[8px] overflow-hidden">
          <button
            v-for="src in availableLabels"
            :key="src"
            class="px-4 py-1.5 text-[0.82rem] font-medium border-none cursor-pointer transition-colors"
            :class="preferredSource === src
              ? 'bg-blue-500 text-white'
              : 'bg-transparent text-primary-600 hover:bg-primary-50'"
            @click="setPreferredSource(src)"
          >{{ src }}</button>
        </div>
        <span v-if="savingSource" class="text-[0.75rem] text-primary-400">⏳ 保存中...</span>
      </div>

      <!-- 加载中 -->
      <div v-if="loading" class="text-primary-500 py-8 text-center">⏳ 正在加载插件市场列表...</div>

      <!-- 加载失败 -->
      <div v-else-if="error" class="text-red-500 py-4">
        ❌ {{ error }}
        <button class="ml-3 px-3 py-1 text-sm bg-blue-500 text-white border-none rounded cursor-pointer hover:bg-blue-600" @click="fetchAll">🔄 重试</button>
      </div>

      <!-- 列表 -->
      <div v-else-if="plugins.length === 0" class="text-primary-400 py-8 text-center">暂无可用插件</div>

      <div v-else class="grid gap-4" style="grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));">
        <div v-for="p in plugins" :key="p.id" class="bg-white border border-primary-100 rounded-[10px] p-4 flex flex-col gap-3">
          
          <!-- 头部：名称 + 版本 + 状态标签 -->
          <div class="flex items-start justify-between gap-2">
            <div class="font-bold text-primary-900 text-[0.95rem] leading-snug flex-1">{{ p.name }}</div>
            <div class="flex items-center gap-1.5 shrink-0">
              <span class="text-[0.75rem] text-primary-400 bg-primary-50 px-2 py-0.5 rounded">v{{ p.version }}</span>
              <span v-if="installStatus[p.id]?.installed && !needsUpdate(p.id)" class="text-[0.7rem] bg-green-100 text-green-700 px-1.5 py-0.5 rounded font-medium">已安装</span>
              <span v-else-if="needsUpdate(p.id)" class="text-[0.7rem] bg-yellow-100 text-yellow-700 px-1.5 py-0.5 rounded font-medium">可更新</span>
            </div>
          </div>

          <!-- 描述 -->
          <div class="text-[0.82rem] text-primary-500 leading-relaxed">{{ p.description || '暂无描述' }}</div>

          <!-- 扩展名标签 -->
          <div class="flex flex-wrap gap-1.5">
            <span v-for="ext in (p.extensions || [])" :key="ext" class="text-[0.72rem] bg-blue-50 text-blue-600 px-2 py-0.5 rounded font-medium">.{{ ext }}</span>
          </div>

          <!-- 大小 -->
          <div class="text-[0.75rem] text-primary-400">
            <span v-if="p.sizeBytes">📦 {{ formatSize(p.sizeBytes) }} &nbsp;|&nbsp; {{ p.archiveFormat || 'zip' }}</span>
          </div>

          <!-- 下载源选择 -->
          <div v-if="p.downloadSources && p.downloadSources.length > 0" class="flex items-center gap-2">
            <span class="text-[0.75rem] text-primary-500 shrink-0">⬇️ 下载源:</span>
            <select
              v-model="selectedSources[p.id]"
              class="flex-1 text-[0.8rem] border border-primary-200 rounded px-2 py-1 bg-white text-primary-800 outline-none focus:border-blue-400"
            >
              <option v-for="ds in p.downloadSources" :key="ds.label" :value="ds.label">
                {{ ds.label }}{{ ds.priority === 1 ? ' ⭐' : '' }}
              </option>
            </select>
          </div>
          <div v-else class="text-[0.75rem] text-red-400">暂无可用下载源</div>

          <!-- 下载进度条 -->
          <div v-if="downloadingProgress[p.id]" class="flex flex-col gap-1">
            <div class="w-full h-2 bg-primary-100 rounded-full overflow-hidden">
              <div
                class="h-full rounded-full transition-all duration-200 ease-out"
                :style="{ width: Math.min(downloadingProgress[p.id]?.percentage || 0, 100) + '%' }"
                :class="progressBarColor(downloadingProgress[p.id]?.stage || '')"
              ></div>
            </div>
            <div class="flex justify-between text-[0.7rem] text-primary-400">
              <span>{{ progressStageLabel(downloadingProgress[p.id]?.stage || '') }}</span>
              <span v-if="downloadingProgress[p.id]?.percentage != null">{{ Math.round(downloadingProgress[p.id]!.percentage) }}%</span>
            </div>
          </div>

          <!-- 按钮组 -->
          <div class="mt-auto pt-2 flex gap-2">
            <!-- 未安装 → 下载 -->
            <button
              v-if="!installStatus[p.id]?.installed"
              class="flex-1 px-3 py-2 text-[0.85rem] font-semibold border-none rounded-[8px] cursor-pointer transition-colors bg-blue-500 text-white hover:bg-blue-600 disabled:bg-blue-100 disabled:text-blue-400 disabled:cursor-wait"
              :disabled="!!downloadingProgress[p.id]"
              @click="handleDownload(p)"
            >
              {{ downloadingProgress[p.id] ? '下载中...' : '⬇️ 下载安装' }}
            </button>

            <!-- 已安装且需更新 → 更新 -->
            <button
              v-if="needsUpdate(p.id)"
              class="flex-1 px-3 py-2 text-[0.85rem] font-semibold border-none rounded-[8px] cursor-pointer transition-colors bg-yellow-500 text-white hover:bg-yellow-600 disabled:bg-yellow-100 disabled:text-yellow-400 disabled:cursor-wait"
              :disabled="!!downloadingProgress[p.id]"
              @click="handleUpdate(p)"
            >
              {{ downloadingProgress[p.id] ? '更新中...' : '🔄 更新' }}
            </button>

            <!-- 已安装 → 卸载 -->
            <button
              v-if="installStatus[p.id]?.installed"
              class="px-3 py-2 text-[0.85rem] font-semibold border border-red-300 text-red-500 rounded-[8px] cursor-pointer transition-colors hover:bg-red-50"
              :disabled="!!downloadingProgress[p.id]"
              @click="handleUninstall(p)"
            >
              🗑️ 卸载
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, computed } from 'vue'
import { useToast } from '../../composables/useToast'

const { showToast } = useToast()

defineProps<{ isActive: boolean }>()

// ========== 类型定义 ==========

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

interface InstallStatus {
  installed: boolean
  version: string
}

interface ProgressInfo {
  percentage: number
  stage: string
  message: string
}

// ========== 状态 ==========

const loading = ref(false)
const error = ref('')
const plugins = ref<PluginMeta[]>([])
const installStatus = reactive<Record<string, InstallStatus>>({})
const downloadingProgress = reactive<Record<string, ProgressInfo>>({})
const selectedSources = reactive<Record<string, string>>({})

// 下载源偏好
const preferredSource = ref('')
const savingSource = ref(false)
const availableLabels = ['Github', 'Github Proxy']

const INDEX_URL = 'https://gh-proxy.com/https://raw.githubusercontent.com/yunend/openvue-plugins/plugins-data/plugins-index.json'

// ========== 方法 ==========

/// 判断插件是否需要更新
function needsUpdate(id: string): boolean {
  const st = installStatus[id]
  if (!st?.installed) return false
  const p = plugins.value.find(x => x.id === id)
  if (!p) return false
  return st.version !== p.version
}

/// 获取所有插件ID，批量查询安装状态
async function refreshInstallStatus() {
  const ids = plugins.value.map(p => p.id)
  if (ids.length === 0) return
  try {
    const { invoke } = window.__TAURI__.core
    const statuses = await invoke('get_plugins_install_status', { pluginIds: ids }) as Record<string, InstallStatus>
    for (const [id, st] of Object.entries(statuses)) {
      installStatus[id] = st
    }
  } catch (e: any) {
    console.error('refreshInstallStatus 失败:', e)
  }
}

/// 加载用户偏好的下载源标签
async function loadPreferredSource() {
  try {
    const { invoke } = window.__TAURI__.core
    const pref = await invoke('get_preferred_download_source') as string | null
    preferredSource.value = pref || ''
  } catch (e: any) {
    console.error('加载下载源偏好失败:', e)
  }
}

/// 切换下载源偏好并保存
async function setPreferredSource(label: string) {
  if (savingSource.value || label === preferredSource.value) return
  savingSource.value = true
  try {
    const { invoke } = window.__TAURI__.core
    await invoke('set_preferred_download_source', { label })
    preferredSource.value = label
    showToast(`✅ 默认下载源已切换为: ${label}`, 'success')
    // 重新为每个插件选中偏好源（若有）
    applyPreferredSourceToAll()
  } catch (e: any) {
    showToast(`❌ 保存下载源偏好失败: ${e.message || String(e)}`, 'error')
  } finally {
    savingSource.value = false
  }
}

/// 将偏好源应用到所有插件的下载源选择框
function applyPreferredSourceToAll() {
  const pref = preferredSource.value
  if (!pref) return
  for (const p of plugins.value) {
    if (!p.downloadSources || p.downloadSources.length === 0) continue
    // 优先选与用户偏好匹配的源
    const matched = p.downloadSources.find(ds => ds.label === pref)
    if (matched) {
      selectedSources[p.id] = matched.label
    } else {
      // 没有匹配则按 priority 选择
      const sorted = [...p.downloadSources].sort((a, b) => a.priority - b.priority)
      selectedSources[p.id] = sorted[0].label
    }
  }
}

/// 获取插件市场数据 + 安装状态
async function fetchAll() {
  loading.value = true
  error.value = ''
  try {
    const { invoke } = window.__TAURI__.core
    const data = await invoke('fetch_plugins_index', { url: INDEX_URL }) as PluginMeta[]
    plugins.value = data

    // 先加载用户偏好
    await loadPreferredSource()

    // 初始化下载源选择（优先用用户偏好，其次按 priority）
    applyPreferredSourceToAll()

    // 查询安装状态
    await refreshInstallStatus()
  } catch (e: any) {
    error.value = e.message || String(e)
  } finally {
    loading.value = false
  }
}

/// 下载安装插件
async function handleDownload(p: PluginMeta) {
  const payload: PluginMeta = {
    ...p,
    // 只保留用户选择的下载源
    downloadSources: p.downloadSources.filter(ds => ds.label === selectedSources[p.id]),
  }

  downloadingProgress[p.id] = { percentage: 0, stage: 'resolving', message: '准备中...' }

  try {
    const { invoke } = window.__TAURI__.core
    await invoke('download_and_install_plugin', { pluginJson: payload })
    showToast(`✅ 插件 [${p.name}] 安装完成！`, 'success')
    delete downloadingProgress[p.id]
    // 小延迟确保后端文件扫描完成
    await new Promise(r => setTimeout(r, 300))
    await refreshInstallStatus()
    window.dispatchEvent(new CustomEvent('plugin-installed'))
  } catch (e: any) {
    showToast(`❌ 下载安装失败: ${e.message || String(e)}`, 'error')
    delete downloadingProgress[p.id]
  }
}

/// 更新插件
async function handleUpdate(p: PluginMeta) {
  const payload: PluginMeta = {
    ...p,
    downloadSources: p.downloadSources.filter(ds => ds.label === selectedSources[p.id]),
  }

  downloadingProgress[p.id] = { percentage: 0, stage: 'downloading', message: '更新中...' }

  try {
    const { invoke } = window.__TAURI__.core
    await invoke('download_and_install_plugin', { pluginJson: payload })
    showToast(`✅ 插件 [${p.name}] 已更新到 v${p.version}！`, 'success')
    delete downloadingProgress[p.id]
    await new Promise(r => setTimeout(r, 300))
    await refreshInstallStatus()
    window.dispatchEvent(new CustomEvent('plugin-installed'))
  } catch (e: any) {
    showToast(`❌ 更新失败: ${e.message || String(e)}`, 'error')
    delete downloadingProgress[p.id]
  }
}

/// 卸载插件
async function handleUninstall(p: PluginMeta) {
  try {
    const { invoke } = window.__TAURI__.core
    await invoke('uninstall_plugin_command', { pluginId: p.id })
    showToast(`🗑️ 插件 [${p.name}] 已卸载`, 'info')
    delete installStatus[p.id]
    window.dispatchEvent(new CustomEvent('plugin-installed'))
  } catch (e: any) {
    showToast(`❌ 卸载失败: ${e.message || String(e)}`, 'error')
  }
}

// ========== 进度事件监听（Tauri 事件必须用 listen） ==========

function handleProgressEvent(payload: ProgressInfo & { pluginId: string }) {
  if (payload && payload.pluginId) {
    downloadingProgress[payload.pluginId] = {
      percentage: payload.percentage,
      stage: payload.stage,
      message: payload.message,
    }
    if (payload.stage === 'done') {
      setTimeout(() => delete downloadingProgress[payload.pluginId], 1000)
    }
  }
}

// ========== 辅助函数 ==========

function progressBarColor(stage: string): string {
  switch (stage) {
    case 'downloading': return 'bg-blue-400'
    case 'verifying': return 'bg-green-400'
    case 'extracting': return 'bg-purple-400'
    case 'installing': return 'bg-orange-400'
    case 'done': return 'bg-green-500'
    default: return 'bg-blue-300'
  }
}

function progressStageLabel(stage: string): string {
  switch (stage) {
    case 'resolving': return '⏳ 选择下载源...'
    case 'downloading': return '⬇️ 下载中...'
    case 'verifying': return '🔐 校验中...'
    case 'extracting': return '📦 解压中...'
    case 'installing': return '📋 安装中...'
    case 'done': return '✅ 完成'
    default: return stage
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

// ========== 生命周期 ==========

let unlistenProgress: (() => void) | null = null
let unlistenInstalled: (() => void) | null = null

onMounted(async () => {
  fetchAll()

  // 使用 Tauri 的 listen() 监听后端事件
  try {
    const { listen } = await import('@tauri-apps/api/event')
    unlistenProgress = await listen<ProgressInfo & { pluginId: string }>(
      'plugin-download-progress',
      (event) => { handleProgressEvent(event.payload) }
    )
    unlistenInstalled = await listen('plugin-installed', () => refreshInstallStatus())
  } catch (_) {
    // 非 Tauri 环境（如浏览器开发模式）忽略
  }
})

onUnmounted(() => {
  unlistenProgress?.()
  unlistenInstalled?.()
})
</script>