<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border-b border-primary-100 px-[26px] py-[22px]">
      <div class="text-[1.05rem] font-bold text-primary-900 mb-4 pb-[10px] border-b border-primary-50">{{ t('plugins.title') }}</div>
      <div class="text-[0.8rem] text-primary-300 mt-[5px] mb-[18px] leading-relaxed">
        <span v-html="t('plugins.desc1')"></span><br>
        <span v-html="t('plugins.desc2')"></span><br>
        <span v-html="t('plugins.desc3')"></span><br>
        <span v-html="t('plugins.desc4')"></span><br>
        <span v-html="t('plugins.desc5')"></span>
      </div>

      <!-- 🔧 自定义插件 -->
      <div class="mb-[18px] px-5 py-4 bg-white border border-dashed border-blue-200 rounded-[10px]">
        <div class="text-[0.95rem] font-semibold text-primary-900 mb-3">🔧 自定义插件</div>
        <div class="flex flex-wrap items-end gap-3">
          <div class="flex-1 min-w-[120px]">
            <label class="block text-[0.82rem] text-primary-500 font-medium mb-1">{{ t('plugins.customExtLabel') }}</label>
            <input
              type="text"
              class="w-full px-[12px] py-[9px] border-2 border-primary-100 rounded-[8px] text-sm text-primary-900 bg-white focus:outline-none focus:border-blue-400"
              v-model="customExt"
              :placeholder="t('plugins.customExtPlaceholder')"
            >
          </div>
          <div class="flex-[2] min-w-[200px]">
            <label class="block text-[0.82rem] text-primary-500 font-medium mb-1">{{ t('plugins.customFolderLabel') }}</label>
            <div class="flex gap-[8px] items-stretch">
              <input
                type="text"
                class="flex-1 px-[12px] py-[9px] border-2 border-primary-100 rounded-[8px] text-sm text-primary-900 bg-white focus:outline-none focus:border-blue-400"
                v-model="customFolderPath"
                :placeholder="t('plugins.customFolderPlaceholder')"
                readonly
              >
              <button
                type="button"
                class="flex-none px-[14px] py-0 text-[0.88rem] font-semibold border-none rounded-[8px] cursor-pointer bg-slate-500 text-white hover:bg-slate-600 whitespace-nowrap"
                @click="handleBrowseCustomFolder"
              >
                📁 {{ t('plugins.customBrowse') }}
              </button>
            </div>
          </div>
          <button
            type="button"
            class="flex-none px-[18px] py-[9px] text-[0.9rem] font-semibold border-none rounded-[8px] cursor-pointer bg-blue-500 text-white hover:bg-blue-600 whitespace-nowrap"
            @click="handleAddCustomPlugin"
          >
            ➕ {{ t('plugins.customAdd') }}
          </button>
        </div>
      </div>

      <!-- 筛选按钮 -->
      <div class="flex gap-[10px] mb-[18px] flex-wrap">
        <button
          v-for="filter in filters" 
          :key="filter.value"
          class="flex-none px-[18px] py-3 text-[0.95rem] font-semibold rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap border"
          :class="pluginsFilter === filter.value ? 'ring-2 ring-offset-1' : ''"
          :style="{
            background: filter.color.bg,
            color: filter.color.text,
            borderColor: filter.color.border,
            '--tw-ring-color': filter.color.text,
          } as Record<string, string>"
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
        
        <!-- 每个扩展名是一个「组卡」，内部展开多个处理器 -->
        <div
          v-for="plugin in filteredPlugins"
          :key="plugin.ext"
          class="px-5 py-4 bg-white border border-primary-50 rounded-[10px]"
        >
          <!-- ── 组头：扩展名总览（整体不再分左右两栏，统一单列防挤压换行） ── -->
          <div class="w-full min-w-0">
            <!-- 第 1 行：扩展名标题 + 数量，不换行，截断用省略号 -->
            <div class="flex items-center gap-3 mb-2 w-full min-w-0">
              <span class="text-[1.5rem] flex-none">{{ fileExtIcon(plugin.ext) }}</span>
              <div class="text-[1.05rem] font-semibold text-primary-900 whitespace-nowrap overflow-hidden text-ellipsis">
                .{{ plugin.ext }}
                <span class="text-primary-300 font-medium text-[0.88rem] ml-2">
                  {{ plugin.ext.toUpperCase() + t('plugins.fileSuffix') }}
                  <span class="text-[0.78rem] ml-2">（共 {{ plugin.handlers.length }} 个处理器）</span>
                </span>
              </div>
            </div>

            <!-- 第 2 行：当前激活处理器名称，长名称允许换行但不把容器撑破 -->
            <div class="text-[0.8rem] text-primary-300 leading-relaxed break-words mb-2">
              <span>{{ t('plugins.currentActive') }}：</span>
              <b class="text-primary-700">
                {{ (plugin.handlers.find(h => h.handlerId === plugin.activeHandlerId) || plugin.handlers[0])?.name || '（无，走浏览器默认）' }}
              </b>
            </div>

            <!-- 第 3 行：状态 badge + 下拉切换菜单；整体占满宽，下拉框限制最大宽度防出界 -->
            <div class="flex items-center gap-3 flex-wrap w-full min-w-0">
              <span
                class="flex-none"
                :style="statusBadgeStyle(plugin.activeStatus)"
              >
                {{ statusLabel(plugin.activeStatus) }}
              </span>
              <!-- 🎯 切换激活处理器：下拉菜单（统一入口）——浏览器默认 / 未开发状态隐藏，因为没可切换的候选 -->
              <div
                v-if="['enabled', 'disabled'].includes(mapStatus(plugin.activeStatus))"
                class="flex items-center gap-2 flex-none min-w-0 max-w-full"
              >
                <span class="text-[0.85rem] text-primary-400 whitespace-nowrap flex-none">
                  {{ t('plugins.activePickerLabel') }}：
                </span>
                <select
                  class="px-3 py-1.5 text-[0.85rem] bg-white border border-primary-100 rounded-[7px] text-primary-800 cursor-pointer focus:outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition max-w-[58%] sm:max-w-[420px] min-w-[180px] whitespace-nowrap overflow-hidden text-ellipsis"
                  :value="plugin.activeHandlerId ?? '__none__'"
                  :title="(plugin.handlers.find(h => h.handlerId === plugin.activeHandlerId) || plugin.handlers[0])?.name || ''"
                  @change="(e) => handleActivate(plugin.ext, (e.target as HTMLSelectElement).value)"
                >
                  <option value="__none__" disabled>{{ t('plugins.chooseActivePlaceholder') }}</option>
                  <option
                    v-for="h in plugin.handlers"
                    :key="h.handlerId"
                    :value="h.handlerId"
                    :title="`${h.name} · [${h.handlerId}]`"
                  >
                    {{ h.name }} · [{{ h.handlerId }}] · {{ statusLabel(h.status) }}
                  </option>
                </select>
              </div>
            </div>
          </div>

          <!-- ── 组内：该扩展名的所有处理器列表 ── -->
          <div class="mt-4 pl-4 pr-1 py-3 bg-primary-50/60 rounded-[9px] border border-dashed border-primary-100">
            <div
              v-for="handler in plugin.handlers"
              :key="handler.handlerId"
              class="flex items-start justify-between gap-4 flex-wrap py-3 last:pb-0 first:pt-0 border-b last:border-b-0 border-primary-100"
            >
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 flex-wrap">
                  <span
                    v-if="plugin.activeHandlerId === handler.handlerId"
                    :title="t('plugins.activeMarker')"
                    class="inline-block text-[0.82rem] font-bold px-2 py-0.5 rounded-full bg-green-100 text-green-800 border border-green-200"
                  >
                    ★ {{ t('plugins.activeMarker') }}
                  </span>
                  <span class="text-[0.95rem] font-semibold text-primary-900">
                    {{ handler.name }}
                  </span>
                  <span class="text-[0.78rem] text-primary-400">
                    handlerId: <code class="bg-[#f3f4f8] px-1.5 py-0.5 rounded">{{ handler.handlerId }}</code>
                  </span>
                </div>
                <div class="text-[0.8rem] text-primary-300 mt-[3px]">{{ handler.description || '' }}</div>
                <div class="flex items-center gap-[10px] mt-[8px] flex-wrap">
                  <span :style="statusBadgeStyle(handler.status)">
                    {{ statusLabel(handler.status) }}
                  </span>
                  <span v-if="handler.pluginId" class="text-[0.82rem] text-primary-400">
                    🔗 {{ t('plugins.pluginId') }}: <b>{{ handler.pluginId }}</b>
                  </span>
                  <span v-if="handler.urlTemplate" class="text-[0.78rem] text-primary-500 break-all">
                    URL: <code class="bg-[#f3f4f8] px-[6px] py-[2px] rounded">{{ resolveUrl(handler.urlTemplate, handler.pluginId) }}</code>
                  </span>
                </div>
              </div>

              <div class="flex-none flex flex-col gap-2 items-end">
                <!-- 启用 / 禁用按钮 —— 激活切换已统一走头部下拉菜单 -->
                <template v-if="canToggle(handler.status)">
                  <button
                    class="flex-none px-4 py-2 text-[0.88rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap"
                    :class="mapStatus(handler.status) === 'enabled' ? 'bg-red-500 text-white hover:bg-red-600' : 'bg-green-500 text-white hover:bg-green-600'"
                    @click="handleHandlerToggle(plugin.ext, handler.handlerId, toggleTarget(handler.status))"
                  >
                    {{ mapStatus(handler.status) === 'enabled' ? t('plugins.disable') : t('plugins.enable') }}
                  </button>
                </template>
              </div>
            </div>
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
import { onMounted, computed, ref } from 'vue'
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
  activateHandler,
  filterPlugins,
  getPluginsDir,
  addCustomPlugin
} = usePluginManager()


