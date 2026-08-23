<template>
  <div class="panel">
    <div class="panel-section">
      <div class="panel-section-title">🖥️ 操作系统行为</div>
      
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">开机自动启动</div>
          <div class="setting-desc">Windows 用户登录后自动启动本应用</div>
        </div>
        <div class="setting-right">
          <span
            class="setting-status"
            :class="autoStartEnabled ? 'enabled' : 'disabled'"
          >
            {{ autoStartEnabled ? '已启用' : '已禁用' }}
          </span>
          <label class="toggle-switch">
            <input
              type="checkbox"
              :checked="autoStartEnabled"
              @change="handleToggleAutostart($event)"
            >
            <span class="slider"></span>
          </label>
        </div>
      </div>
      
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">最小化到系统托盘</div>
          <div class="setting-desc">隐藏主窗口，HTTP 服务继续在后台运行</div>
        </div>
        <div class="setting-right">
          <button
            class="btn btn-blue"
            style="flex: 0 0 150px;"
            @click="hideToTray"
          >
            📦 隐藏到托盘
          </button>
        </div>
      </div>
      
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">退出应用程序</div>
          <div class="setting-desc">停止 HTTP 服务并完全关闭本程序</div>
        </div>
        <div class="setting-right">
          <button
            class="btn btn-red"
            style="flex: 0 0 150px;"
            @click="quitApp"
          >
            🚪 退出应用
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useSystemSettings } from '../../composables/useSystemSettings'

const {
  autoStartEnabled,
  initAutostartStatus,
  toggleAutostart,
  hideToTray,
  quitApp
} = useSystemSettings()

onMounted(async () => {
  await initAutostartStatus()
})

async function handleToggleAutostart(e: Event) {
  const target = e.target as HTMLInputElement
  const originalChecked = !target.checked
  try {
    await toggleAutostart(target.checked)
  } catch (e) {
    target.checked = originalChecked
  }
}
</script>