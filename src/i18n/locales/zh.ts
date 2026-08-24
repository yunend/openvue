const zh = {
  app: {
    title: '🎛️ Tauri 控制台',
    subtitle: 'HTTP 文件服务管理器',
    version: '版本',
    restartService: '🔄 重启HTTP服务',
    copy: '复制',
    copied: '已复制'
  },
  nav: {
    status: '当前服务状态',
    config: '配置文件管理',
    system: '系统设置',
    plugins: '插件配置',
    about: '关于'
  },
  status: {
    overview: '🟢 运行概览',
    httpService: 'HTTP 服务',
    running: '✅ 运行中',
    stopped: '⏸️ 未启动',
    listeningPort: '监听端口',
    publicFolder: '指定文件根目录',
    accessUrl: '访问地址',
    serviceStopped: '（服务未启动）',
    start: '▶ 启动服务',
    stop: '■ 停止服务',
    refresh: '🔄 刷新',
    autoStarting: '🔌 正在自动启动 HTTP 服务...',
    openLinkFailed: '打开链接失败',
    copyFailed: '复制失败'
  },
  config: {
    basicTitle: '⚙️ 基础配置',
    portLabel: 'HTTP 服务端口号',
    portPlaceholder: '例如：8005',
    portHint: '取值范围 1 - 65535，修改后需重启 HTTP 服务生效',
    folderLabel: '指定文件根目录路径',
    folderPlaceholder: '例如：public 或 D:/MyWebsite',
    folderHint: '支持相对路径或绝对路径；点击右侧按钮可直接选择文件夹',
    browse: '📁 浏览...',
    enableUpload: '启用文件上传功能',
    enableUploadHint: '开启后，前端用户可通过 /api/upload 上传文件',
    load: '📥 读取当前配置',
    save: '💾 保存到 config.json'
  },
  system: {
    title: '🖥️ 操作系统行为',
    autoStartLabel: '开机自动启动',
    autoStartHint: 'Windows 用户登录后自动启动本应用',
    enabled: '已启用',
    disabled: '已禁用',
    minimizeLabel: '最小化到系统托盘',
    minimizeHint: '隐藏主窗口，HTTP 服务继续在后台运行',
    hideToTray: '📦 隐藏到托盘',
    quitLabel: '退出应用程序',
    quitHint: '停止 HTTP 服务并完全关闭本程序',
    quit: '🚪 退出应用'
  },
  plugins: {
    title: '🧩 扩展名与插件映射表',
    desc1: '• <b class="text-primary-700">已启用</b>：点击对应插件页打开（需 plugins 目录下已放置资源）',
    desc2: '• <b class="text-primary-700">未启用</b>：虽安装了插件，但暂不使用，按浏览器默认行为打开',
    desc3: '• <b class="text-primary-700">浏览器默认支持</b>：浏览器原生能渲染的格式（图片/视频/HTML 等）',
    desc4: '• <b class="text-primary-700">未开发</b>：未来计划支持，当前走浏览器默认（会直接下载）',
    filters: {
      all: '全部',
      enabled: '✅ 已启用',
      disabled: '⏸️ 未启用',
      browserDefault: '🌐 浏览器默认',
      undeveloped: '🚧 未开发'
    },
    empty: '该分类下暂无条目',
    loading: '⏳ 正在加载插件配置...',
    fileSuffix: ' 文件',
    pluginId: '插件ID',
    status: {
      enabled: '✅ 已启用',
      disabled: '⏸️ 未启用',
      browserDefault: '🌐 浏览器默认支持',
      undeveloped: '🚧 未开发'
    },
    disable: '⏸️ 禁用',
    enable: '▶️ 启用',
    reload: '🔄 重新加载配置',
    clear: '🗑️ 清除筛选'
  },
  about: {
    desc: '基于 Tauri 2 + Axum 构建的轻量级本地 HTTP 文件服务工具。<br>一键启动、开机自启、托盘常驻、目录浏览、文件上传，<br>配置灵活，开箱即用。',
    currentVersion: '当前版本',
    platform: '构建平台',
    gui: 'GUI 框架',
    backend: '后端语言',
    checkUpdate: '检查更新',
    checking: '检查中...',
    github: 'GitHub 仓库',
    tauriDocs: 'Tauri 官方文档',
    latest: '✅ 已是最新版本（{version}）',
    available: '🎉 发现新版本 <strong>{latest}</strong>（当前 {current}）<br><a href="{url}" target="_blank" onclick="event.preventDefault(); window.open(\'{url}\')">前往下载 →</a>',
    noVersion: '未找到版本信息',
    apiError: 'GitHub API 返回 {status}',
    updateError: '❌ 检查更新失败：{msg}<br>请检查网络连接后重试。'
  },
  toast: {
    pluginsLoaded: '🧩 插件配置已加载',
    pluginsLoadFailed: '加载插件配置失败: {err}',
    pluginUpdating: '💾 正在更新 .{ext} 状态...',
    pluginToggleFailed: '❌ 切换失败: {err}',
    refreshFailed: '刷新状态失败: {err}',
    started: '✅ HTTP 服务已启动',
    startFailed: '❌ {err}',
    stopped: '🛑 HTTP 服务已停止',
    stopFailed: '❌ {err}',
    restarting: '🔄 正在重启 HTTP 服务...',
    restartFailed: '❌ 重启 HTTP 服务失败: {err}',
    configLoaded: '📥 配置已读取',
    configLoadFailed: '读取配置失败: {err}',
    invalidPort: '请输入有效的端口号 (1-65535)',
    emptyFolder: '请输入指定文件目录路径',
    saved: '💾 配置已保存',
    saveFailed: '保存失败: {err}',
    pluginUpdated: '✅ 插件状态已更新',
    folderSelected: '📁 已选择目录: {path}',
    browseFailed: '打开文件夹选择失败: {err}',
    autoStartEnabled: '✅ 已开启开机自启动',
    autoStartDisabled: '❌ 已关闭开机自启动',
    operationFailed: '操作失败: {err}',
    minimized: '📦 已最小化到系统托盘',
    hideFailed: '隐藏失败: {err}',
    quitConfirm: '⚠️ HTTP 服务正在运行，确定要退出吗？'
  },
  language: {
    switchTo: 'English',
    label: '语言'
  }
}

export default zh