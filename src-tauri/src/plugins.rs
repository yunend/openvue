//! 插件配置管理模块（plugins.json 加载 / 保存 + 扩展名映射查询）

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// plugins.json 默认路径
/// 委托给 paths::plugins_path()，自动处理 Windows / macOS / Linux 差异
pub fn get_default_plugins_path() -> Result<PathBuf, String> {
    crate::paths::plugins_path()
}

/// 插件来源：内置（dist-web/plugins/）或用户自定义（可写插件目录）
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginSource {
    Builtin,
    Custom,
}

impl Default for PluginSource {
    fn default() -> Self { PluginSource::Builtin }
}

/// 单个扩展名的一个处理器
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionHandler {
    /// 处理器唯一 ID（同一扩展名内不重复），如 "pdf-native" / "pdf-onlyoffice"
    pub handler_id: String,
    #[serde(rename = "pluginId", alias = "plugin_id", default)]
    pub plugin_id: Option<String>,
    #[serde(rename = "urlTemplate", alias = "url_template", default)]
    pub url_template: Option<String>,
    pub description: String,
    pub name: String,
    /// 插件来源：builtin=内置资源, custom=用户自定义
    #[serde(default)]
    pub source: PluginSource,
}

/// 单个扩展名配置：多个备选处理器 + 当前激活的 handler_id
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionConfig {
    pub handlers: Vec<ExtensionHandler>,
/// 当前激活的 handler_id；None 表示走 browser-default
    #[serde(default)]
    pub active_handler_id: Option<String>,
}

/// plugins.json 根结构
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PluginsConfig {
    pub extensions: BTreeMap<String, ExtensionConfig>,
}

/// 加载 plugins.json
///
/// ⚡ 首次启动时，如果用户可写目录下还没有 plugins.json：
///    优先从应用资源目录（只读）复制默认模板过去；
///    资源目录也没有模板时，才创建空骨架。
pub fn load_plugins_config(plugins_path: Option<&str>) -> Result<PluginsConfig, String> {
    let path = match plugins_path {
        Some(p) => PathBuf::from(p),
        None => get_default_plugins_path()?,
    };

    if !path.exists() {
        let template = crate::paths::default_plugins_path()?;
        if template.exists() {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("创建插件配置目录失败: {}", e))?;
                }
            }
            std::fs::copy(&template, &path).map_err(|e| {
                format!(
                    "从模板复制 plugins.json 失败: {} → {} ({})",
                    template.display(),
                    path.display(),
                    e
                )
            })?;
            println!("📋 plugins.json 首次启动，已从资源模板复制: {}", path.display());
        } else {
            println!("⚠️ plugins.json 不存在且无资源模板，使用默认空配置: {}", path.display());
            let default = PluginsConfig::default();
            save_plugins_config_to_path(&default, &path)?;
            return Ok(default);
        }
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取 plugins.json 失败: {}", e))?;

    let cfg: PluginsConfig = serde_json::from_str(&content)
        .map_err(|e| format!("解析 plugins.json 失败: {}", e))?;

    println!("✅ 插件配置加载成功，共 {} 个扩展名条目", cfg.extensions.len());
    Ok(cfg)
}

/// 保存配置到指定路径
pub fn save_plugins_config_to_path(cfg: &PluginsConfig, path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建插件配置目录失败: {}", e))?;
        }
    }
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("序列化 plugins.json 失败: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("写入 plugins.json 失败: {}", e))?;
    Ok(())
}

/// 保存到默认路径
pub fn save_plugins_config(cfg: &PluginsConfig) -> Result<(), String> {
    let path = get_default_plugins_path()?;
    save_plugins_config_to_path(cfg, &path)
}


impl PluginsConfig {
    /// 激活指定 handler（只需设置 active_handler_id，null = 浏览器默认）
    pub fn activate_handler(
        &mut self,
        ext: &str,
        handler_id: &str,
    ) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let config = self
            .extensions
            .get_mut(&ext_key)
            .ok_or_else(|| format!("扩展名 .{} 不存在", ext_key))?;

        // 校验 handler 是否存在
        if !config.handlers.iter().any(|h| h.handler_id == handler_id) {
            return Err(format!(
                "扩展名 .{} 下找不到处理器 id={}",
                ext_key, handler_id
            ));
        }

        config.active_handler_id = Some(handler_id.to_string());
        Ok(())
    }

    /// 切回浏览器默认（active_handler_id 设为 None）
    pub fn set_browser_default(
        &mut self,
        ext: &str,
    ) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let config = self
            .extensions
            .get_mut(&ext_key)
            .ok_or_else(|| format!("扩展名 .{} 不存在", ext_key))?;
        config.active_handler_id = None;
        Ok(())
    }

    /// 添加自定义插件处理器（folder_name 作为 handlerId/pluginId）
    /// url_template 使用 "/pfolder/" 前缀（与内置的 "/plugins/" 相异）
    pub fn add_custom_handler(
        &mut self,
        ext: &str,
        folder_name: &str,
    ) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let handler_id = folder_name.to_string();
        let plugin_id = folder_name.to_string();
        // ⚠️ 关键：自定义插件用 /pfolder/ 前缀，与内置 /plugins/ 相异
        let url_template = "/pfolder/{pluginId}/?path={publicPath}".to_string();
        let name = format!("自定义{}", ext_key);

        let new_handler = ExtensionHandler {
            handler_id: handler_id.clone(),
            plugin_id: Some(plugin_id),
            url_template: Some(url_template),
            description: String::new(),
            name,
            source: PluginSource::Custom,
        };

        let config = self.extensions.entry(ext_key.clone()).or_insert_with(|| ExtensionConfig {
            handlers: vec![],
            active_handler_id: None,
        });

        config.handlers.push(new_handler);
        config.active_handler_id = Some(handler_id);

        Ok(())
    }
}