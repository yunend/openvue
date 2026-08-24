const en = {
  app: {
    title: '🎛️ Tauri Console',
    subtitle: 'HTTP File Server Manager',
    version: 'Version',
    restartService: '🔄 Restart HTTP Service',
    copy: 'Copy',
    copied: 'Copied'
  },
  nav: {
    status: 'Status',
    config: 'Config',
    system: 'System',
    plugins: 'Plugins',
    about: 'About'
  },
  status: {
    overview: '🟢 Overview',
    httpService: 'HTTP Service',
    running: '✅ Running',
    stopped: '⏸️ Stopped',
    listeningPort: 'Listening Port',
    publicFolder: 'Public Folder',
    accessUrl: 'Access URL',
    serviceStopped: '(service not started)',
    start: '▶ Start',
    stop: '■ Stop',
    refresh: '🔄 Refresh',
    autoStarting: '🔌 Auto-starting HTTP service...',
    openLinkFailed: 'Failed to open link',
    copyFailed: 'Failed to copy'
  },
  config: {
    basicTitle: '⚙️ Basic Config',
    portLabel: 'HTTP Service Port',
    portPlaceholder: 'e.g. 8005',
    portHint: 'Range 1 - 65535, restart HTTP service after change',
    folderLabel: 'Public Folder Path',
    folderPlaceholder: 'e.g. public or D:/MyWebsite',
    folderHint: 'Supports relative or absolute paths; click the button to browse',
    browse: '📁 Browse...',
    enableUpload: 'Enable File Upload',
    enableUploadHint: 'Allow users to upload files via /api/upload',
    load: '📥 Load Config',
    save: '💾 Save to config.json'
  },
  system: {
    title: '🖥️ OS Behavior',
    autoStartLabel: 'Auto Start on Boot',
    autoStartHint: 'Auto start this app when Windows user logs in',
    enabled: 'Enabled',
    disabled: 'Disabled',
    minimizeLabel: 'Minimize to Tray',
    minimizeHint: 'Hide main window, HTTP service keeps running',
    hideToTray: '📦 Hide to Tray',
    quitLabel: 'Quit App',
    quitHint: 'Stop HTTP service and completely close the app',
    quit: '🚪 Quit'
  },
  plugins: {
    title: '🧩 Extension & Plugin Mapping',
    desc1: '• <b class="text-primary-700">Enabled</b>: Click to open in plugin page (requires plugin files in plugins dir)',
    desc2: '• <b class="text-primary-700">Disabled</b>: Plugin installed but not used, opens with browser default behavior',
    desc3: '• <b class="text-primary-700">Browser Default</b>: Natively renderable formats (image/video/HTML etc.)',
    desc4: '• <b class="text-primary-700">Undeveloped</b>: Planned for future, currently uses browser default (direct download)',
    filters: {
      all: 'All',
      enabled: '✅ Enabled',
      disabled: '⏸️ Disabled',
      browserDefault: '🌐 Browser Default',
      undeveloped: '🚧 Undeveloped'
    },
    empty: 'No items in this category',
    loading: '⏳ Loading plugin config...',
    fileSuffix: ' File',
    pluginId: 'Plugin ID',
    status: {
      enabled: '✅ Enabled',
      disabled: '⏸️ Disabled',
      browserDefault: '🌐 Browser Default',
      undeveloped: '🚧 Undeveloped'
    },
    disable: '⏸️ Disable',
    enable: '▶️ Enable',
    reload: '🔄 Reload Config',
    clear: '🗑️ Clear Filter'
  },
  about: {
    desc: 'A lightweight local HTTP file server built with Tauri 2 + Axum.<br>One-click start, auto-start on boot, tray, directory browse, file upload,<br>flexible config, ready to use.',
    currentVersion: 'Current Version',
    platform: 'Platform',
    gui: 'GUI Framework',
    backend: 'Backend Language',
    checkUpdate: 'Check for Updates',
    checking: 'Checking...',
    github: 'GitHub Repo',
    tauriDocs: 'Tauri Docs',
    latest: '✅ Already latest ({version})',
    available: '🎉 New version <strong>{latest}</strong> (current {current})<br><a href="{url}" target="_blank" onclick="event.preventDefault(); window.open(\'{url}\')">Download →</a>',
    noVersion: 'Version info not found',
    apiError: 'GitHub API returned {status}',
    updateError: '❌ Update check failed: {msg}<br>Please check your network and retry.'
  },
  toast: {
    pluginsLoaded: '🧩 Plugin config loaded',
    pluginsLoadFailed: 'Failed to load plugin config: {err}',
    pluginUpdating: '💾 Updating .{ext} status...',
    pluginToggleFailed: '❌ Toggle failed: {err}',
    refreshFailed: 'Refresh failed: {err}',
    started: '✅ HTTP service started',
    startFailed: '❌ {err}',
    stopped: '🛑 HTTP service stopped',
    stopFailed: '❌ {err}',
    restarting: '🔄 Restarting HTTP service...',
    restartFailed: '❌ Failed to restart HTTP service: {err}',
    configLoaded: '📥 Config loaded',
    configLoadFailed: 'Failed to load config: {err}',
    invalidPort: 'Please enter a valid port (1-65535)',
    emptyFolder: 'Please enter a public folder path',
    saved: '💾 Config saved',
    saveFailed: 'Save failed: {err}',
    pluginUpdated: '✅ Plugin status updated',
    folderSelected: '📁 Folder selected: {path}',
    browseFailed: 'Failed to open folder picker: {err}',
    autoStartEnabled: '✅ Auto-start enabled',
    autoStartDisabled: '❌ Auto-start disabled',
    operationFailed: 'Operation failed: {err}',
    minimized: '📦 Minimized to tray',
    hideFailed: 'Hide failed: {err}',
    quitConfirm: '⚠️ HTTP service is running, are you sure you want to quit?'
  },
  language: {
    switchTo: '中文',
    label: 'Language'
  }
}

export default en