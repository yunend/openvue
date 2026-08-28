<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border-b border-primary-100 px-[26px] py-[22px]">
      <div class="text-[1.05rem] font-bold text-primary-900 mb-4 pb-[10px] border-b border-primary-50">{{ t('status.overview') }}</div>
      <div class="grid grid-cols-2 gap-[14px]">
        <div class="bg-white border border-primary-50 rounded-[10px] px-[18px] py-4">
          <div class="text-[0.85rem] text-primary-400 mb-[6px] font-medium">{{ t('status.httpService') }}</div>
          <div 
            class="text-[1.05rem] font-semibold break-all"
            :class="status.isRunning ? 'text-green-700' : 'text-red-700'"
          >
            {{ status.isRunning ? t('status.running') : t('status.stopped') }}
          </div>
        </div>
        <div class="bg-white border border-primary-50 rounded-[10px] px-[18px] py-4">
          <div class="text-[0.85rem] text-primary-400 mb-[6px] font-medium">{{ t('status.listeningPort') }}</div>
          <div class="text-[1.05rem] font-semibold text-primary-900 break-all">{{ status.port || '-' }}</div>
        </div>
        <div class="bg-white border border-primary-50 rounded-[10px] px-[18px] py-4 col-span-2">
          <div class="text-[0.85rem] text-primary-400 mb-[6px] font-medium">{{ t('status.publicFolder') }}</div>
          <div class="text-[1.05rem] font-semibold text-primary-900 break-all">{{ status.publicFolder || '-' }}</div>
        </div>
        <div class="bg-white border border-primary-50 rounded-[10px] px-[18px] py-4 col-span-2">
          <div class="text-[0.85rem] text-primary-400 mb-[6px] font-medium">{{ t('status.accessUrl') }}</div>
          <div class="text-[1.05rem] font-semibold text-primary-900 break-all">
            <template v-if="status.urls && status.urls.length > 0">
              <div
                v-for="(url, idx) in status.urls"
                :key="idx"
                class="flex items-center gap-2 py-1"
              >
                <a
                  class="text-primary-500 no-underline hover:underline"
                  :href="url"
                  target="_blank"
                  @click.prevent="openExternal(url)"
                >
                  {{ url }}
                </a>
                <button
                  class="text-[0.85rem] bg-primary-50 hover:bg-primary-100 border-none rounded px-2 py-0.5 cursor-pointer"
                  :title="t('app.copy') + ' ' + url"
                  @click="copyUrl(url)"
                >📋</button>
              </div>
            </template>
            <span v-else>{{ t('status.serviceStopped') }}</span>
          </div>
        </div>
      </div>
      <div class="flex gap-3 mt-5">
        <button
          class="flex-1 px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-green-500 text-white hover:bg-green-600 disabled:opacity-55 disabled:cursor-not-allowed"
          :disabled="isRunning"
          @click="startServer"
        >
          {{ t('status.start') }}
        </button>
        <button
          class="flex-1 px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-red-500 text-white hover:bg-red-600 disabled:opacity-55 disabled:cursor-not-allowed"
          :disabled="!isRunning"
          @click="stopServer"
        >
          {{ t('status.stop') }}
        </button>
        <button class="flex-1 px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-slate-500 text-white hover:bg-slate-600" @click="refreshStatus">
          {{ t('status.refresh') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useServerControl } from '../../composables/useServerControl'
import { useToast } from '../../composables/useToast'

const { t } = useI18n()

defineProps({ isActive: Boolean })

const {
  isRunning,
  status,
  refreshStatus,
  startServer,
  stopServer
} = useServerControl()

const { showToast } = useToast()

async function openExternal(url: string) {
  try {
    const { invoke } = window.__TAURI__.core
    await invoke('open_url', { url })
  } catch (e) {
    console.error(t('status.openLinkFailed'), e)
  }
}

async function copyUrl(url: string) {
  try {
    await navigator.clipboard.writeText(url)
    showToast(`📋 ${t('app.copied')}: ${url}`, 'success')
  } catch (e) {
    console.error(t('status.copyFailed'), e)
  }
}

onMounted(async () => {
  await refreshStatus()
  
  if (!isRunning.value) {
    showToast(t('status.autoStarting'), 'info')
    setTimeout(async () => {
      try {
        await startServer()
        await refreshStatus()
      } catch (e) {
        console.error('auto start failed:', e)
      }
    }, 300)
  }
})
</script>