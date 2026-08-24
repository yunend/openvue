<!-- src-tauri/static/views/UploadView.vue -->
<template>
  <main class="flex-1 flex items-start justify-center py-12 bg-page-bg">
    <div class="w-10/12 max-w-2xl bg-card-bg text-text-primary rounded-lg shadow-lg p-8 border border-border-color">
      <h2 class="text-2xl font-bold text-center mb-6">{{ t('upload.title') }}</h2>

      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div
          @click="fileInput?.click()"
          @dragover.prevent="onDragOver"
          @dragleave.prevent="onDragLeave"
          @drop.prevent="onDrop"
          class="border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors"
          :class="isDragging 
            ? 'border-blue-500 bg-blue-100 dark:bg-blue-900/30 scale-[1.02]' 
            : 'border-blue-300 dark:border-blue-500 hover:border-blue-500 dark:hover:border-blue-400 bg-blue-50 dark:bg-slate-800/50'"
        >
          <div class="text-4xl mb-3">{{ isDragging ? '📥' : '📁' }}</div>
          <div class="text-blue-600 dark:text-blue-400 font-medium mb-1">
            {{ isDragging ? t('upload.dropHere') : (selectedFiles.length > 0 ? `${t('upload.selectedFiles')}: ${selectedFiles.length}` : t('upload.selectFile')) }}
          </div>
          <div class="text-sm text-gray-500 dark:text-gray-400">
            {{ selectedFiles.length > 0 ? selectedFileNames : t('upload.dragHint') }}
          </div>
          <input
            type="file"
            ref="fileInput"
            multiple
            class="hidden"
            @change="onFileChange"
          />
        </div>
        <button
          type="submit"
          :disabled="uploading"
          class="w-full py-3 bg-blue-600 text-white font-semibold rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ uploading ? t('upload.uploading') : t('upload.upload') }}
        </button>
      </form>

      <div class="mt-3 text-sm text-blue-600 dark:text-blue-400 text-center">{{ t('upload.hint') }}</div>

      <div v-if="showProgress" class="mt-6">
        <h5 class="text-lg font-semibold mb-3">{{ t('upload.currentFile') }}</h5>
        <div class="text-sm mb-2">
          <strong>{{ t('upload.currentFile') }}:</strong> {{ currentFileName }}
          <span class="text-text-secondary">({{ currentFileSize }})</span>
          <span class="float-right">{{ progressPercent }}%</span>
        </div>

        <div class="w-full bg-gray-200 dark:bg-slate-700 rounded-full h-6 mb-4 overflow-hidden">
          <div
            class="h-full bg-blue-600 transition-all duration-300"
            :style="{ width: progressPercent + '%' }"
          ></div>
        </div>

        <div class="text-sm" :class="statusClass" v-html="statusMessage"></div>

        <div class="text-center mt-4">
          <button
            v-if="uploading"
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 transition-colors"
            @click="cancelUpload"
          >
            {{ t('upload.cancel') }}
          </button>
        </div>
      </div>

      <div class="mt-4 text-center text-text-secondary">{{ uploadStatusText }}</div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const fileInput = ref<HTMLInputElement | null>(null)
const uploading = ref(false)
const showProgress = ref(false)
const currentFileName = ref('')
const currentFileSize = ref('')
const progressPercent = ref(0)
const statusMessage = ref('')
const statusClass = ref('text-gray-600')
const uploadStatusText = ref('')
const uploadEnabled = ref(false)
const selectedFiles = ref<File[]>([])
const isDragging = ref(false)

const selectedFileNames = computed(() => {
  return selectedFiles.value.map(f => f.name).join(', ')
})

interface UploadStatusData {
  enabled: boolean
}

function onFileChange(): void {
  const files = fileInput.value?.files
  if (files) {
    selectedFiles.value = Array.from(files)
  }
}

function onDragOver(): void {
  isDragging.value = true
}

function onDragLeave(): void {
  isDragging.value = false
}

function onDrop(event: DragEvent): void {
  isDragging.value = false
  if (event.dataTransfer?.files) {
    selectedFiles.value = Array.from(event.dataTransfer.files)
  }
}

