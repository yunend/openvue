<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border border-primary-50 rounded-xl max-w-[560px] mx-5 text-center py-[30px] px-6">
      <div class="text-[4.2rem] mb-[14px]">🎛️</div>
      <div class="text-[1.7rem] font-bold text-primary-900 mb-[6px]">Tauri HTTP Server</div>
      <div class="inline-block px-[14px] py-[5px] bg-primary-50 text-primary-500 rounded-full text-[0.85rem] font-semibold mb-6">v{{ version }}</div>
      <div class="text-primary-400 text-[0.95rem] leading-loose mb-6">
        基于 Tauri 2 + Axum 构建的轻量级本地 HTTP 文件服务工具。<br>
        一键启动、开机自启、托盘常驻、目录浏览、文件上传，<br>
        配置灵活，开箱即用。
      </div>
      
      <div class="grid grid-cols-2 gap-[14px] text-left mb-7">
        <div class="bg-primary-50 border border-primary-50 rounded-[10px] px-4 py-[14px]">
          <div class="text-[0.8rem] text-primary-300 mb-[4px]">当前版本</div>
          <div class="text-[0.95rem] font-semibold text-primary-900 break-all">{{ version }}</div>
        </div>
        <div class="bg-primary-50 border border-primary-50 rounded-[10px] px-4 py-[14px]">
          <div class="text-[0.8rem] text-primary-300 mb-[4px]">构建平台</div>
          <div class="text-[0.95rem] font-semibold text-primary-900 break-all">Tauri 2.x + Axum</div>
        </div>
        <div class="bg-primary-50 border border-primary-50 rounded-[10px] px-4 py-[14px]">
          <div class="text-[0.8rem] text-primary-300 mb-[4px]">GUI 框架</div>
          <div class="text-[0.95rem] font-semibold text-primary-900 break-all">WebView2（Windows）</div>
        </div>
        <div class="bg-primary-50 border border-primary-50 rounded-[10px] px-4 py-[14px]">
          <div class="text-[0.8rem] text-primary-300 mb-[4px]">后端语言</div>
          <div class="text-[0.95rem] font-semibold text-primary-900 break-all">Rust (tokio)</div>
        </div>
      </div>
      
      <div class="flex gap-[14px] justify-center">
        <a
          class="inline-flex items-center gap-2 px-[22px] py-[11px] bg-primary-500 text-white rounded-[10px] no-underline font-semibold text-[0.9rem] transition-all duration-200 hover:bg-primary-900 hover:-translate-y-0.5 hover:shadow-[0_4px_12px_rgba(26,35,126,0.3)]"
          href="https://github.com/yunend/openvue/"
          target="_blank"
          @click.prevent="openExternal('https://github.com/yunend/openvue/')"
        >
          <span>🐙</span>
          <span>GitHub 仓库</span>
        </a>
        <a
          class="inline-flex items-center gap-2 px-[22px] py-[11px] bg-primary-500 text-white rounded-[10px] no-underline font-semibold text-[0.9rem] transition-all duration-200 hover:bg-primary-900 hover:-translate-y-0.5 hover:shadow-[0_4px_12px_rgba(26,35,126,0.3)]"
          href="https://tauri.app/"
          target="_blank"
          @click.prevent="openExternal('https://tauri.app/')"
        >
          <span>Tauri 官方文档</span>
        </a>
      </div>
      
      <!-- 版本更新检查 -->
      <div class="mt-5 text-center">
        <button
          class="inline-flex items-center gap-2 px-6 py-[10px] bg-[#2ea44f] text-white border-none rounded-md text-sm font-semibold cursor-pointer transition-all duration-200 hover:bg-[#2c974b] hover:-translate-y-px disabled:bg-[#6e7681] disabled:cursor-not-allowed disabled:translate-y-0"
          :class="{ 'animate-pulseBtn': isCheckingUpdate }"
          :disabled="isCheckingUpdate"
          @click="checkForUpdates"
        >
          <span>{{ isCheckingUpdate ? '⏳' : '🔄' }}</span>
          <span>{{ isCheckingUpdate ? '检查中...' : '检查更新' }}</span>
        </button>
        <div
          v-if="updateStatus.display"
          class="mt-3 px-4 py-[10px] rounded-md text-[13px] leading-relaxed"
          :class="updateStatusClass"
          v-html="updateStatus.message"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps({
  isActive: Boolean,
  version: {
    type: String,
    default: '0.1.0'
  }
})

const isCheckingUpdate = ref(false)
const updateStatus = ref({ display: false, type: '', message: '' })

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
  updateStatus.value = { display: false, type: '', message: '' }
  
  try {
    const res = await fetch('https://api.github.com/repos/yunend/openvue/releases/latest', {
      headers: { 'Accept': 'application/vnd.github.v3+json' }
    })
    
    if (!res.ok) throw new Error(`GitHub API 返回 ${res.status}`)
    
    const data = await res.json()
    const latestVersion = data.tag_name || data.name
    
    if (!latestVersion) throw new Error('未找到版本信息')
    
    const cleanLatest = latestVersion.replace(/^v/, '')
    const cleanCurrent = props.version.replace(/^v/, '')
    const comparison = compareSemver(cleanCurrent, cleanLatest)
    
    updateStatus.value = {
      display: true,
      type: comparison >= 0 ? 'up-to-date' : 'available',
      message: comparison >= 0 
        ? `✅ 已是最新版本（${props.version}）`
        : `🎉 发现新版本 <strong>${latestVersion}</strong>（当前 ${props.version}）<br>` +
          `<a href="${data.html_url}" target="_blank" onclick="event.preventDefault(); window.open('${data.html_url}')">前往下载 →</a>`
    }
  } catch (e) {
    updateStatus.value = {
      display: true,
      type: 'error',
      message: `❌ 检查更新失败：${(e as Error).message}<br>请检查网络连接后重试。`
    }
    console.error('检查更新失败:', e)
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