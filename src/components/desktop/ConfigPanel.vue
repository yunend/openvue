<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border-b border-primary-100 px-[26px] py-[22px]">
      <div class="text-[1.05rem] font-bold text-primary-900 mb-4 pb-[10px] border-b border-primary-50">{{ t('config.basicTitle') }}</div>
      
      <div class="mb-[18px]">
        <label class="block text-[0.9rem] text-primary-900 font-semibold mb-2">{{ t('config.portLabel') }}</label>
        <input
          type="number"
          class="w-full px-[15px] py-[11px] border-2 border-primary-100 rounded-[9px] text-base text-primary-900 bg-white transition-colors duration-200 focus:outline-none focus:border-primary-500"
          v-model.number="localConfig.port"
          min="1"
          max="65535"
          :placeholder="t('config.portPlaceholder')"
        >
        <div class="text-[0.8rem] text-primary-300 mt-[5px]">{{ t('config.portHint') }}</div>
      </div>
      
      <div class="mb-[18px]">
        <label class="block text-[0.9rem] text-primary-900 font-semibold mb-2">{{ t('config.folderLabel') }}</label>
        <div class="flex gap-[10px] items-stretch">
          <input
            type="text"
            class="flex-1 px-[15px] py-[11px] border-2 border-primary-100 rounded-[9px] text-base text-primary-900 bg-white transition-colors duration-200 focus:outline-none focus:border-primary-500"
            v-model="localConfig.publicFolder"
            :placeholder="t('config.folderPlaceholder')"
          >
          <button
            type="button"
            class="flex-none px-5 py-0 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-slate-500 text-white hover:bg-slate-600"
            @click="handleBrowseFolder"
          >
            {{ t('config.browse') }}
          </button>
        </div>
        <div class="text-[0.8rem] text-primary-300 mt-[5px]">{{ t('config.folderHint') }}</div>
      </div>
      
      <div class="flex items-center justify-between px-[18px] py-[14px] bg-white border border-primary-50 rounded-[10px] mb-[14px]">
        <div>
          <div class="text-[0.95rem] font-semibold text-primary-900">{{ t('config.enableUpload') }}</div>
          <div class="text-[0.8rem] text-primary-300 mt-[2px]">{{ t('config.enableUploadHint') }}</div>
        </div>
        <label class="toggle-switch">
          <input type="checkbox" v-model="localConfig.enableUpload">
          <span class="slider"></span>
        </label>
      </div>

      <!-- 插件文件夹配置 -->
      <div class="mb-[18px] px-[18px] py-[14px] bg-white border border-primary-50 rounded-[10px]">
        <label class="block text-[0.9rem] text-primary-900 font-semibold mb-2">{{ t('config.pluginsFolderLabel') }}</label>

        <!-- 当前生效目录（展示 + 打开超链接） -->
        <div class="flex items-center gap-2 mb-2 text-[0.82rem] flex-wrap">
          <span class="text-primary-500">{{ t('config.pluginsFolderActive') }}:</span>
          <a
            class="text-blue-500 hover:text-blue-700 underline break-all cursor-pointer"
            @click="handleOpenPluginsFolder"
          >
            {{ localConfig.effectivePluginsDir || '(加载中...)' }}
          </a>
        </div>

        <!-- 自定义路径输入行 -->
        <div class="flex gap-[8px] items-stretch flex-wrap">
          <input
            type="text"
            class="flex-1 min-w-[180px] px-[12px] py-[9px] border-2 border-primary-100 rounded-[8px] text-sm text-primary-900 bg-white focus:outline-none focus:border-blue-400"
            v-model="pluginsFolderInput"
            :placeholder="t('config.pluginsFolderPlaceholder')"
          >
          <button
            type="button"
            class="flex-none px-[14px] py-2 text-[0.88rem] font-semibold border-none rounded-[8px] cursor-pointer bg-slate-500 text-white hover:bg-slate-600 whitespace-nowrap"
            @click="handleBrowsePluginsFolder"
          >
            {{ t('config.browse') }}
          </button>
          <button
            type="button"
            class="flex-none px-[14px] py-2 text-[0.88rem] font-semibold border-none rounded-[8px] cursor-pointer bg-blue-500 text-white hover:bg-blue-600 whitespace-nowrap"
            @click="handleApplyPluginsFolder"
          >
            {{ t('config.apply') }}
          </button>
          <button
            v-if="localConfig.pluginsFolder"
            type="button"
            class="flex-none px-[14px] py-2 text-[0.88rem] font-semibold border-none rounded-[8px] cursor-pointer bg-gray-400 text-white hover:bg-gray-500 whitespace-nowrap"
            @click="handleResetPluginsFolder"
          >
            {{ t('config.reset') }}
          </button>
        </div>
        <div class="text-[0.8rem] text-primary-300 mt-[5px]">
          {{ t('config.pluginsFolderHint') }}
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useConfigManager, type AppConfig } from '../../composables/useConfigManager'
import { useServerControl } from '../../composables/useServerControl'

