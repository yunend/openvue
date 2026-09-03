<template>
  <div 
    class="fixed inset-0 bg-black/55 z-50 flex items-center justify-center p-4"
    @click.self="$emit('close')"
  >
    <div class="bg-card-bg rounded-xl shadow-2xl w-full max-w-xl max-h-[85vh] flex flex-col overflow-hidden">
      <div class="bg-nav-bg text-nav-text px-6 py-4 flex items-center justify-between">
        <h2 class="text-xl font-bold tracking-wide">ℹ️ {{ t('about.title') }}</h2>
        <button 
          @click="$emit('close')"
          class="text-3xl leading-none opacity-90 hover:opacity-60 transition select-none px-2 -mr-2"
        >×</button>
      </div>

      <div class="px-6 py-5 overflow-y-auto space-y-5">
        <div v-if="aboutLoading" class="text-center py-10 text-text-secondary">
          <div class="text-4xl mb-3 animate-bounce">⏳</div>
          {{ t('about.loading') }}
        </div>

        <template v-else-if="aboutData">
           <div class="bg-card-alt border border-card-alt-border rounded-lg p-4 space-y-3">
            <div>
              <div class="text-sm text-text-secondary mb-1">{{ t('about.version') }}</div>
              <div class="flex items-baseline gap-3">
                <span class="text-3xl font-bold text-link-text font-mono">v{{ aboutData.version }}</span>
              </div>
              <div class="text-xs text-text-secondary mt-2">{{ t('about.techStack', { stack: aboutData.buildStack }) }}</div>
            </div>

            <div class="border-t border-border-color pt-3">
              <div class="text-sm text-text-secondary mb-2">⚙️ {{ t('about.config') }}</div>
              <div class="space-y-2">
                <div class="flex items-center gap-2">
                  <span class="text-sm text-text-secondary">{{ t('about.fileUpload') }}：</span>
                  <span :class="aboutData.config?.enableUpload ? 'text-green-600 font-medium' : 'text-red-600'">
                    {{ aboutData.config?.enableUpload ? t('about.enabled') : t('about.disabled') }}
                  </span>
                </div>
                <div class="flex items-center gap-2">
                   <span class="text-sm text-text-secondary">{{ t('about.port') }}：</span>
                  <span class="font-mono font-medium text-text-primary">{{ aboutData.config?.port }}</span>
                </div>
                <div class="flex items-start gap-2">
                  <span class="text-sm text-text-secondary whitespace-nowrap">{{ t('about.publicFolder') }}：</span>
                  <span class="font-mono text-xs break-all text-text-primary">{{ aboutData.config?.publicFolder }}</span>
                </div>
              </div>
            </div>
          </div>

          <div>
            <h3 class="font-bold text-text-primary mb-2">🧩 {{ t('about.extensions') }}</h3>
            <div v-if="enabledPlugins.length > 0" class="flex flex-wrap gap-2">
              <span 
                v-for="plugin in enabledPlugins" 
                :key="plugin.ext"
                class="px-2 py-1 rounded text-xs font-mono bg-green-100 text-green-800 border border-green-300 cursor-help"
                :title="plugin.activeHandler?.name || plugin.activeHandler?.description || ''"
              >
                .{{ plugin.ext }}
              </span>
            </div>
            <div v-else class="text-sm text-text-secondary">{{ t('about.noExtensions') }}</div>
          </div>

          <div>
            <h3 class="font-bold text-text-primary mb-3">🔗 {{ t('about.help') }}</h3>
            <ul class="grid grid-cols-1 sm:grid-cols-2 gap-2">
              <li v-for="link in aboutData.helpLinks" :key="link.url">
                <a 
                  :href="link.url" 
                  target="_blank" 
                  rel="noopener noreferrer"
                   class="block px-3 py-2 border border-border-color rounded text-text-primary hover:bg-card-alt hover:border-card-alt-border hover:text-link-text transition text-sm"
                >
                  {{ link.label }}
                </a>
              </li>
            </ul>
          </div>
        </template>

        <div v-else class="bg-red-50 border border-red-200 rounded-lg p-4 text-center text-red-600">
          <div class="text-3xl mb-2">❌</div>
          <div>{{ t('about.loadError', { error: aboutError || '未知错误' }) }}</div>
        </div>
      </div>

      <div class="bg-page-bg px-6 py-3 border-t border-border-color flex justify-end">
        <button 
          @click="$emit('close')"
          class="px-5 py-2 bg-nav-bg text-nav-text rounded-lg hover:opacity-80 transition shadow-sm"
        >
          {{ t('about.close') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAboutModal } from '../../composables/useAboutModal'

const { t } = useI18n()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const {
  aboutLoading,
  aboutData,
  aboutError,
  aboutPlugins,
  loadAboutInfo,
  copiedConfig,
  copyConfigToClipboard,
  statusLabel,
  statusBadgeClass
} = useAboutModal()

const prettyConfigJson = computed(() => {
  if (!aboutData.value) return ''
  return JSON.stringify(aboutData.value.config, null, 2)
})

const enabledPlugins = computed(() => {
  if (!aboutPlugins.value) return []
  return Object.values(aboutPlugins.value)
    .filter(p => p.activeStatus === 'enabled')
    .map(p => ({
      ext: p.ext,
      activeHandlerId: p.activeHandlerId,
      activeStatus: p.activeStatus,
      activeHandler: p.activeHandler,
    }))
})

onMounted(() => {
  loadAboutInfo()
})
</script>