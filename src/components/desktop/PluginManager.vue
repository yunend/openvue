<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border border-primary-50 rounded-xl px-[26px] py-[22px] mb-[22px]">
      <div class="text-[1.05rem] font-bold text-primary-900 mb-4 pb-[10px] border-b border-primary-50">{{ t('plugins.title') }}</div>
      <div class="text-[0.8rem] text-primary-300 mt-[5px] mb-[18px] leading-relaxed">
        <span v-html="t('plugins.desc1')"></span><br>
        <span v-html="t('plugins.desc2')"></span><br>
        <span v-html="t('plugins.desc3')"></span><br>
        <span v-html="t('plugins.desc4')"></span>
      </div>

      <!-- 筛选按钮 -->
      <div class="flex gap-[10px] mb-[18px] flex-wrap">
        <button
          v-for="filter in filters" 
          :key="filter.value"
          class="flex-none px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap"
          :style="{ background: filter.color }"
          @click="filterPlugins(filter.value)"
        >
          {{ filter.label }}
        </button>
      </div>

      <!-- 插件列表 -->
      <div class="flex flex-col gap-3">
        <div
          v-if="filteredPlugins.length === 0"
          class="text-center text-primary-300 py-10"
        >
          {{ pluginsFilter !== 'all' ? t('plugins.empty') : t('plugins.loading') }}
        </div>
        
        <div
          v-for="plugin in filteredPlugins"
          :key="plugin.ext"
          class="flex items-start justify-between px-5 py-4 bg-white border border-primary-50 rounded-[10px]"
        >
          <div class="flex-1">
            <div class="flex items-center gap-3 flex-wrap mb-[6px]">
              <span class="text-[1.5rem]">{{ fileExtIcon(plugin.ext) }}</span>
              <div>
                <div class="text-[1.05rem] font-semibold text-primary-900">
                  .{{ plugin.ext }}
                  <span class="text-primary-300 font-medium text-[0.88rem] ml-2">
                    {{ plugin.name || (plugin.ext.toUpperCase() + t('plugins.fileSuffix')) }}
                  </span>
                </div>
                <div class="text-[0.8rem] text-primary-300 mt-[2px]">{{ plugin.description || '' }}</div>
              </div>
            </div>
            <div class="flex items-center gap-[10px] mt-[10px] flex-wrap">
              <span :style="statusBadgeStyle(plugin.status)">
                {{ statusLabel(plugin.status) }}
              </span>
              <span v-if="plugin.pluginId" class="text-[0.82rem] text-primary-400">
                🔗 {{ t('plugins.pluginId') }}: <b>{{ plugin.pluginId }}</b>
              </span>
              <span v-if="plugin.urlTemplate" class="text-[0.78rem] text-primary-500 break-all">
                URL: <code class="bg-[#f3f4f8] px-[6px] py-[2px] rounded">{{ plugin.urlTemplate }}</code>
              </span>
            </div>
          </div>
          
          <div class="flex-none flex flex-col gap-2 items-end">
            <template v-if="canToggle(plugin.status)">
              <button
                class="flex-none px-4 py-2 text-[0.88rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap"
                :class="mapStatus(plugin.status) === 'enabled' ? 'bg-red-500 text-white hover:bg-red-600' : 'bg-green-500 text-white hover:bg-green-600'"
                @click="handleToggle(plugin.ext, toggleTarget(plugin.status))"
              >
                {{ mapStatus(plugin.status) === 'enabled' ? t('plugins.disable') : t('plugins.enable') }}
              </button>
            </template>
          </div>
        </div>
      </div>

      <div class="flex gap-3 mt-6">
        <button class="flex-1 px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-blue-500 text-white hover:bg-blue-600" @click="loadPluginsConfig">
          {{ t('plugins.reload') }}
        </button>
        <button class="flex-1 px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-slate-500 text-white hover:bg-slate-600" @click="filterPlugins('all')">
          {{ t('plugins.clear') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePluginManager, fileExtIcon, type PluginFilter } from '../../composables/usePluginManager'

defineProps({ isActive: Boolean })

const { t } = useI18n()

const {
  pluginsCache,
  pluginsFilter,
  filteredPlugins,
  loadPluginsConfig,
  togglePlugin,
  filterPlugins
} = usePluginManager()


const filters = computed<{ label: string; value: PluginFilter; color: string }[]>(() => [
  { label: t('plugins.filters.all'), value: 'all', color: '#78909c' },
  { label: t('plugins.filters.enabled'), value: 'enabled', color: '#2e7d32' },
  { label: t('plugins.filters.disabled'), value: 'disabled', color: '#c62828' },
  { label: t('plugins.filters.browserDefault'), value: 'browser-default', color: '#1565c0' },
  { label: t('plugins.filters.undeveloped'), value: 'undeveloped', color: '#757575' }
])

function mapStatus(status: string) {
  const map: Record<string, string> = {
    'BrowserDefault': 'browser-default',
    'Enabled': 'enabled',
    'Disabled': 'disabled',
    'Undeveloped': 'undeveloped'
  }
  return map[status] || status || 'browser-default'
}

function statusLabel(status: string) {
  const key = mapStatus(status)
  const labels: Record<string, string> = {
    'enabled': t('plugins.status.enabled'),
    'disabled': t('plugins.status.disabled'),
    'browser-default': t('plugins.status.browserDefault'),
    'undeveloped': t('plugins.status.undeveloped')
  }
  return labels[key] || status
}

function statusBadgeStyle(status: string) {
  const key = mapStatus(status)
  const styles: Record<string, string> = {
    'enabled': 'display:inline-block; padding:4px 10px; border-radius:30px; background:#e8f5e9; color:#2e7d32; font-size:0.82rem; font-weight:600;',
    'disabled': 'display:inline-block; padding:4px 10px; border-radius:30px; background:#ffebee; color:#c62828; font-size:0.82rem; font-weight:600;',
    'browser-default': 'display:inline-block; padding:4px 10px; border-radius:30px; background:#e3f2fd; color:#1565c0; font-size:0.82rem; font-weight:600;',
    'undeveloped': 'display:inline-block; padding:4px 10px; border-radius:30px; background:#efebe9; color:#6d4c41; font-size:0.82rem; font-weight:600;'
  }
  return styles[key] || styles['browser-default']
}

function canToggle(status: string) {
  const key = mapStatus(status)
  return key === 'enabled' || key === 'disabled'
}

function toggleTarget(status: string) {
  const key = mapStatus(status)
  return key === 'enabled' ? 'disabled' : 'enabled'
}

async function handleToggle(ext: string, newStatus: string) {
  await togglePlugin(ext, newStatus)
}

onMounted(async () => {
  await loadPluginsConfig()
})
</script>