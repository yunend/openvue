<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border border-primary-50 rounded-xl px-[26px] py-[22px] mb-[22px]">
      <div class="text-[1.05rem] font-bold text-primary-900 mb-4 pb-[10px] border-b border-primary-50">🖥️ 操作系统行为</div>
      
      <div class="flex items-center justify-between px-5 py-4 bg-white border border-primary-50 rounded-[10px] mb-[14px]">
        <div class="flex-1">
          <div class="text-base font-semibold text-primary-900 mb-[3px]">开机自动启动</div>
          <div class="text-[0.82rem] text-primary-300">Windows 用户登录后自动启动本应用</div>
        </div>
        <div class="flex items-center gap-[14px]">
          <span
            class="text-[0.85rem] font-semibold"
            :class="autoStartEnabled ? 'text-green-700' : 'text-red-700'"
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
      
      <div class="flex items-center justify-between px-5 py-4 bg-white border border-primary-50 rounded-[10px] mb-[14px]">
        <div class="flex-1">
          <div class="text-base font-semibold text-primary-900 mb-[3px]">最小化到系统托盘</div>
          <div class="text-[0.82rem] text-primary-300">隐藏主窗口，HTTP 服务继续在后台运行</div>
        </div>
        <div class="flex items-center gap-[14px]">
          <button
            class="flex-none w-[150px] px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-blue-500 text-white hover:bg-blue-600"
            @click="hideToTray"
          >
            📦 隐藏到托盘
          </button>
        </div>
      </div>
      
      <div class="flex items-center justify-between px-5 py-4 bg-white border border-primary-50 rounded-[10px] mb-[14px]">
        <div class="flex-1">
          <div class="text-base font-semibold text-primary-900 mb-[3px]">退出应用程序</div>
          <div class="text-[0.82rem] text-primary-300">停止 HTTP 服务并完全关闭本程序</div>
        </div>
        <div class="flex items-center gap-[14px]">
          <button
            class="flex-none w-[150px] px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-red-500 text-white hover:bg-red-600"
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

defineProps({ isActive: Boolean })

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