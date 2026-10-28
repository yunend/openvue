<template>
  <main class="flex-1 flex items-start justify-center py-12 bg-page-bg">
    <div class="w-10/12 max-w-2xl bg-card-bg text-text-primary rounded-lg shadow-lg p-8 border border-border-color">
      <h2 class="text-2xl font-bold text-center mb-6 flex items-center justify-center gap-3">
        {{ t('upload.title') }}
        <span
          class="inline-flex items-center justify-center w-8 h-8 rounded-full transition-all duration-300"
          :class="uploadEnabled ? 'bg-green-100 dark:bg-green-900/30' : 'bg-gray-200 dark:bg-gray-700'"
          :title="uploadEnabled ? t('upload.enabled') : t('upload.disabled')"
        >
          <!-- 启用状态图标（带上传箭头） -->
          <svg
            v-if="uploadEnabled"
            class="w-5 h-5 text-green-600 dark:text-green-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
          </svg>
          <!-- 禁用状态图标（灰色+斜线） -->
          <svg
            v-else
            class="w-5 h-5 text-gray-400 dark:text-gray-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636"></path>
          </svg>
        </span>
      </h2>

      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div
          @click="uploadEnabled && fileInput?.click()"
          @dragover.prevent="uploadEnabled && onDragOver()"
          @dragleave.prevent="uploadEnabled && onDragLeave()"
          @drop.prevent="uploadEnabled && onDrop($event)"
          class="border-2 border-dashed rounded-lg p-8 text-center transition-colors"
          :class="[
            uploadEnabled ? 'cursor-pointer' : 'cursor-not-allowed opacity-50',
            isDragging && uploadEnabled
              ? 'border-blue-500 bg-blue-100 dark:bg-blue-900/30 scale-[1.02]'
              : uploadEnabled
                ? 'border-blue-300 dark:border-blue-500 hover:border-blue-500 dark:hover:border-blue-400 bg-blue-50 dark:bg-slate-800/50'
                : 'border-gray-300 dark:border-gray-600 bg-gray-100 dark:bg-slate-700/50'
          ]"
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
          :disabled="uploading || !uploadEnabled"
          class="w-full py-3 bg-blue-600 text-white font-semibold rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ uploading ? t('upload.uploading') : (uploadEnabled ? t('upload.upload') : t('upload.disabled')) }}
        </button>
      </form>

      <div class="mt-3 text-sm text-blue-600 dark:text-blue-400 text-center">{{ t('upload.hint') }}</div>

      <div v-if="showProgress" class="mt-6">
        <div class="text-sm mb-2">
          <strong>{{ t('upload.currentFile') }}:</strong> {{ currentFileName }}
          <span class="text-text-secondary">({{ currentFileSize }})</span>
          <span class="float-right">{{ progressPercent }}%</span>
        </div>

        <div class="w-full bg-gray-200 dark:bg-slate-700 rounded-full h-6 mb-4 overflow-hidden">
          <div
            class="h-full bg-blue-600"
            :style="{ width: progressPercent + '%' }"
          ></div>
        </div>

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
    </div>
  </main>

  <!-- 上传结果弹窗 -->
  <div
    v-if="showResultModal"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    @click.self="closeResultModal"
  >
    <div class="bg-card-bg rounded-lg shadow-xl max-w-lg w-11/12 max-h-[80vh] overflow-hidden">
      <!-- 弹窗头部 -->
      <div
        class="px-6 py-4 flex items-center justify-between border-b border-border-color"
        :class="resultModalType === 'success' ? 'bg-green-50 dark:bg-green-900/20' : resultModalType === 'error' ? 'bg-red-50 dark:bg-red-900/20' : 'bg-yellow-50 dark:bg-yellow-900/20'"
      >
        <h3 class="text-lg font-semibold flex items-center gap-2">
          <span class="text-2xl">{{ resultModalType === 'success' ? '✅' : resultModalType === 'error' ? '❌' : '⚠️' }}</span>
          <span>{{ resultModalTitle }}</span>
        </h3>
        <button
          @click="closeResultModal"
          class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 text-2xl leading-none"
        >
          ×
        </button>
      </div>

      <!-- 弹窗内容 -->
      <div class="px-6 py-4 max-h-[60vh] overflow-y-auto">
        <!-- 成功文件列表 -->
        <div v-if="successFiles.length > 0" class="mb-4">
          <h4 class="text-sm font-semibold text-green-600 dark:text-green-400 mb-2">
            ✅ 成功上传 ({{ successFiles.length }})
          </h4>
          <div class="space-y-2">
            <div
              v-for="(file, index) in successFiles"
              :key="index"
              class="p-3 bg-green-50 dark:bg-green-900/10 rounded border border-green-200 dark:border-green-800"
            >
              <div class="font-medium text-sm">{{ file.name }}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                {{ formatFileSize(file.size) }}
              </div>
            </div>
          </div>
        </div>

        <!-- 失败文件列表 -->
        <div v-if="failedFiles.length > 0" class="mb-4">
          <h4 class="text-sm font-semibold text-red-600 dark:text-red-400 mb-2">
            ❌ 上传失败 ({{ failedFiles.length }})
          </h4>
          <div class="space-y-2">
            <div
              v-for="(file, index) in failedFiles"
              :key="index"
              class="p-3 bg-red-50 dark:bg-red-900/10 rounded border border-red-200 dark:border-red-800"
            >
              <div class="font-medium text-sm">{{ file.name }}</div>
              <div class="text-xs text-red-600 dark:text-red-400 mt-1">
                错误: {{ file.error }}
              </div>
            </div>
          </div>
        </div>

        <!-- 汇总信息 -->
        <div class="mt-4 p-3 bg-gray-50 dark:bg-slate-800 rounded text-sm">
          <div class="text-gray-700 dark:text-gray-300">
            <strong>总计:</strong> {{ totalFilesCount }} 个文件，
            <span class="text-green-600 dark:text-green-400">成功 {{ successFiles.length }}</span>，
            <span v-if="failedFiles.length > 0" class="text-red-600 dark:text-red-400">失败 {{ failedFiles.length }}</span>
          </div>
          <div class="text-gray-600 dark:text-gray-400 mt-1">
            总大小: {{ formatFileSize(totalUploadedSize) }}
          </div>
        </div>
      </div>

      <!-- 弹窗底部按钮 -->
      <div class="px-6 py-4 border-t border-border-color flex justify-end">
        <button
          @click="closeResultModal"
          class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          关闭
        </button>
      </div>
    </div>
  </div>
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
const uploadStatusText = ref('')
const uploadEnabled = ref(false)
const selectedFiles = ref<File[]>([])
const isDragging = ref(false)
const showResultModal = ref(false)
const resultModalType = ref<'success' | 'error' | 'warning'>('success')
const resultModalTitle = ref('')
const successFiles = ref<Array<{ name: string; size: number }>>([])
const failedFiles = ref<Array<{ name: string; size: number; error: string }>>([])
const totalFilesCount = ref(0)
const totalUploadedSize = ref(0)

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


  let completedFiles = 0
  let uploadedSize = 0
  successFiles.value = []
  failedFiles.value = []
  totalFilesCount.value = totalFiles

  for (const file of files) {
    try {
      await uploadSingleFile(file)
      progressPercent.value = 0
      completedFiles++
      uploadedSize += file.size
      successFiles.value.push({ name: file.name, size: file.size })
    
    } catch (e) {
      const err = e as Error
      if (err.message === 'aborted') {
      
        uploading.value = false
        return
      }
      failedFiles.value.push({ name: file.name, size: file.size, error: err.message })
    }
  }

  uploading.value = false
  totalUploadedSize.value = uploadedSize

  // 显示结果弹窗
  if (completedFiles === totalFiles) {
    resultModalType.value = 'success'
    resultModalTitle.value = '上传完成'
  
    if (fileInput.value) fileInput.value.value = ''
  } else if (completedFiles === 0) {
    resultModalType.value = 'error'
    resultModalTitle.value = '上传失败'
   
  } else {
    resultModalType.value = 'warning'
    resultModalTitle.value = '部分上传成功'
  }
  
  showResultModal.value = true
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

function cancelUpload(): void {
  if (!confirm('确定要取消上传吗？')) return
  
  if (currentXHR) {
    currentXHR.abort()
    currentXHR = null
  }
  
  // 清除当前文件相关内容
  showProgress.value = false
  currentFileName.value = ''
  currentFileSize.value = ''
  progressPercent.value = 0
  selectedFiles.value = []
  if (fileInput.value) fileInput.value.value = ''
}

function closeResultModal(): void {
  showResultModal.value = false
  showProgress.value = false
  selectedFiles.value = []
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