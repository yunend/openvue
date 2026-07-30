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
  window.open('/public' + item.path, '_blank')
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
    console.log(`🌐 浏览器默认打开 .${ext}：`, publicPath)
    downloadFile(item)
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