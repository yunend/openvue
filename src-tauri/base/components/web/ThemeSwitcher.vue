<template>
  <div class="relative">
    <button
      @click="toggleDropdown"
      class="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-white/20 transition duration-200"
      :class="isOpen ? 'bg-white/20' : ''"
    >
      <span class="text-lg">{{ currentThemeOption.icon }}</span>
      <span class="text-sm hidden sm:inline">{{ currentThemeOption.label }}</span>
      <svg 
        class="w-4 h-4 transition-transform duration-200" 
        :class="isOpen ? 'rotate-180' : ''"
        viewBox="0 0 20 20" 
        fill="currentColor"
      >
        <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.06l3.71-3.83a.75.75 0 111.08 1.04l-4.25 4.39a.75.75 0 01-1.08 0L5.21 8.27a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
      </svg>
    </button>

    <!-- 下拉菜单 -->
    <div
      v-if="isOpen"
      class="absolute right-0 mt-2 w-40 rounded-lg shadow-lg bg-white dark:bg-slate-800 border border-gray-200 dark:border-slate-700 py-1 z-50"
    >
      <button
        v-for="theme in themes"
        :key="theme.name"
        @click="selectTheme(theme.name)"
        class="w-full flex items-center gap-3 px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-slate-700 transition-colors"
        :class="currentTheme === theme.name ? 'text-blue-600 dark:text-blue-400 font-medium' : 'text-gray-700 dark:text-gray-300'"
      >
        <span class="text-base">{{ theme.icon }}</span>
        <span>{{ theme.label }}</span>
        <span v-if="currentTheme === theme.name" class="ml-auto">✓</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useTheme } from '../../composables/useTheme'

const { currentTheme, themes, setTheme, getCurrentThemeOption } = useTheme()

const isOpen = ref(false)

const currentThemeOption = computed(() => getCurrentThemeOption())

function toggleDropdown(): void {
  isOpen.value = !isOpen.value
}

function selectTheme(themeName: string): void {
  setTheme(themeName as any)
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