<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border-b border-primary-100 text-center py-[28px] px-[26px]">
      <div class="text-[3.8rem] mb-[12px] leading-none">🎛️</div>
      <div class="text-[1.55rem] font-bold text-primary-900 mb-[8px]">Tauri HTTP Server</div>

      <!-- 版本号 + 检查更新按钮（紧挨着放一行） -->
      <div class="flex flex-wrap items-center justify-center gap-3 mb-5">
        <div class="inline-flex items-center px-[16px] py-[6px] bg-primary-50 text-primary-500 rounded-full text-[0.88rem] font-semibold border border-primary-100">
          <span class="mr-1.5">🏷️</span>v{{ version }}
        </div>
        <button
          class="inline-flex items-center gap-2 px-[22px] py-[9px] bg-[#2ea44f] text-white border-none rounded-md text-[0.88rem] font-semibold cursor-pointer transition-all duration-200 hover:bg-[#2c974b] hover:-translate-y-px disabled:bg-[#6e7681] disabled:cursor-not-allowed disabled:translate-y-0 shadow-[0_1px_3px_rgba(46,164,79,0.35)]"
          :class="{ 'animate-pulseBtn': isCheckingUpdate }"
          :disabled="isCheckingUpdate"
          @click="checkForUpdates"
        >
          <span>{{ isCheckingUpdate ? '⏳' : '🔄' }}</span>
          <span>{{ isCheckingUpdate ? t('about.checking') : t('about.checkUpdate') }}</span>
        </button>
      </div>

      <!-- 检查更新结果提示（紧跟在版本号下方） -->
      <div
        v-if="updateStatus.display"
        class="mx-auto max-w-[460px] mb-5 px-4 py-[11px] rounded-md text-[13px] leading-relaxed"
        :class="updateStatusClass"
      >
        <div v-html="updateStatus.message"></div>
        <a
          v-if="updateStatus.url"
          class="inline-flex items-center gap-1 mt-2 text-[#0969da] font-semibold hover:underline"
          :href="updateStatus.url"
          target="_blank"
          @click.prevent="openExternal(updateStatus.url)"
        >
          {{ t('about.download') }}
        </a>
      </div>

      <div class="text-left px-[8px] text-primary-400 text-[0.93rem] leading-relaxed mb-5" v-html="t('about.desc')"></div>
      
      <!-- 2x2 信息网格（宽松版） -->
      <div class="grid grid-cols-2 gap-[12px] text-left mb-5">
        <div class="bg-white/70 border border-primary-100 rounded-[10px] px-4 py-[13px]">
          <div class="text-[0.78rem] text-primary-300 mb-[4px]">{{ t('about.currentVersion') }}</div>
          <div class="text-[0.92rem] font-semibold text-primary-900 break-all">{{ version }}</div>
        </div>
        <div class="bg-white/70 border border-primary-100 rounded-[10px] px-4 py-[13px]">
          <div class="text-[0.78rem] text-primary-300 mb-[4px]">{{ t('about.platform') }}</div>
          <div class="text-[0.92rem] font-semibold text-primary-900 break-all">Tauri 2.x + Axum</div>
        </div>
        <div class="bg-white/70 border border-primary-100 rounded-[10px] px-4 py-[13px]">
          <div class="text-[0.78rem] text-primary-300 mb-[4px]">{{ t('about.gui') }}</div>
          <div class="text-[0.92rem] font-semibold text-primary-900 break-all">WebView2 (Windows)</div>
        </div>
        <div class="bg-white/70 border border-primary-100 rounded-[10px] px-4 py-[13px]">
          <div class="text-[0.78rem] text-primary-300 mb-[4px]">{{ t('about.backend') }}</div>
          <div class="text-[0.92rem] font-semibold text-primary-900 break-all">Rust (tokio)</div>
        </div>
      </div>
      
      <div class="flex flex-wrap gap-[12px] justify-center mt-2">
        <a
          class="inline-flex items-center gap-2 px-[22px] py-[10px] bg-primary-500 text-white rounded-[9px] no-underline font-semibold text-[0.88rem] transition-all duration-200 hover:bg-primary-900 hover:-translate-y-0.5 hover:shadow-[0_3px_10px_rgba(26,35,126,0.3)]"
          href="https://github.com/yunend/openvue/"
          target="_blank"
          @click.prevent="openExternal('https://github.com/yunend/openvue/')"
        >
          <span>🐙</span>
          <span>{{ t('about.github') }}</span>
        </a>
        <a
          class="inline-flex items-center gap-2 px-[22px] py-[10px] bg-primary-500 text-white rounded-[9px] no-underline font-semibold text-[0.88rem] transition-all duration-200 hover:bg-primary-900 hover:-translate-y-0.5 hover:shadow-[0_3px_10px_rgba(26,35,126,0.3)]"
          href="https://tauri.app/"
          target="_blank"
          @click.prevent="openExternal('https://tauri.app/')"
        >
          <span>📚</span>
          <span>{{ t('about.tauriDocs') }}</span>
        </a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps({
  isActive: Boolean,
  version: {
    type: String,
    default: '0.1.0'
  }
})

const isCheckingUpdate = ref(false)
const updateStatus = ref({ display: false, type: '', message: '', url: '' })

const updateStatusClass = computed(() => {
  const map: Record<string, string> = {
    'up-to-date': 'bg-[#e6f4ea] text-[#1a7f37] border border-[#a7d8b9]',
    'available': 'bg-[#fff8c5] text-[#7d4e00] border border-[#f0d48a]',
    'error': 'bg-[#ffebe9] text-[#a0111f] border border-[#f5a8a3]'
  }
  return map[updateStatus.value.type] || ''
})

async function openExternal(url: string) {
  try {
    const { invoke } = window.__TAURI__.core
    await invoke('open_url', { url })
  } catch (e) {
    console.error('打开链接失败:', e)
  }
}

async function checkForUpdates() {
  isCheckingUpdate.value = true
  updateStatus.value = { display: false, type: '', message: '', url: '' }
  
  try {
    const res = await fetch('https://api.github.com/repos/yunend/openvue/releases/latest', {
      headers: { 'Accept': 'application/vnd.github.v3+json' }
    })
    
    if (!res.ok) throw new Error(t('about.apiError', { status: res.status }))
    
    const data = await res.json()
    const latestVersion = data.tag_name || data.name
    
    if (!latestVersion) throw new Error(t('about.noVersion'))
    
    const cleanLatest = latestVersion.replace(/^v/, '')
    const cleanCurrent = props.version.replace(/^v/, '')
    const comparison = compareSemver(cleanCurrent, cleanLatest)
    
    updateStatus.value = {
      display: true,
      type: comparison >= 0 ? 'up-to-date' : 'available',
      message: comparison >= 0 
        ? t('about.latest', { version: props.version })
        : t('about.available', { latest: latestVersion, current: props.version }),
      url: comparison >= 0 ? '' : data.html_url
    }
  } catch (e) {
    updateStatus.value = {
      display: true,
      type: 'error',
      message: t('about.updateError', { msg: (e as Error).message }),
      url: ''
    }
    console.error('check update failed:', e)
  } finally {
    isCheckingUpdate.value = false
  }
}

function compareSemver(a: string, b: string) {
  const pa = a.split('.').map(Number)
  const pb = b.split('.').map(Number)
  const len = Math.max(pa.length, pb.length)
  for (let i = 0; i < len; i++) {
    const na = pa[i] || 0
    const nb = pb[i] || 0
    if (na > nb) return 1
    if (na < nb) return -1
  }
  return 0
}
</script>