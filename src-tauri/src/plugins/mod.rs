//! 插件配置管理模块
//!
//! 设计变更（去中心化）：
//! - 每个插件目录下自带 plugin.json 定义元信息
//! - 运行时通过扫描插件目录聚合出 PluginsConfig
//! - 用户偏好（激活选择、下载源偏好）存储在 plugins_state.json
//! - 全局 plugins.json 已废弃

pub mod types;
pub mod state;
pub mod scanner;
pub mod actions;

// ========== 重新导出公共 API ==========

pub use types::{
    DownloadSource, ExtensionConfig, ExtensionHandler, PluginMeta, PluginSource, PluginsConfig,
    PluginsState,
};

pub use state::{load_plugins_state, save_plugins_state, save_plugins_state_to_path};

pub use scanner::{
    apply_plugins_state, merge_configs, scan_and_build_config, scan_plugins_dir,
};