/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

/* ========== 通用类型 ========== */

/** 插件扩展名状态 */
type PluginStatus = 'enabled' | 'disabled' | 'browser-default' | 'undeveloped'

/** 插件扩展名条目 */
interface PluginEntry {
  status: PluginStatus
  urlTemplate?: string
  pluginId?: string
}

/** /api/plugins 响应 */
interface PluginsData {
  extensions: Record<string, PluginEntry>
}

/** 文件/目录条目 */
interface FileItem {
  path: string
  name: string
  type: 'file' | 'directory'
  mtime: string
  size?: number
}

/** 帮助链接 */
interface HelpLink {
  url: string
  label: string
}

/** /api/about 响应 */
interface AboutData {
  version: string
  buildStack: string
  config: Record<string, unknown>
  helpLinks: HelpLink[]
}

/** 上传状态响应 */
interface UploadStatusData {
  enabled: boolean
}