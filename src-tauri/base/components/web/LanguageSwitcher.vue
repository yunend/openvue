<template>
  <div class="relative">
    <button
      @click="toggleDropdown"
      class="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-white/20 transition duration-200"
      :class="isOpen ? 'bg-white/20' : ''"
    >
      <span class="text-lg">🌐</span>
      <span class="text-sm hidden sm:inline">{{ currentLocaleLabel }}</span>
      <svg 
        class="w-4 h-4 transition-transform duration-200" 
        :class="isOpen ? 'rotate-180' : ''"
        viewBox="0 0 20 20" 
        fill="currentColor"
      >
        <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.06l3.71-3.83a.75.75 0 111.08 1.04l-4.25 4.39a.75.75 0 01-1.08 0L5.21 8.27a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
      </svg>
    </button>

    <div
      v-if="isOpen"
      class="absolute right-0 mt-2 w-32 rounded-lg shadow-lg bg-white dark:bg-slate-800 border border-gray-200 dark:border-slate-700 py-1 z-50"
    >
      <button
        v-for="lang in languages"
        :key="lang.code"
        @click="selectLanguage(lang.code)"
        class="w-full flex items-center gap-3 px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-slate-700 transition-colors"
        :class="currentLocale === lang.code ? 'text-blue-600 dark:text-blue-400 font-medium' : 'text-gray-700 dark:text-gray-300'"
      >
        <span>{{ lang.label }}</span>
        <span v-if="currentLocale === lang.code" class="ml-auto">✓</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { setLocale, getCurrentLocale, type LocaleKey } from '../../i18n'

const { locale } = useI18n()

const isOpen = ref(false)
const currentLocale = ref<LocaleKey>(getCurrentLocale())

const languages = [
  { code: 'zh-CN' as LocaleKey, label: '中文' },
  { code: 'en' as LocaleKey, label: 'English' },
]

const currentLocaleLabel = computed(() => {
  return languages.find(l => l.code === currentLocale.value)?.label || '中文'
})

function toggleDropdown(): void {
  isOpen.value = !isOpen.value
}

function selectLanguage(code: LocaleKey): void {
  setLocale(code)
  currentLocale.value = code
  locale.value = code
  isOpen.value = false
}

function closeDropdown(event: MouseEvent): void {
  const target = event.target as HTMLElement
  if (!target.closest('.relative')) {
    isOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', closeDropdown)
})

onUnmounted(() => {
  document.removeEventListener('click', closeDropdown)
})
</script>