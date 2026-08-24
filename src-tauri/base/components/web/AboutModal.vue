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
          <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 space-y-3">
            <div>
              <div class="text-sm text-gray-500 mb-1">当前软件版本</div>
              <div class="flex items-baseline gap-3">
                <span class="text-3xl font-bold text-blue-700 font-mono">v{{ aboutData.version }}</span>
              </div>
              <div class="text-xs text-gray-400 mt-2">技术栈：{{ aboutData.buildStack }}</div>
            </div>

            <div class="border-t border-blue-200 pt-3">
              <div class="text-sm text-gray-500 mb-2">⚙️ 当前配置</div>
              <div class="space-y-2">
                <div class="flex items-center gap-2">
                  <span class="text-sm text-gray-600">文件上传：</span>
                  <span :class="aboutData.config?.enableUpload ? 'text-green-600 font-medium' : 'text-red-600'">
                    {{ aboutData.config?.enableUpload ? '已启用' : '已禁用' }}
                  </span>
                </div>
                <div class="flex items-center gap-2">
                  <span class="text-sm text-gray-600">服务端口：</span>
                  <span class="font-mono font-medium">{{ aboutData.config?.port }}</span>
                </div>
                <div class="flex items-start gap-2">
                  <span class="text-sm text-gray-600 whitespace-nowrap">公共目录：</span>
                  <span class="font-mono text-xs break-all">{{ aboutData.config?.publicFolder }}</span>
                </div>
              </div>
            </div>
          </div>

          <div>
            <h3 class="font-bold text-gray-700 mb-2">🧩 当前已启用的扩展名</h3>
            <div v-if="enabledPlugins.length > 0" class="flex flex-wrap gap-2">
              <span 
                v-for="plugin in enabledPlugins" 
                :key="plugin.ext"
                class="px-2 py-1 rounded text-xs font-mono bg-green-100 text-green-800 border border-green-300"
              >
                .{{ plugin.ext }}
              </span>
            </div>
            <div v-else class="text-sm text-gray-400">暂无已启用的扩展名</div>
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

const enabledPlugins = computed(() => {
  if (!aboutPlugins.value) return []
  return Object.entries(aboutPlugins.value)
    .filter(([_, entry]) => entry.status === 'enabled')
    .map(([ext, entry]) => ({ ext, ...entry }))
})

onMounted(() => {
  loadAboutInfo()
})
</script>