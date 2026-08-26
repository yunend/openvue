<template>
  <div id="file-browser" class="w-11/12 bg-card-bg min-h-80 mx-auto mt-6 flex flex-col items-center gap-1 overflow-y-auto p-4 border border-border-color rounded-lg">
    <div class="w-full text-sm text-blue-600 dark:text-blue-400 font-medium mb-1">
      📂 /{{ currentPath.length > 0 ? currentPath.join('/') : t('home.rootDir') }}
    </div>
    
    <div class="w-full mb-2">
      <input 
        v-model="searchKeyword"
        type="text" 
        :placeholder="t('home.search')"
        class="w-full px-4 py-2 border border-border-color bg-page-bg text-text-primary rounded outline-none focus:border-blue-600 focus:ring-2 focus:ring-blue-600 transition"
      />
    </div>
    
    <div class="w-full flex justify-between items-center mb-2">
      <button 
        @click="goBack"
        :disabled="currentPath.length === 0"
        :class="currentPath.length === 0 ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'"
        class="px-4 py-2 text-white rounded transition-colors"
      >
        ← {{ t('home.back') }}
      </button>
      
      <div class="flex gap-2">
        <button 
          @click="sortBy = 'name'" 
          :class="sortBy === 'name' ? 'bg-green-600' : 'bg-gray-600'"
          class="px-3 py-2 text-white rounded hover:opacity-80 transition-opacity text-sm"
        >
          {{ sortBy === 'name' ? '✓ ' : '' }}{{ t('home.sortByName') }}
        </button>
        <button 
          @click="sortBy = 'time'" 
          :class="sortBy === 'time' ? 'bg-green-600' : 'bg-gray-600'"
          class="px-3 py-2 text-white rounded hover:opacity-80 transition-opacity text-sm"
        >
          {{ sortBy === 'time' ? '✓ ' : '' }}{{ t('home.sortByTime') }}
        </button>
      </div>
    </div>
    
    <div v-if="loading" class="text-text-secondary">{{ t('home.loading') }}</div>
    <div v-else-if="error" class="text-red-500">{{ error }}</div>
    <div v-else-if="items.length === 0" class="text-text-secondary">{{ t('home.empty') }}</div>
    
    <div v-else class="w-full">
      <div 
        v-for="item in items" 
        :key="item.path" 
        class="flex items-center justify-between p-2 hover:bg-slate-200 dark:hover:bg-slate-700 rounded transition-colors"
      >
        <div 
          class="flex items-center gap-2 cursor-pointer flex-1 text-text-primary" 
          @click="handleClick(item)"
        >
          <span class="text-xl">{{ item.type === 'directory' ? '📂' : '📄' }}</span>
          <span class="font-medium">{{ item.name }}</span>
        </div>
        
        <div class="flex items-center gap-3">
          <span class="text-sm text-text-secondary">{{ formatDate(item.mtime) }}</span>
          <button 
            v-if="item.type === 'file'" 
            @click.stop="downloadFile(item)"
            class="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 transition-colors flex items-center gap-1"
            :title="t('home.download')"
          >
            <span>⬇️</span>
            <span>{{ t('home.download') }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFileBrowser } from '../../composables/useFileBrowser'
import { usePluginResolver } from '../../composables/usePluginResolver'

const { t, locale } = useI18n()

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

function formatDate(dateString: string): string {
  if (!dateString) return ''
  const date = new Date(dateString)
  const localeStr = locale.value === 'en' ? 'en-US' : 'zh-CN'
  return date.toLocaleString(localeStr, { 
    year: 'numeric', 
    month: '2-digit', 
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function getFileExtension(filename: string): string {
  if (!filename) return ''
  const lastDotIndex = filename.lastIndexOf('.')
  if (lastDotIndex === -1) return ''
  return filename.slice(lastDotIndex + 1).toLowerCase()
}

function downloadFile(item: FileItem): void {
  const url = '/public' + item.path
  const a = document.createElement('a')
  a.href = url
  a.download = item.name
  a.style.display = 'none'
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
}

function previewFile(item: FileItem): void {
  const url = '/public' + item.path
  // 所有文件直接打开，后端会处理文本文件的编码和 MIME 类型
  window.open(url, '_blank')
}

function handleClick(item: FileItem): void {
  if (item.type === 'directory') {
    const newPath = item.path.split('/').filter(Boolean)
    loadDir(newPath)
    return
  }

  if (item.type !== 'file') return

  const ext = getFileExtension(item.name)
  const publicPath ='/public' + item.path
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