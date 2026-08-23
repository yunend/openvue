<template>
  <div class="panel" :class="{ active: isActive }">
    <div class="panel-section">
      <div class="panel-section-title">🟢 运行概览</div>
      <div class="status-grid">
        <div class="status-item">
          <div class="status-label">HTTP 服务</div>
          <div 
            class="status-value"
            :class="status.isRunning ? 'running' : 'stopped'"
          >
            {{ status.isRunning ? '✅ 运行中' : '⏸️ 未启动' }}
          </div>
        </div>
        <div class="status-item">
          <div class="status-label">监听端口</div>
          <div class="status-value">{{ status.port || '-' }}</div>
        </div>
        <div class="status-item full">
          <div class="status-label">公共文件根目录</div>
          <div class="status-value">{{ status.staticFolder || '-' }}</div>
        </div>
        <div class="status-item full">
          <div class="status-label">访问地址</div>
          <div class="status-value">
            <template v-if="status.urls && status.urls.length > 0">
              <div
                v-for="(url, idx) in status.urls"
                :key="idx"
                class="url-row"
              >
                <a
                  class="url-link"
                  :href="url"
                  target="_blank"
                  @click.prevent="openExternal(url)"
                >
                  {{ url }}
                </a>
                <button
                  class="btn-copy"
                  :title="'复制 ' + url"
                  @click="copyUrl(url)"
                >📋</button>
              </div>
            </template>
            <span v-else>（服务未启动）</span>
          </div>
        </div>
      </div>
      <div class="control-row">
        <button
          class="btn btn-green"
          :disabled="isRunning"
          @click="startServer"
        >
          ▶ 启动服务
        </button>
        <button
          class="btn btn-red"
          :disabled="!isRunning"
          @click="stopServer"
        >
          ■ 停止服务
        </button>
        <button class="btn btn-grey" @click="refreshStatus">
          🔄 刷新
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useServerControl } from '../../composables/useServerControl'
import { useToast } from '../../composables/useToast'

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
    console.error('打开链接失败:', e)
  }
}

async function copyUrl(url: string) {
  try {
    await navigator.clipboard.writeText(url)
    showToast('📋 已复制: ' + url, 'success')
  } catch (e) {
    console.error('复制失败:', e)
  }
}

defineProps({
  isActive: Boolean
})

onMounted(async () => {
  await refreshStatus()
  
  if (!isRunning.value) {
    showToast('🔌 正在自动启动 HTTP 服务...', 'info')
    setTimeout(async () => {
      try {
        await startServer()
        await refreshStatus()
      } catch (e) {
        console.error('自动启动失败:', e)
      }
    }, 300)
  }
})

</script>

<style scoped>
.status-panel {
  animation: fadeIn 0.22s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: translateY(0); }
}

.panel-section {
  background: #f8f9ff;
  border: 1px solid #e8eaf6;
  border-radius: 12px;
  padding: 22px 26px;
  margin-bottom: 22px;
}

.panel-section-title {
  font-size: 1.05rem;
  font-weight: 700;
  color: #1a237e;
  margin-bottom: 16px;
  padding-bottom: 10px;
  border-bottom: 1px solid #e8eaf6;
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 14px;
}

.status-item {
  background: white;
  border: 1px solid #e8eaf6;
  border-radius: 10px;
  padding: 16px 18px;
}

.status-item.full {
  grid-column: span 2;
}

.status-label {
  font-size: 0.85rem;
  color: #5c6bc0;
  margin-bottom: 6px;
  font-weight: 500;
}

.status-value {
  font-size: 1.05rem;
  color: #1a237e;
  font-weight: 600;
  word-break: break-all;
}

.status-value.running {
  color: #2e7d32;
}

.status-value.stopped {
  color: #c62828;
}

.control-row {
  display: flex;
  gap: 12px;
  margin-top: 20px;
}

.btn-start,
.btn-stop,
.btn-secondary {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 0.95rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-start {
  background: linear-gradient(135deg, #2e7d32, #43a047);
  color: white;
}

.btn-start:hover {
  box-shadow: 0 4px 12px rgba(46, 125, 50, 0.35);
}

.btn-stop {
  background: linear-gradient(135deg, #c62828, #e53935);
  color: white;
}

.btn-stop:hover {
  box-shadow: 0 4px 12px rgba(198, 40, 40, 0.35);
}

.btn-secondary {
  background: #e8eaf6;
  color: #3949ab;
}

.btn-secondary:hover {
  background: #c5cae9;
}

.url-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.url-row:last-child {
  margin-bottom: 0;
}

.url-link {
  color: #3f51b5;
  text-decoration: none;
  font-weight: 600;
}

.url-link:hover {
  text-decoration: underline;
}

.btn-copy {
  background: none;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  padding: 2px 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background 0.15s;
}

.btn-copy:hover {
  background: #f0f0f0;
}
</style>