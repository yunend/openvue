//! 插件模块所有数据结构定义

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ========== 插件来源 ==========

/// 插件来源：内置（dist-web/plugins/）或用户自定义（可写插件目录）
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginSource {
    Builtin,
    Custom,
}

impl Default for PluginSource {
    fn default() -> Self {
        PluginSource::Builtin
    }
}

impl PluginSource {
    /// 用于生成 handler_id 的后缀
    pub fn to_handler_suffix(&self) -> &str {
        match self {
            PluginSource::Builtin => "builtin",
            PluginSource::Custom => "custom",
        }
    }
}

// ========== 运行时配置类型 ==========

/// 单个扩展名的一个处理器
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionHandler {
    pub handler_id: String,
    #[serde(rename = "pluginId", alias = "plugin_id", default)]
    pub plugin_id: Option<String>,
    #[serde(rename = "urlTemplate", alias = "url_template", default)]
    pub url_template: Option<String>,
    pub description: String,
    pub name: String,
    #[serde(default)]
    pub source: PluginSource,
}

/// 单个扩展名配置：多个备选处理器 + 当前激活的 handler_id
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionConfig {
    pub handlers: Vec<ExtensionHandler>,
    #[serde(default)]
    pub active_handler_id: Option<String>,
}

/// 运行时聚合的插件配置（由扫描 plugin.json 目录生成）
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PluginsConfig {
    pub extensions: BTreeMap<String, ExtensionConfig>,
}

// ========== plugin.json 元信息 ==========

/// 下载源定义
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSource {
    /// 标签，如 "GitHub Release" / "Gitee Release"
    pub label: String,
    /// 下载 URL
    pub url: String,
    /// 优先级（数值越小越优先）
    #[serde(default)]
    pub priority: u32,
}

/// 每个插件目录下的 plugin.json 结构
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PluginMeta {
    /// 插件唯一 ID（必须与所在目录名一致）
    pub id: String,
    pub name: String,
    /// 版本号
    #[serde(default)]
    pub version: String,
    /// 描述
    #[serde(default)]
    pub description: String,
    /// 支持的扩展名列表
    pub extensions: Vec<String>,
    /// URL 模板（{pluginId} 和 {publicPath} 占位符会被替换）
    pub url_template: String,
    /// 项目主页
    #[serde(default)]
    pub homepage: String,
    /// 多个下载源（每个源哈希相同，用户可选择偏好源）
    #[serde(default)]
    pub download_sources: Vec<DownloadSource>,
    /// ZIP 包的 SHA256
    #[serde(default)]
    pub sha256: String,
    /// 打包格式
    #[serde(default)]
    pub archive_format: String,
    /// 包大小（字节）
    #[serde(default)]
    pub size_bytes: u64,
}

// ========== 用户偏好状态 ==========

/// plugins_state.json 结构（只存用户偏好，不存插件元信息）
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginsState {
    /// 各扩展名用户选择的激活 handler_id
    #[serde(default)]
    pub active_handlers: BTreeMap<String, String>,
    /// 用户偏好的下载源标签
    #[serde(default)]
    pub preferred_download_source_label: Option<String>,
}

impl PluginsState {
    /// 获取指定扩展名的用户激活 handler_id
    pub fn get_active(&self, ext: &str) -> Option<&str> {
        self.active_handlers.get(ext).map(|s| s.as_str())
    }

    /// 记录指定扩展名的用户激活选择
    pub fn set_active(&mut self, ext: &str, handler_id: &str) {
        self.active_handlers
            .insert(ext.to_string(), handler_id.to_string());
    }

    /// 清除指定扩展名的激活选择（退化为浏览器默认）
    pub fn clear_active(&mut self, ext: &str) {
        self.active_handlers.remove(ext);
    }
}