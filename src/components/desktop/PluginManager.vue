<template>
  <div class="panel" :class="{ active: isActive }">
    <div class="panel-section">
      <div class="panel-section-title">🧩 扩展名与插件映射表</div>
      <div class="form-hint" style="margin-bottom:18px;">
        • <b>已启用</b>：点击对应插件页打开（需 plugins 目录下已放置资源）<br>
        • <b>未启用</b>：虽安装了插件，但暂不使用，按浏览器默认行为打开<br>
        • <b>浏览器默认支持</b>：浏览器原生能渲染的格式（图片/视频/HTML 等）<br>
        • <b>未开发</b>：未来计划支持，当前走浏览器默认（会直接下载）
      </div>

      <!-- 筛选按钮 -->
      <div style="display:flex; gap:10px; margin-bottom:18px; flex-wrap:wrap;">
        <button
          v-for="filter in filters"
          :key="filter.value"
          class="btn btn-grey"
          style="flex:0 0 auto;"
          :style="{ background: filter.color }"
          @click="filterPlugins(filter.value)"
        >
          {{ filter.label }}
        </button>
      </div>

      <!-- 插件列表 -->
      <div style="display:flex; flex-direction:column; gap:12px;">
        <div
          v-if="filteredPlugins.length === 0"
          style="text-align:center; color:#7986cb; padding:40px 0;"
        >
          {{ pluginsFilter !== 'all' ? '该分类下暂无条目' : '⏳ 正在加载插件配置...' }}
        </div>
        
        <div
          v-for="plugin in filteredPlugins"
          :key="plugin.ext"
          class="switch-row"
          style="padding:16px 20px; align-items:flex-start;"
        >
          <div style="flex:1;">
            <div style="display:flex; align-items:center; gap:12px; flex-wrap:wrap; margin-bottom:6px;">
              <span style="font-size:1.5rem;">{{ fileExtIcon(plugin.ext) }}</span>
              <div>
                <div class="switch-name" style="font-size:1.05rem;">
                  .{{ plugin.ext }}
                  <span style="color:#7986cb; font-weight:500; font-size:0.88rem; margin-left:8px;">
                    {{ plugin.name || plugin.ext.toUpperCase() + ' 文件' }}
                  </span>
                </div>
                <div class="switch-desc" style="margin-top:4px;">{{ plugin.description || '' }}</div>
              </div>
            </div>
            <div style="display:flex; align-items:center; gap:10px; margin-top:10px; flex-wrap:wrap;">
              <span :style="statusBadgeStyle(plugin.status)">
                {{ statusLabel(plugin.status) }}
              </span>
              <span v-if="plugin.pluginId" style="font-size:0.82rem; color:#5c6bc0;">
                🔗 插件ID: <b>{{ plugin.pluginId }}</b>
              </span>
              <span v-if="plugin.urlTemplate" style="font-size:0.78rem; color:#78909c; word-break:break-all;">
                URL: <code style="background:#f3f4f8; padding:2px 6px; border-radius:4px;">{{ plugin.urlTemplate }}</code>
              </span>
            </div>
          </div>
          
          <div style="flex-shrink:0; display:flex; flex-direction:column; gap:8px; align-items:flex-end;">
            <template v-if="canToggle(plugin.status)">
              <button
                class="btn"
                :class="mapStatus(plugin.status) === 'enabled' ? 'btn-red' : 'btn-green'"
                style="flex:0 0 auto; padding:8px 16px; font-size:0.88rem;"
                @click="handleToggle(plugin.ext, toggleTarget(plugin.status))"
              >
                {{ mapStatus(plugin.status) === 'enabled' ? '⏸️ 禁用' : '▶️ 启用' }}
              </button>
            </template>
          </div>
        </div>
      </div>

      <div class="control-row" style="margin-top:24px;">
        <button class="btn btn-blue" @click="loadPluginsConfig">
          🔄 重新加载配置
        </button>
        <button class="btn btn-grey" @click="filterPlugins('all')">
          🗑️ 清除筛选
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, computed } from 'vue'
import { usePluginManager, fileExtIcon } from '../../composables/usePluginManager'

const {
  pluginsCache,
  pluginsFilter,
  filteredPlugins,
  loadPluginsConfig,
  togglePlugin,
  filterPlugins
} = usePluginManager()

defineProps({
  isActive: Boolean
})

const filters = [
  { label: '全部', value: 'all', color: '#78909c' },
  { label: '✅ 已启用', value: 'enabled', color: '#2e7d32' },
  { label: '⏸️ 未启用', value: 'disabled', color: '#c62828' },
  { label: '🌐 浏览器默认', value: 'browser-default', color: '#1565c0' },
  { label: '🚧 未开发', value: 'undeveloped', color: '#757575' }
]

function mapStatus(status) {
  const map = {
    'BrowserDefault': 'browser-default',
    'Enabled': 'enabled',
    'Disabled': 'disabled',
    'Undeveloped': 'undeveloped'
  }
  return map[status] || status || 'browser-default'
}

function statusLabel(status) {
  const key = mapStatus(status)
  const labels = {
    'enabled': '✅ 已启用',
    'disabled': '⏸️ 未启用',
    'browser-default': '🌐 浏览器默认支持',
    'undeveloped': '🚧 未开发'
  }
  return labels[key] || status
}

function statusBadgeStyle(status) {
  const key = mapStatus(status)
  const styles = {
    'enabled': 'display:inline-block; padding:4px 10px; border-radius:30px; background:#e8f5e9; color:#2e7d32; font-size:0.82rem; font-weight:600;',
    'disabled': 'display:inline-block; padding:4px 10px; border-radius:30px; background:#ffebee; color:#c62828; font-size:0.82rem; font-weight:600;',
    'browser-default': 'display:inline-block; padding:4px 10px; border-radius:30px; background:#e3f2fd; color:#1565c0; font-size:0.82rem; font-weight:600;',
    'undeveloped': 'display:inline-block; padding:4px 10px; border-radius:30px; background:#efebe9; color:#6d4c41; font-size:0.82rem; font-weight:600;'
  }
  return styles[key] || styles['browser-default']
}

function canToggle(status) {
  const key = mapStatus(status)
  return key === 'enabled' || key === 'disabled'
}

function toggleTarget(status) {
  const key = mapStatus(status)
  return key === 'enabled' ? 'disabled' : 'enabled'
}

async function handleToggle(ext, newStatus) {
  await togglePlugin(ext, newStatus)
}

onMounted(async () => {
  await loadPluginsConfig()
})
</script>