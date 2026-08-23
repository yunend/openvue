import { ref } from 'vue'

type ToastType = 'info' | 'success' | 'error'

interface ToastState {
  show: boolean
  message: string
  type: ToastType
}

const toast = ref<ToastState>({
  show: false,
  message: '',
  type: 'info'
})

let hideTimer: ReturnType<typeof setTimeout> | null = null

export function useToast() {
  function showToast(message: string, type: ToastType = 'info', duration: number = 2800): void {
    if (hideTimer) {
      clearTimeout(hideTimer)
      hideTimer = null
    }
    toast.value = { show: true, message, type }
    hideTimer = setTimeout(() => {
      toast.value.show = false
    }, duration)
  }

  function hideToast(): void {
    toast.value.show = false
    if (hideTimer) {
      clearTimeout(hideTimer)
      hideTimer = null
    }
  }

  return { toast, showToast, hideToast }
}