<template>
  <div id="file-browser" class="w-12/12 bg-slate-50 h-300 mx-auto mt-1 flex flex-col items-center gap-1 overflow-y-auto p-4">
    <div class="w-full text-sm text-blue-600 font-medium mb-1">
      📂 /{{ currentPath.length > 0 ? currentPath.join('/') : '根目录' }}
    </div>
    
    <div class="w-full mb-2">
      <input 
        v-model="searchKeyword"
        type="text" 
        placeholder="搜索文件名或文件夹名..."
        class="w-full px-4 py-2 border border-gray-300 rounded outline-none focus:border-blue-600 focus:ring-2 focus:ring-blue-600 transition"
      />
    </div>
    
    <div class="w-full flex justify-between items-center mb-2">
      <button 
        @click="goBack"
        :disabled="currentPath.length === 0"
        :class="currentPath.length === 0 ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'"
        class="px-4 py-2 text-white rounded transition-colors"
      >
        ← 返回上一级
      </button>
      
      <div class="flex gap-2">
        <button 
          @click="sortBy = 'name'" 
          :class="sortBy === 'name' ? 'bg-green-600' : 'bg-gray-600'"
          class="px-3 py-2 text-white rounded hover:opacity-80 transition-opacity text-sm"
        >
          {{ sortBy === 'name' ? '✓ ' : '' }}按名称
        </button>
        <button 
          @click="sortBy = 'time'" 
          :class="sortBy === 'time' ? 'bg-green-600' : 'bg-gray-600'"
          class="px-3 py-2 text-white rounded hover:opacity-80 transition-opacity text-sm"
        >
          {{ sortBy === 'time' ? '✓ ' : '' }}按时间
        </button>
      </div>
    </div>
    
    <div v-if="loading" class="text-gray-500">加载中...</div>
    <div v-else-if="error" class="text-red-500">{{ error }}</div>
    <div v-else-if="items.length === 0" class="text-gray-500">目录为空</div>
    
    <div v-else class="w-full">
      <div 
        v-for="item in items" 
        :key="item.path" 
        class="flex items-center justify-between p-2 hover:bg-slate-300 rounded"
      >
        <div 
          class="flex items-center gap-2 cursor-pointer flex-1" 
          @click="handleClick(item)"
        >
          <span class="text-xl">{{ item.type === 'directory' ? '📂' : '📄' }}</span>
          <span>{{ item.name }}</span>
        </div>
        
        <div class="flex items-center gap-3">
          <span class="text-sm text-gray-500">{{ formatDate(item.mtime) }}</span>
          <button 
            v-if="item.type === 'file'" 
            @click.stop="downloadFile(item)"
            class="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 transition-colors flex items-center gap-1"
            title="下载文件"
          >
            <span>⬇️</span>
            <span>下载</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import { useFileBrowser } from '../../composables/useFileBrowser'
import { usePluginResolver } from '../../composables/usePluginResolver'

const {
  items,
  loading,
  error,
  currentPath,
  searchKeyword,
  sortBy,
  allItems,
  loadDir,
  filterItems,
  sortAllItems,
  goBack
} = useFileBrowser()

const { resolvePluginUrl, loadPluginsMap } = usePluginResolver()

function formatDate(dateString) {
  if (!dateString) return ''
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', { 
    year: 'numeric', 
    month: '2-digit', 
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function getFileExtension(filename) {
  if (!filename) return ''
  const lastDotIndex = filename.lastIndexOf('.')
  if (lastDotIndex === -1) return ''
  return filename.slice(lastDotIndex + 1).toLowerCase()
}

function downloadFile(item) {
  const url = '/public' + item.path
  const a = document.createElement('a')
  a.href = url
  a.download = item.name
  a.style.display = 'none'
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
}

// 需要用 Blob URL 预览的文本类扩展名（fetch 解码确保 UTF-8 正确）
const TEXT_EXTENSIONS = ['txt', 'md', 'json', 'xml', 'html', 'htm', 'css', 'js', 'ts', 'yaml', 'yml', 'csv', 'log', 'ini', 'conf', 'toml', 'rs', 'py', 'java', 'c', 'cpp', 'h', 'vue', 'jsx', 'tsx', 'sql', 'sh', 'bat', 'ps1', 'yml', 'properties', 'env', 'gitignore', 'editorconfig', 'js', 'ts']

// 文本类型到 MIME 的映射
function getMimeType(ext) {
  const map = {
    txt: 'text/plain',
    md: 'text/markdown',
    json: 'application/json',
    xml: 'application/xml',
    html: 'text/html',
    htm: 'text/html',
    css: 'text/css',
    js: 'application/javascript',
    ts: 'application/javascript',
    vue: 'text/plain',
    yaml: 'text/yaml',
    yml: 'text/yaml',
    csv: 'text/csv',
    log: 'text/plain',
    ini: 'text/plain',
    conf: 'text/plain',
    toml: 'text/plain',
    rs: 'text/plain',
    py: 'text/plain',
    java: 'text/plain',
    c: 'text/plain',
    cpp: 'text/plain',
    h: 'text/plain',
    sql: 'text/plain',
    sh: 'text/plain',
    bat: 'text/plain',
    properties: 'text/plain',
    env: 'text/plain',
  }
  return map[ext] || 'text/plain'
}

async function previewFile(item) {
  const url = '/public' + item.path
  const ext = getFileExtension(item.name)

  // 文本类文件：fetch 后用 Blob URL 打开，确保 UTF-8 编码正确
  if (TEXT_EXTENSIONS.includes(ext)) {
    try {
      const res = await fetch(url)
      const text = await res.text()
      const mime = getMimeType(ext)
      const blob = new Blob([text], { type: mime + ';charset=utf-8' })
      const blobUrl = URL.createObjectURL(blob)
      window.open(blobUrl, '_blank')
      // 延迟释放 Blob URL
      setTimeout(() => URL.revokeObjectURL(blobUrl), 60000)
    } catch (e) {
      // fetch 失败则回退到直接打开
      window.open(url, '_blank')
    }
  } else {
    // 图片、视频、PDF 等：直接打开，浏览器自带预览
    window.open(url, '_blank')
  }
}

function handleClick(item) {
  if (item.type === 'directory') {
    const newPath = item.path.split('/').filter(Boolean)
    loadDir(newPath)
    return
  }

  if (item.type !== 'file') return

  const ext = getFileExtension(item.name)
  const publicPath = '/public' + item.path
  const pluginUrl = resolvePluginUrl(ext, publicPath)

  if (pluginUrl) {
    console.log(`🧩 使用插件打开 .${ext}：`, pluginUrl)
    window.open(pluginUrl, `${ext}_viewer`)
  } else {
    console.log(`👁️ 预览文件 .${ext}：`, publicPath)
    previewFile(item)
  }
}

watch(sortBy, () => {
  sortAllItems()
})

watch(searchKeyword, () => {
  filterItems()
})

onMounted(async () => {
  await Promise.all([loadDir([]), loadPluginsMap()])
})
</script>