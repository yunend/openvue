<!-- src-tauri/static/views/UploadView.vue -->
<template>
  <main class="flex-1 flex items-start justify-center py-12">
    <div class="w-10/12 max-w-2xl bg-white rounded-lg shadow-lg p-8">
      <h2 class="text-2xl font-bold text-center text-gray-800 mb-6">多文件上传</h2>

      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <input
            type="file"
            ref="fileInput"
            multiple
            class="w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded file:border-0 file:text-sm file:font-medium file:bg-blue-600 file:text-white hover:file:bg-blue-700 cursor-pointer"
          />
        </div>
        <button
          type="submit"
          :disabled="uploading"
          class="w-full py-3 bg-blue-600 text-white font-semibold rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          上传文件
        </button>
      </form>

      <div class="mt-3 text-sm text-blue-600 text-center">每个文件不能大于1G</div>

      <div v-if="showProgress" class="mt-6">
        <h5 class="text-lg font-semibold text-gray-700 mb-3">上传进度</h5>
        <div class="text-sm text-gray-600 mb-2">
          <strong>当前文件:</strong> {{ currentFileName }}
          <span class="text-gray-500">({{ currentFileSize }})</span>
          <span class="float-right">{{ progressPercent }}%</span>
        </div>

        <div class="w-full bg-gray-200 rounded-full h-6 mb-4 overflow-hidden">
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
            取消上传
          </button>
        </div>
      </div>

      <div class="mt-4 text-center text-gray-600">{{ uploadStatusText }}</div>
    </div>
  </main>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'

const fileInput = ref(null)
const uploading = ref(false)
const showProgress = ref(false)
const currentFileName = ref('')
const currentFileSize = ref('')
const progressPercent = ref(0)
const statusMessage = ref('')
const statusClass = ref('text-gray-600')
const uploadStatusText = ref('')
const uploadEnabled = ref(false)

let currentXHR = null

onMounted(async () => {
  try {
    const res = await fetch('/api/upload-status')
    const data = await res.json()
    uploadEnabled.value = data.enabled
    uploadStatusText.value = data.enabled ? '文件上传功能已启用' : '文件上传功能已禁用'
  } catch {
    uploadStatusText.value = '加载状态失败'
  }
})

function formatFileSize(bytes) {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

async function handleSubmit() {
  const files = fileInput.value?.files
  if (!files || files.length === 0) {
    uploadStatusText.value = '请选择要上传的文件'
    return
  }

  for (const file of files) {
    if (file.size > 1024 * 1024 * 1024) {
      uploadStatusText.value = `文件 "${file.name}" 超过1GB大小限制`
      return
    }
  }

  if (!uploadEnabled.value) {
    uploadStatusText.value = '文件上传功能已被禁用'
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
      if (e.message === 'aborted') {
        setStatus('上传已取消', 'warning')
        uploading.value = false
        return
      }
      setStatus(`文件 "${file.name}" 上传失败: ${e.message}`, 'error')
    }
  }

  uploading.value = false
  if (completedFiles === totalFiles) {
    setStatus(`所有文件上传完成！共上传 ${completedFiles} 个文件，总大小: ${formatFileSize(uploadedSize)}`, 'success')
    fileInput.value.value = ''
    setTimeout(() => {
      showProgress.value = false
    }, 5000)
  } else {
    setStatus(`上传完成，共成功上传 ${completedFiles}/${totalFiles} 个文件`, 'warning')
  }
}

function uploadSingleFile(file) {
  return new Promise((resolve, reject) => {
    const formData = new FormData()
    formData.append('files', file)

    const xhr = new XMLHttpRequest()
    currentXHR = xhr

    currentFileName.value = file.name
    currentFileSize.value = formatFileSize(file.size)

    xhr.upload.addEventListener('progress', (e) => {
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

function setStatus(msg, type) {
  statusMessage.value = msg
  const map = {
    success: 'text-green-600',
    error: 'text-red-600',
    warning: 'text-yellow-600',
    info: 'text-gray-600',
  }
  statusClass.value = map[type] || 'text-gray-600'
}

function cancelUpload() {
  if (currentXHR) {
    currentXHR.abort()
    currentXHR = null
  }
}

onBeforeUnmount(() => {
  if (currentXHR) currentXHR.abort()
})

// Esc 取消上传
function handleKeydown(e) {
  if (e.key === 'Escape' && uploading.value) {
    if (confirm('确定要取消上传吗？')) cancelUpload()
  }
}
onMounted(() => window.addEventListener('keydown', handleKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', handleKeydown))
</script>