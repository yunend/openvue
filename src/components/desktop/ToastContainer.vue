<template>
  <div
    class="fixed top-0 left-0 right-0 z-[99999] flex justify-center px-5 pt-[26px] pb-[10px] min-h-[60px]"
    :class="toast.show ? 'pointer-events-auto' : 'pointer-events-none'"
  >
    <div
      v-if="toast.show"
      class="relative px-[24px] pr-[36px] py-[13px] rounded-[10px] text-white font-semibold shadow-[0_4px_22px_rgba(0,0,0,0.22)] animate-slideDown text-[0.95rem] max-w-[90%]"
      :class="toastClass"
    >
      <span class="break-all">{{ toast.message }}</span>
      <button
        type="button"
        class="absolute top-[6px] right-[8px] w-[22px] h-[22px] flex items-center justify-center rounded-full text-white/70 hover:text-white hover:bg-white/20 transition-colors text-[0.9rem] leading-none cursor-pointer border-none bg-transparent p-0"
        @click="hideToast"
        aria-label="Close"
      >
        ✕
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useToast } from '../../composables/useToast'

const { toast, hideToast } = useToast()

const toastClass = computed(() => {
  const map: Record<string, string> = {
    'success': 'bg-gradient-to-br from-green-700 to-green-500',
    'error': 'bg-gradient-to-br from-red-700 to-red-500',
    'info': 'bg-gradient-to-br from-blue-700 to-blue-500'
  }
  return map[toast.value.type] || map.info
})
</script>