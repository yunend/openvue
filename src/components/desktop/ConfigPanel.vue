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
      

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useConfigManager } from '../../composables/useConfigManager'

defineProps({ isActive: Boolean })

const { t } = useI18n()

const { config, loadConfig, autoSaveConfig, browseFolder } = useConfigManager()
const localConfig = ref({ port: 8005, publicFolder: 'public', enableUpload: false })
const initialized = ref(false)
let pathDebounceTimer: ReturnType<typeof setTimeout> | null = null
let portDebounceTimer: ReturnType<typeof setTimeout> | null = null

onMounted(async () => {
  await loadConfig()
  localConfig.value = { ...config.value }
  await nextTick()
  initialized.value = true
})

watch(() => localConfig.value.port, () => {
  if (!initialized.value) return
  if (portDebounceTimer) clearTimeout(portDebounceTimer)
  portDebounceTimer = setTimeout(() => {
    autoSaveConfig(localConfig.value)
  }, 500)
})

watch(() => localConfig.value.enableUpload, () => {
  if (initialized.value) autoSaveConfig(localConfig.value)
})

watch(() => localConfig.value.publicFolder, () => {
  if (!initialized.value) return
  if (pathDebounceTimer) clearTimeout(pathDebounceTimer)
  pathDebounceTimer = setTimeout(() => {
    autoSaveConfig(localConfig.value)
  }, 500)
})

async function handleBrowseFolder() {
  const chosen = await browseFolder(localConfig.value.publicFolder)
  if (chosen) {
    localConfig.value.publicFolder = chosen
  }
}
</script>