let currentXHR: XMLHttpRequest | null = null

onMounted(async () => {
  try {
    const res = await fetch('/api/upload-status')
    const data: UploadStatusData = await res.json()
    uploadEnabled.value = data.enabled
    uploadStatusText.value = data.enabled ? t('upload.enabled') : t('upload.disabled')
  } catch {
    uploadStatusText.value = '加载状态失败'
  }
})

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

async function handleSubmit(): Promise<void> {
  // 使用 selectedFiles（支持拖拽和点击选择）
  const files = selectedFiles.value
  if (!files || files.length === 0) {
    uploadStatusText.value = t('upload.selectFile')
    return
  }

  for (const file of files) {
    if (file.size > 1024 * 1024 * 1024) {
      uploadStatusText.value = `文件 "${file.name}" 超过1GB大小限制`
      return
    }
  }

  if (!uploadEnabled.value) {
    uploadStatusText.value = t('upload.disabled')
    return
  }

  uploadStatusText.value = ''
  showProgress.value = true
  uploading.value = true

  const totalFiles = files.length
  let totalSize = 0
  for (const file of files) totalSize += file.size

  setStatus(`准备上传 ${totalFiles} 个文件，总大小: ${formatFileSize(totalSize)}`, 'info')

  let completedFiles = 0
  let uploadedSize = 0

  for (const file of files) {
    try {
      await uploadSingleFile(file)
      completedFiles++
      uploadedSize += file.size
      setStatus(`文件 "${file.name}" 上传成功`, 'success')
    } catch (e) {
      const err = e as Error
      if (err.message === 'aborted') {
        setStatus('上传已取消', 'warning')
        uploading.value = false
        return
      }
      setStatus(`文件 "${file.name}" 上传失败: ${err.message}`, 'error')
    }
  }

  uploading.value = false
  if (completedFiles === totalFiles) {
    setStatus(`所有文件上传完成！共上传 ${completedFiles} 个文件，总大小: ${formatFileSize(uploadedSize)}`, 'success')
    if (fileInput.value) fileInput.value.value = ''
    setTimeout(() => {
      showProgress.value = false
    }, 5000)
  } else {
    setStatus(`上传完成，共成功上传 ${completedFiles}/${totalFiles} 个文件`, 'warning')
  }
}

function uploadSingleFile(file: File): Promise<void> {
  return new Promise((resolve, reject) => {
    const formData = new FormData()
    formData.append('files', file)

    const xhr = new XMLHttpRequest()
    currentXHR = xhr

    currentFileName.value = file.name
    currentFileSize.value = formatFileSize(file.size)

    xhr.upload.addEventListener('progress', (e: ProgressEvent) => {
      if (e.lengthComputable) {
        progressPercent.value = Math.round((e.loaded / e.total) * 100)
      }
    })

    xhr.addEventListener('load', () => {
      if (xhr.status === 200) resolve()
      else reject(new Error(xhr.statusText))
    })

    xhr.addEventListener('error', () => reject(new Error('网络错误')))
    xhr.addEventListener('abort', () => reject(new Error('aborted')))

    xhr.open('POST', '/upload')
    xhr.send(formData)
  })
}

function setStatus(msg: string, type: 'success' | 'error' | 'warning' | 'info'): void {
  statusMessage.value = msg
  const map: Record<string, string> = {
    success: 'text-green-600',
    error: 'text-red-600',
    warning: 'text-yellow-600',
    info: 'text-gray-600',
  }
  statusClass.value = map[type] || 'text-gray-600'
}

function cancelUpload(): void {
  if (currentXHR) {
    currentXHR.abort()
    currentXHR = null
  }
}

onBeforeUnmount(() => {
  if (currentXHR) currentXHR.abort()
})

// Esc 取消上传
function handleKeydown(e: KeyboardEvent): void {
  if (e.key === 'Escape' && uploading.value) {
    if (confirm('确定要取消上传吗？')) cancelUpload()
  }
}
onMounted(() => window.addEventListener('keydown', handleKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', handleKeydown))
</script>