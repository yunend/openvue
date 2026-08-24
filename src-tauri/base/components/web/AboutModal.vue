<template>
  <div 
    class="fixed inset-0 bg-black/55 z-50 flex items-center justify-center p-4"
    @click.self="$emit('close')"
  >
    <div class="bg-white rounded-xl shadow-2xl w-full max-w-xl max-h-[85vh] flex flex-col overflow-hidden">
      <div class="bg-blue-700 text-white px-6 py-4 flex items-center justify-between">
        <h2 class="text-xl font-bold tracking-wide">ℹ️ 关于本软件</h2>
        <button 
          @click="$emit('close')"
          class="text-3xl leading-none hover:text-blue-200 transition select-none px-2 -mr-2"
        >×</button>
      </div>

      <div class="px-6 py-5 overflow-y-auto space-y-5">
        <div v-if="aboutLoading" class="text-center py-10 text-gray-500">
          <div class="text-4xl mb-3 animate-bounce">⏳</div>
          正在加载配置信息...
        </div>

        <template v-else-if="aboutData">
          <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <div class="text-sm text-gray-500 mb-1">当前软件版本</div>
            <div class="flex items-baseline gap-3">
              <span class="text-3xl font-bold text-blue-700 font-mono">v{{ aboutData.version }}</span>
            </div>
            <div class="text-xs text-gray-400 mt-2">技术栈：{{ aboutData.buildStack }}</div>
          </div>

          <div>
            <div class="flex items-center justify-between mb-2">
              <h3 class="font-bold text-gray-700">⚙️ 当前配置（config.json）</h3>
              <button 
                @click="copyConfigToClipboard(prettyConfigJson)"
                class="text-xs px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded transition text-gray-600"
              >
                {{ copiedConfig ? '✅ 已复制' : '📋 复制配置' }}
              </button>
            </div>
            <pre class="bg-slate-900 text-green-400 text-sm p-4 rounded-lg overflow-x-auto leading-relaxed font-mono">{{ prettyConfigJson }}</pre>
          </div>

          <div>
            <h3 class="font-bold text-gray-700 mb-2">🧩 当前可用扩展名</h3>
            <div v-if="aboutPlugins && Object.keys(aboutPlugins).length > 0" class="flex flex-wrap gap-2">
              <span 
                v-for="(entry, ext) in aboutPlugins" 
                :key="ext"
                :class="statusBadgeClass(entry.status)"
                class="px-2 py-1 rounded text-xs font-mono"
              >
                .{{ ext }}
                <span class="ml-1 opacity-70">({{ statusLabel(entry.status) }})</span>
              </span>
            </div>
            <div v-else class="text-sm text-gray-400">暂无扩展名配置</div>
          </div>

          <div>
            <h3 class="font-bold text-gray-700 mb-3">🔗 帮助与文档</h3>
            <ul class="grid grid-cols-1 sm:grid-cols-2 gap-2">
              <li v-for="link in aboutData.helpLinks" :key="link.url">
                <a 
                  :href="link.url" 
                  target="_blank" 
                  rel="noopener noreferrer"
                  class="block px-3 py-2 border border-gray-200 rounded hover:bg-blue-50 hover:border-blue-300 hover:text-blue-700 transition text-sm"
                >
                  {{ link.label }}
                </a>
              </li>
            </ul>
          </div>
        </template>

        <div v-else class="bg-red-50 border border-red-200 rounded-lg p-4 text-center text-red-600">
          <div class="text-3xl mb-2">❌</div>
          <div>加载关于信息失败：{{ aboutError || '未知错误' }}</div>
        </div>
      </div>

      <div class="bg-gray-50 px-6 py-3 border-t flex justify-end">
        <button 
          @click="$emit('close')"
          class="px-5 py-2 bg-blue-700 text-white rounded-lg hover:bg-blue-800 transition shadow-sm"
        >
          关闭
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useAboutModal } from '../../composables/useAboutModal'

const emit = defineEmits<{
  (e: 'close'): void
}>()

const {
  aboutLoading,
  aboutData,
  aboutError,
  aboutPlugins,
  loadAboutInfo,
  copiedConfig,
  copyConfigToClipboard,
  statusLabel,
  statusBadgeClass
} = useAboutModal()

const prettyConfigJson = computed(() => {
  if (!aboutData.value) return ''
  return JSON.stringify(aboutData.value.config, null, 2)
})

onMounted(() => {
  loadAboutInfo()
})
</script>