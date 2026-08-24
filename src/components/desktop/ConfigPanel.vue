<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border border-primary-50 rounded-xl px-[26px] py-[22px] mb-[22px]">
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
      
      <div class="flex gap-3 mt-6">
        <button class="flex-1 px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-violet-500 text-white hover:bg-violet-600" @click="loadConfig">
          {{ t('config.load') }}
        </button>
        <button class="flex-1 px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-orange-500 text-white hover:bg-orange-600" @click="handleSaveConfig">
          {{ t('config.save') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useConfigManager } from '../../composables/useConfigManager'

defineProps({ isActive: Boolean })

const { t } = useI18n()

const { config, loadConfig, saveConfig, browseFolder } = useConfigManager()
const localConfig = ref({ port: 8005, publicFolder: 'public', enableUpload: false })


onMounted(async () => {
  await loadConfig()
  localConfig.value = { ...config.value }
})

async function handleBrowseFolder() {
  const chosen = await browseFolder(localConfig.value.publicFolder)
  if (chosen) {
    localConfig.value.publicFolder = chosen
  }
}

async function handleSaveConfig() {
  const success = await saveConfig(localConfig.value)
  if (success) {
    localConfig.value = { ...config.value }
  }
}
</script>