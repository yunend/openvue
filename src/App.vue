<template>
  <div class="flex h-screen p-4 gap-4">
    <!-- 左侧导航 -->
    <aside class="w-[220px] bg-white rounded-[14px] shadow-[0_2px_14px_rgba(26,35,126,0.10)] flex flex-col overflow-hidden shrink-0">
      <div class="px-5 py-[22px] bg-gradient-to-br from-primary-500 to-primary-400 text-white">
        <h1 class="text-[1.2rem] font-bold mb-[6px]">🎛️ Tauri 控制台</h1>
        <p class="text-[0.8rem] opacity-90">HTTP 文件服务管理器</p>
      </div>
      <ul class="list-none px-[10px] py-[14px] flex-1 overflow-y-auto">
        <li v-for="item in navItems" :key="item.id" class="mb-1">
          <button
            class="w-full flex items-center gap-3 px-[14px] py-3 border-none bg-transparent rounded-[10px] text-[0.95rem] font-medium text-primary-700 cursor-pointer transition-all duration-200 text-left hover:bg-primary-50"
            :class="{ 'bg-gradient-to-r from-primary-500 to-primary-400 text-white shadow-[0_2px_8px_rgba(63,81,181,0.35)]': activePanel === item.id }"
            @click="switchPanel(item.id)"
          >
            <span class="text-[1.1rem] w-[22px] text-center">{{ item.icon }}</span>
            <span>{{ item.label }}</span>
          </button>
        </li>
      </ul>
      <div class="px-5 py-[14px] text-[0.78rem] text-primary-300 border-t border-primary-50">v{{ version }}</div>
    </aside>

    <!-- 右侧内容区 -->
    <main class="flex-1 bg-white rounded-[14px] shadow-[0_2px_14px_rgba(26,35,126,0.10)] overflow-hidden flex flex-col">
      <div class="px-[30px] py-[22px] border-b border-primary-50 flex items-center justify-between">
        <div class="text-[1.4rem] font-bold text-primary-900 flex items-center gap-3">
          <span class="text-[1.5rem]">{{ currentPanelInfo.icon }}</span>
          <span>{{ currentPanelInfo.label }}</span>
        </div>
        <button class="flex-none px-5 py-[10px] bg-white text-primary-500 border-2 border-primary-500 font-semibold rounded-[9px] transition-all duration-200 hover:bg-primary-500 hover:text-white" @click="restartHttpService">🔄 重启HTTP服务</button>
      </div>

      <div class="flex-1 px-[34px] py-7 overflow-y-auto">
        <!-- 面板 1：服务状态 -->
        <StatusPanel :isActive="activePanel === 'status'" />

        <!-- 面板 2：配置管理 -->
        <ConfigPanel :isActive="activePanel === 'config'" />

        <!-- 面板 3：系统设置 -->
        <SystemSettings :isActive="activePanel === 'system'" />

        <!-- 面板 4：插件配置 -->
        <PluginManager :isActive="activePanel === 'plugins'" />

        <!-- 面板 5：关于 -->
        <AboutPanel :isActive="activePanel === 'about'" :version="version" />
      </div>
    </main>

    <!-- Toast 提示 -->
    <ToastContainer />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import StatusPanel from './components/desktop/StatusPanel.vue'
import ConfigPanel from './components/desktop/ConfigPanel.vue'
import SystemSettings from './components/desktop/SystemSettings.vue'
import PluginManager from './components/desktop/PluginManager.vue'
import AboutPanel from './components/desktop/AboutPanel.vue'
import ToastContainer from './components/desktop/ToastContainer.vue'
import { useServerControl } from './composables/useServerControl'

const { restartHttpService } = useServerControl()

const activePanel = ref('status')
const version = ref('0.1.0')

const navItems = [
  { id: 'status', icon: '📊', label: '当前服务状态' },
  { id: 'config', icon: '⚙️', label: '配置文件管理' },
  { id: 'system', icon: '🖥️', label: '系统设置' },
  { id: 'plugins', icon: '🧩', label: '插件配置' },
  { id: 'about', icon: 'ℹ️', label: '关于' }
]

const currentPanelInfo = computed(() => {
  return navItems.find(item => item.id === activePanel.value) || navItems[0]
})

function switchPanel(panelId:string) {
  activePanel.value = panelId
}

onMounted(async () => {
  try {
    const { invoke } = window.__TAURI__.core
    version.value = await invoke('get_version')
  } catch (e) {
    console.warn('获取版本号失败:', e)
  }
})
</script>