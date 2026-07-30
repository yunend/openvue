<template>
  <div class="panel" :class="{ active: isActive }">
    <div class="panel-section about-card">
      <div class="about-logo">🎛️</div>
      <div class="about-name">Tauri HTTP Server</div>
      <div class="about-version">v{{ version }}</div>
      <div class="about-desc">
        基于 Tauri 2 + Axum 构建的轻量级本地 HTTP 文件服务工具。<br>
        一键启动、开机自启、托盘常驻、目录浏览、文件上传，<br>
        配置灵活，开箱即用。
      </div>
      
      <div class="about-info-grid">
        <div class="about-info-item">
          <div class="about-info-label">当前版本</div>
          <div class="about-info-value">{{ version }}</div>
        </div>
        <div class="about-info-item">
          <div class="about-info-label">构建平台</div>
          <div class="about-info-value">Tauri 2.x + Axum</div>
        </div>
        <div class="about-info-item">
          <div class="about-info-label">GUI 框架</div>
          <div class="about-info-value">WebView2（Windows）</div>
        </div>
        <div class="about-info-item">
          <div class="about-info-label">后端语言</div>
          <div class="about-info-value">Rust (tokio)</div>
        </div>
      </div>
      
      <div class="about-links">
        <a
          class="about-link github"
          href="https://github.com/yunend/openvue/"
          target="_blank"
          @click.prevent="openExternal('https://github.com/yunend/openvue/')"
        >
          <span>🐙</span>
          <span>GitHub 仓库</span>
        </a>
        <a
          class="about-link"
          href="https://tauri.app/"
          target="_blank"
          @click.prevent="openExternal('https://tauri.app/')"
        >
          <span>Tauri 官方文档</span>
        </a>
      </div>
      
      <!-- 版本更新检查 -->
      <div class="update-section">
        <button
          class="update-btn"
          :disabled="isCheckingUpdate"
          :class="{ loading: isCheckingUpdate }"
          @click="checkForUpdates"
        >
          <span>{{ isCheckingUpdate ? '⏳' : '🔄' }}</span>
          <span>{{ isCheckingUpdate ? '检查中...' : '检查更新' }}</span>
        </button>
        <div
          v-if="updateStatus.display"
          class="update-status"
          :class="updateStatus.type"
          v-html="updateStatus.message"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useToast } from '../../composables/useToast'

const props = defineProps({
  isActive: Boolean,
  version: {
    type: String,
    default: '0.1.0'
  }
})

const { showToast } = useToast()
const isCheckingUpdate = ref(false)
const updateStatus = ref({ display: false, type: '', message: '' })

async function openExternal(url) {
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
      message: `❌ 检查更新失败：${e.message}<br>请检查网络连接后重试。`
    }
    console.error('检查更新失败:', e)
  } finally {
    isCheckingUpdate.value = false
  }
}

function compareSemver(a, b) {
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