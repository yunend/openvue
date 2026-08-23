<template>
  <div class="panel">
    <div class="panel-section">
      <div class="panel-section-title">⚙️ 基础配置</div>
      
      <div class="form-group">
        <label class="form-label">HTTP 服务端口号</label>
        <input
          type="number"
          class="form-input"
          v-model.number="localConfig.port"
          min="1"
          max="65535"
          placeholder="例如：8005"
        >
        <div class="form-hint">取值范围 1 - 65535，修改后需重启 HTTP 服务生效</div>
      </div>
      
      <div class="form-group">
        <label class="form-label">指定文件根目录路径</label>
        <div style="display:flex; gap:10px; align-items:stretch;">
          <input
            type="text"
            class="form-input"
            v-model="localConfig.staticFolder"
            placeholder="例如：static 或 D:/MyWebsite"
            style="flex:1;"
          >
          <button
            type="button"
            class="btn btn-grey"
            style="flex:0 0 auto; padding:0 20px;"
            @click="handleBrowseFolder"
          >
            📁 浏览...
          </button>
        </div>
        <div class="form-hint">支持相对路径或绝对路径；点击右侧按钮可直接选择文件夹</div>
      </div>
      
      <div class="switch-row">
        <div>
          <div class="switch-name">启用文件上传功能</div>
          <div class="switch-desc">开启后，前端用户可通过 /api/upload 上传文件</div>
        </div>
        <label class="toggle-switch">
          <input type="checkbox" v-model="localConfig.enableUpload">
          <span class="slider"></span>
        </label>
      </div>
      
      <div class="control-row">
        <button class="btn btn-purple" @click="loadConfig">
          📥 读取当前配置
        </button>
        <button class="btn btn-orange" @click="handleSaveConfig">
          💾 保存到 config.json
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useConfigManager } from '../../composables/useConfigManager'

const { config, loadConfig, saveConfig, browseFolder } = useConfigManager()
const localConfig = ref({ port: 8005, staticFolder: 'static', enableUpload: false })


onMounted(async () => {
  await loadConfig()
  localConfig.value = { ...config.value }
})

async function handleBrowseFolder() {
  const chosen = await browseFolder(localConfig.value.staticFolder)
  if (chosen) {
    localConfig.value.staticFolder = chosen
  }
}

async function handleSaveConfig() {
  const success = await saveConfig(localConfig.value)
  if (success) {
    localConfig.value = { ...config.value }
  }
}
</script>

<style scoped>
.config-panel {
  animation: fadeIn 0.22s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: translateY(0); }
}

.panel-section {
  background: #f8f9ff;
  border: 1px solid #e8eaf6;
  border-radius: 12px;
  padding: 22px 26px;
  margin-bottom: 22px;
}

.panel-section-title {
  font-size: 1.05rem;
  font-weight: 700;
  color: #1a237e;
  margin-bottom: 16px;
  padding-bottom: 10px;
  border-bottom: 1px solid #e8eaf6;
}

.loading {
  text-align: center;
  padding: 20px;
  color: #5c6bc0;
}

.form-group {
  margin-bottom: 18px;
}

.form-label {
  display: block;
  font-size: 0.9rem;
  color: #1a237e;
  font-weight: 600;
  margin-bottom: 8px;
}

.form-input {
  width: 100%;
  padding: 11px 15px;
  border: 2px solid #c5cae9;
  border-radius: 9px;
  font-size: 1rem;
  color: #1a237e;
  transition: all 0.2s ease;
}

.form-input:focus {
  outline: none;
  border-color: #3f51b5;
  box-shadow: 0 0 0 3px rgba(63, 81, 181, 0.1);
}

.form-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.form-checkbox input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.control-row {
  display: flex;
  gap: 12px;
  margin-top: 24px;
}

.btn-primary,
.btn-secondary {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 0.95rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background: linear-gradient(135deg, #3f51b5, #5c6bc0);
  color: white;
}

.btn-primary:hover {
  box-shadow: 0 4px 12px rgba(63, 81, 181, 0.35);
}

.btn-secondary {
  background: #e8eaf6;
  color: #3949ab;
}

.btn-secondary:hover {
  background: #c5cae9;
}

.message {
  margin-top: 16px;
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 0.9rem;
}

.message.success {
  background: #e8f5e9;
  color: #2e7d32;
  border: 1px solid #a5d6a7;
}

.message.error {
  background: #ffebee;
  color: #c62828;
  border: 1px solid #ef9a9a;
}
</style>