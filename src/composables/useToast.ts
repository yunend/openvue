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

export function useToast() {
  function showToast(message: string, type: ToastType = 'info'): void {
    toast.value = { show: true, message, type }
  }

  function hideToast(): void {
    toast.value.show = false
  }

  return { toast, showToast, hideToast }
}