const filters = computed<{ label: string; value: PluginFilter; color: { bg: string; text: string; border: string } }[]>(() => [
  // 统一使用「浅色底 + 深色字」方案（和下方 statusBadge 同色系），对比度充足不会看不清
  {
    label: t('plugins.filters.all'),
    value: 'all',
    color: { bg: '#eceff1', text: '#37474f', border: '#cfd8dc' }
  },
  {
    label: t('plugins.filters.enabled'),
    value: 'enabled',
    color: { bg: '#e8f5e9', text: '#1b5e20', border: '#a5d6a7' }
  },
  {
    label: t('plugins.filters.disabled'),
    value: 'disabled',
    color: { bg: '#ffebee', text: '#b71c1c', border: '#ef9a9a' }
  },
  {
    label: t('plugins.filters.browserDefault'),
    value: 'browser-default',
    color: { bg: '#e3f2fd', text: '#0d47a1', border: '#90caf9' }
  },
  {
    label: t('plugins.filters.undeveloped'),
    value: 'undeveloped',
    color: { bg: '#efebe9', text: '#4e342e', border: '#bcaaa4' }
  }
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

const customExt = ref('')
const customFolderPath = ref('')

async function handleBrowseCustomFolder() {
  try {
    const pluginsDir = await getPluginsDir()
    const { invoke } = window.__TAURI__.core
    const chosen = await invoke('choose_folder', { initialDir: pluginsDir }) as string | null
    if (chosen) {
      customFolderPath.value = chosen
    }
  } catch (e) {
    console.error('浏览文件夹失败:', e)
  }
}

async function handleAddCustomPlugin() {
  await addCustomPlugin(customExt.value.trim(), customFolderPath.value)
  customExt.value = ''
  customFolderPath.value = ''
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

function resolveUrl(template: string, pluginId?: string) {
  if (!pluginId) return template
  return template.replace(/\{pluginId\}/g, pluginId)
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

/** 把指定扩展名的指定处理器设为激活（同一扩展名下自动互斥，其它 Handler 的 Enabled → Disabled） */
async function handleActivate(ext: string, handlerId: string) {
  await activateHandler(ext, handlerId)
}

/** 单个处理器的 enable/disable：若改成 enabled 则自动成为激活项（互斥） */
async function handleHandlerToggle(ext: string, handlerId: string, newStatus: string) {
  if (newStatus === 'enabled') {
    // 启用 → activateHandler 会自动把它置为激活项，并保持互斥
    await activateHandler(ext, handlerId)
  } else {
    // 禁用这个处理器：先把同扩展名的其它处理器（若有一个 disabled 但可接替）激活
    // 简化策略：直接 set_extension_status 走后端；如果该 handlerId 不是首个，则再通过 activate 逻辑兜底
    try {
      const { invoke } = window.__TAURI__.core
      // 找到这个扩展名的 handlers，挑一个除当前外 disabled/handlers 的启用
      const cfg = pluginsCache.value.extensions[ext]
      const others = cfg?.handlers?.filter(h => h.handlerId !== handlerId) ?? []
      const fallback = others.find(h => mapStatus(h.status) === 'enabled' || mapStatus(h.status) === 'disabled')
      if (fallback) {
        // 让另一个可接替者成为激活项（它会自动 Enabled），从而当前 active 被踢下去也没问题
        await invoke('activate_plugin_handler', { ext, handlerId: fallback.handlerId }) as string
      }
      // 如果上面失败（或没可接替者），再对目标 handler 做一次 set_status：通过 activateHandler 传一个不存在的 id 不行，
      //   所以退一步：调用 togglePlugin(ext, 'disabled') 使整个扩展名首个 handler 变 disabled，
      //   最终 reload 后 UI 会刷新反映正确状态
      if (!fallback) {
        await togglePlugin(ext, 'disabled')
      }
      await loadPluginsConfig()
    } catch (e) {
      console.error(e)
    }
  }
}

onMounted(async () => {
  await loadPluginsConfig()
})
</script>