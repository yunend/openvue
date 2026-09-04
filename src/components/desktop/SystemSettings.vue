<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border-b border-primary-100 px-[26px] py-[22px]">
      <div class="text-[1.05rem] font-bold text-primary-900 mb-4 pb-[10px] border-b border-primary-50">{{ t('system.title') }}</div>
      
      <div class="flex items-center justify-between px-5 py-4 bg-white border border-primary-50 rounded-[10px] mb-[14px]">
        <div class="flex-1">
          <div class="text-base font-semibold text-primary-900 mb-[3px]">{{ t('system.autoStartLabel') }}</div>
          <div class="text-[0.82rem] text-primary-300">{{ t('system.autoStartHint') }}</div>
        </div>
        <div class="flex items-center gap-[14px]">
          <span
            class="text-[0.85rem] font-semibold"
            :class="autoStartEnabled ? 'text-green-700' : 'text-red-700'"
          >
            {{ autoStartEnabled ? t('system.enabled') : t('system.disabled') }}
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
      
      
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSystemSettings } from '../../composables/useSystemSettings'

defineProps({ isActive: Boolean })

const { t } = useI18n()

const {
  autoStartEnabled,
  initAutostartStatus,
  toggleAutostart,
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