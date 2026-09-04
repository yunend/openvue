<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border-b border-primary-100 px-[26px] py-[22px]">
      <div class="text-[1.05rem] font-bold text-primary-900 mb-4 pb-[10px] border-b border-primary-50">{{ t('plugins.title') }}</div>

      <div class="mb-[18px] px-5 py-4 bg-white border border-dashed border-blue-200 rounded-[10px]">
        <div class="text-[0.95rem] font-semibold text-primary-900 mb-3">{{ t('plugins.customSection') }}</div>
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

      <div class="flex flex-col gap-2">
        <div
          v-if="filteredPlugins.length === 0"
          class="text-center text-primary-300 py-10"
        >
          {{ t('plugins.loading') }}
        </div>
        
        <div
          v-for="plugin in filteredPlugins"
          :key="plugin.ext"
          class="flex items-center gap-4 px-4 py-3 bg-white border border-primary-50 rounded-[10px]"
        >
          <span class="text-[1.4rem] flex-none">{{ fileExtIcon(plugin.ext) }}</span>
          <div class="flex-none w-[100px]">
            <span class="text-[0.95rem] font-semibold text-primary-900">
              .{{ plugin.ext }}
            </span>
          </div>
          <!-- select 需 min-w-0 + w-full 充满剩余空间 -->
          <select
            class="flex-1 min-w-0 w-full px-3 py-1.5 text-[0.88rem] bg-white border border-primary-100 rounded-[7px] text-primary-800 cursor-pointer focus:outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition"
            :value="getCurrentSelectValue(plugin)"
            @change="(e) => handleSelectChange(plugin.ext, (e.target as HTMLSelectElement).value)"
          >
            <!-- 浏览器默认 -->
            <option value="__browser_default__">{{ t('plugins.status.browserDefault') }}</option>
            <!-- 仅显示有 pluginId 的 handler -->
            <option
              v-for="h in plugin.handlers.filter(h => h.pluginId)"
              :key="h.handlerId"
              :value="h.handlerId"
            >
              {{ h.name }}
            </option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePluginManager, fileExtIcon, type PluginItem } from '../../composables/usePluginManager'

defineProps({ isActive: Boolean })

const { t } = useI18n()

const {
  filteredPlugins,
  loadPluginsConfig,
  activateHandler,
  togglePlugin,
  getPluginsDir,
  addCustomPlugin
} = usePluginManager()

const customExt = ref('')
const customFolderPath = ref('')

const BROWSER_DEFAULT = '__browser_default__'

/** 根据插件当前激活状态，计算下拉框应该显示的值 */
function getCurrentSelectValue(plugin: PluginItem): string {
  if (plugin.activeHandlerId) {
    const h = plugin.handlers.find(h => h.handlerId === plugin.activeHandlerId)
    // 激活的 handler 存在 且 有 pluginId（在下拉选项中）→ 显示它
    if (h && h.pluginId) return h.handlerId
  }
  // 激活的 handler 无 pluginId（如 BrowserDefault/Undeveloped），或根本没激活 → 浏览器默认
  return BROWSER_DEFAULT
}

/** 下拉选择变化 */
async function handleSelectChange(ext: string, value: string) {
  if (value === BROWSER_DEFAULT) {
    // 切换到浏览器默认：禁用当前激活的 handler
    await togglePlugin(ext, 'disabled')
  } else {
    // 切换到某个 handler：激活它
    await activateHandler(ext, value)
  }
}

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

onMounted(async () => {
  await loadPluginsConfig()
})
</script>