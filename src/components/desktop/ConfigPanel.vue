<template>
  <div class="animate-fadeIn" :class="isActive ? 'block' : 'hidden'">
    <div class="bg-primary-50 border border-primary-50 rounded-xl px-[26px] py-[22px] mb-[22px]">
      <div class="text-[1.05rem] font-bold text-primary-900 mb-4 pb-[10px] border-b border-primary-50">⚙️ 基础配置</div>
      
      <div class="mb-[18px]">
        <label class="block text-[0.9rem] text-primary-900 font-semibold mb-2">HTTP 服务端口号</label>
        <input
          type="number"
          class="w-full px-[15px] py-[11px] border-2 border-primary-100 rounded-[9px] text-base text-primary-900 bg-white transition-colors duration-200 focus:outline-none focus:border-primary-500"
          v-model.number="localConfig.port"
          min="1"
          max="65535"
          placeholder="例如：8005"
        >
        <div class="text-[0.8rem] text-primary-300 mt-[5px]">取值范围 1 - 65535，修改后需重启 HTTP 服务生效</div>
      </div>
      
      <div class="mb-[18px]">
        <label class="block text-[0.9rem] text-primary-900 font-semibold mb-2">指定文件根目录路径</label>
        <div class="flex gap-[10px] items-stretch">
          <input
            type="text"
            class="flex-1 px-[15px] py-[11px] border-2 border-primary-100 rounded-[9px] text-base text-primary-900 bg-white transition-colors duration-200 focus:outline-none focus:border-primary-500"
            v-model="localConfig.publicFolder"
            placeholder="例如：public 或 D:/MyWebsite"
          >
          <button
            type="button"
            class="flex-none px-5 py-0 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-slate-500 text-white hover:bg-slate-600"
            @click="handleBrowseFolder"
          >
            📁 浏览...
          </button>
        </div>
        <div class="text-[0.8rem] text-primary-300 mt-[5px]">支持相对路径或绝对路径；点击右侧按钮可直接选择文件夹</div>
      </div>
      
      <div class="flex items-center justify-between px-[18px] py-[14px] bg-white border border-primary-50 rounded-[10px] mb-[14px]">
        <div>
          <div class="text-[0.95rem] font-semibold text-primary-900">启用文件上传功能</div>
          <div class="text-[0.8rem] text-primary-300 mt-[2px]">开启后，前端用户可通过 /api/upload 上传文件</div>
        </div>
        <label class="toggle-switch">
          <input type="checkbox" v-model="localConfig.enableUpload">
          <span class="slider"></span>
        </label>
      </div>
      
      <div class="flex gap-3 mt-6">
        <button class="flex-1 px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-violet-500 text-white hover:bg-violet-600" @click="loadConfig">
          📥 读取当前配置
        </button>
        <button class="flex-1 px-[18px] py-3 text-[0.95rem] font-semibold border-none rounded-[9px] cursor-pointer transition-all duration-200 whitespace-nowrap bg-orange-500 text-white hover:bg-orange-600" @click="handleSaveConfig">
          💾 保存到 config.json
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useConfigManager } from '../../composables/useConfigManager'

defineProps({ isActive: Boolean })

const { config, loadConfig, saveConfig, browseFolder } = useConfigManager()
const localConfig = ref({ port: 8005, publicFolder: 'public', enableUpload: false })


onMounted(async () => {
  await loadConfig()
  localConfig.value = { ...config.value }
})

async function handleBrowseFolder() {
  const chosen = await browseFolder(localConfig.value.publicFolder)
  if (chosen) {
    localConfig.value.publicFolder = chosen
  }
}

async function handleSaveConfig() {
  const success = await saveConfig(localConfig.value)
  if (success) {
    localConfig.value = { ...config.value }
  }
}
</script>