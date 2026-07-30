import { ref } from 'vue'

// 单一 Toast 状态（固定位置，无内容时隐藏）
const toast = ref({
  show: false,
  message: '',
  type: 'info'
})

let hideTimer = null

export function useToast() {
  function showToast(message, type = 'info', duration = 2800) {
    // 清除之前的定时器
    if (hideTimer) {
      clearTimeout(hideTimer)
      hideTimer = null
    }
    
    // 显示新消息
    toast.value = { show: true, message, type }
    
    // 自动隐藏
    hideTimer = setTimeout(() => {
      toast.value.show = false
    }, duration)
  }

  function hideToast() {
    toast.value.show = false
    if (hideTimer) {
      clearTimeout(hideTimer)
      hideTimer = null
    }
  }

  return { toast, showToast, hideToast }
}