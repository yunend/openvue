<template>
  <div class="app-layout">
    <!-- 左侧导航 -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <h1>🎛️ Tauri 控制台</h1>
        <p>HTTP 文件服务管理器</p>
      </div>
      <ul class="nav-list">
        <li v-for="item in navItems" :key="item.id" class="nav-item">
          <button
            class="nav-btn"
            :class="{ active: activePanel === item.id }"
            @click="switchPanel(item.id)"
          >
            <span class="nav-icon">{{ item.icon }}</span>
            <span>{{ item.label }}</span>
          </button>
        </li>
      </ul>
      <div class="sidebar-footer">v{{ version }}</div>
    </aside>

    <!-- 右侧内容区 -->
    <main class="content">
      <div class="content-header">
        <div class="content-title">
          <span class="icon">{{ currentPanelInfo.icon }}</span>
          <span>{{ currentPanelInfo.label }}</span>
        </div>
        <button class="btn-outline" @click="restartHttpService">🔄 重启HTTP服务</button>
      </div>

      <div class="content-body">
        <!-- 面板 1：服务状态 -->
        <StatusPanel :class="{ active: activePanel === 'status' }" />

        <!-- 面板 2：配置管理 -->
        <ConfigPanel :class="{ active: activePanel === 'config' }" />

        <!-- 面板 3：系统设置 -->
        <SystemSettings :class="{ active: activePanel === 'system' }" />

        <!-- 面板 4：插件配置 -->
        <PluginManager :class="{ active: activePanel === 'plugins' }" />

        <!-- 面板 5：关于 -->
        <AboutPanel :class="{ active: activePanel === 'about' }" :version="version" />
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