defineProps({ isActive: Boolean })

const { t } = useI18n()

const { config, loadConfig, saveConfig, browseFolder, setPluginsFolder, resetPluginsFolder, openPluginsFolder } = useConfigManager()
const { refreshStatus } = useServerControl()
const localConfig = ref<AppConfig>({
  port: 8005,
  publicFolder: 'public',
  enableUpload: false,
  pluginsFolder: null,
  effectivePluginsDir: ''
})
const pluginsFolderInput = ref('')
const initialized = ref(false)
let pathDebounceTimer: ReturnType<typeof setTimeout> | null = null
let portDebounceTimer: ReturnType<typeof setTimeout> | null = null

onMounted(async () => {
  await loadConfig()
  localConfig.value = { ...config.value }
  pluginsFolderInput.value = config.value.pluginsFolder || ''
  await nextTick()
  initialized.value = true
})

watch(() => localConfig.value.port, async () => {
  if (!initialized.value) return
  if (portDebounceTimer) clearTimeout(portDebounceTimer)
  portDebounceTimer = setTimeout(async () => {
    const ok = await saveConfig(localConfig.value)
    if (ok) setTimeout(refreshStatus, 300)
  }, 500)
})

watch(() => localConfig.value.enableUpload, async () => {
  if (initialized.value) {
    const ok = await saveConfig(localConfig.value)
    if (ok) setTimeout(refreshStatus, 300)
  }
})

watch(() => localConfig.value.publicFolder, async () => {
  if (!initialized.value) return
  if (pathDebounceTimer) clearTimeout(pathDebounceTimer)
  pathDebounceTimer = setTimeout(async () => {
    const ok = await saveConfig(localConfig.value)
    if (ok) setTimeout(refreshStatus, 300)
  }, 500)
})

async function handleBrowseFolder() {
  const chosen = await browseFolder(
    localConfig.value.publicFolder,
    '选择指定文件根目录'
  )
  if (chosen) {
    localConfig.value.publicFolder = chosen
  }
}

async function handleBrowsePluginsFolder() {
  const initial = localConfig.value.pluginsFolder || localConfig.value.effectivePluginsDir
  const chosen = await browseFolder(
    initial || undefined,
    '选择插件根目录'
  )
  if (chosen) {
    pluginsFolderInput.value = chosen
  }
}

async function handleApplyPluginsFolder() {
  const path = pluginsFolderInput.value.trim()
  if (!path) {
    await resetPluginsFolder()
    pluginsFolderInput.value = ''
    localConfig.value = { ...config.value }
    setTimeout(refreshStatus, 300)
    return
  }
  await setPluginsFolder(path)
  localConfig.value = { ...config.value }
  setTimeout(refreshStatus, 300)
}

async function handleResetPluginsFolder() {
  await resetPluginsFolder()
  pluginsFolderInput.value = ''
  localConfig.value = { ...config.value }
  setTimeout(refreshStatus, 300)
}

async function handleOpenPluginsFolder() {
  await openPluginsFolder()
}
</script>