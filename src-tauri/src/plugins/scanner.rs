//! 插件目录扫描、元信息读取、配置聚合

use std::path::Path;

use super::types::{
    ExtensionConfig, ExtensionHandler, PluginMeta, PluginSource, PluginsConfig, PluginsState,
};

/// 读取单个插件目录下的 plugin.json，返回 PluginMeta
/// 如果目录下没有 plugin.json 或解析失败，返回 None 并打印警告
fn read_plugin_meta(plugin_dir: &Path, source: PluginSource) -> Option<PluginMeta> {
    let plugin_json_path = plugin_dir.join("plugin.json");
    if !plugin_json_path.exists() {
        return None;
    }
    let content = match std::fs::read_to_string(&plugin_json_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️ 读取 {} 失败: {}", plugin_json_path.display(), e);
            return None;
        }
    };
    let meta: PluginMeta = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("⚠️ 解析 {} 失败: {}，跳过", plugin_json_path.display(), e);
            return None;
        }
    };

    // 校验 id 必须与目录名一致
    let dir_name = plugin_dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    if meta.id != dir_name {
        eprintln!(
            "⚠️ plugin.json 中 id=\"{}\" 与目录名 \"{}\" 不一致，跳过",
            meta.id, dir_name
        );
        return None;
    }

    Some(meta)
}

/// 扫描一个目录（如 builtin 或 user 插件目录），返回聚合后的 PluginsConfig
pub fn scan_plugins_dir(plugins_root: &Path, source: PluginSource) -> PluginsConfig {
    let mut config = PluginsConfig::default();

    if !plugins_root.exists() || !plugins_root.is_dir() {
        return config;
    }

    let entries = match std::fs::read_dir(plugins_root) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("⚠️ 读取插件目录 {} 失败: {}", plugins_root.display(), e);
            return config;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if dir_name.starts_with('.') {
            continue;
        }

        let meta = match read_plugin_meta(&path, source.clone()) {
            Some(m) => m,
            None => continue,
        };

        let handler_id = format!("{}-{}", meta.id, source.to_handler_suffix());
        let plugin_id = meta.id.clone();
        let source_for_handler = source.clone();

        let handler = ExtensionHandler {
            handler_id: handler_id.clone(),
            plugin_id: Some(plugin_id),
            url_template: Some(meta.url_template),
            description: meta.description.clone(),
            name: meta.name,
            source: source_for_handler,
        };

        for ext in &meta.extensions {
            let ext_lower = ext.to_lowercase();
            let entry = config.extensions.entry(ext_lower).or_insert_with(|| {
                ExtensionConfig {
                    handlers: vec![],
                    active_handler_id: None,
                }
            });
            entry.handlers.push(handler.clone());
        }

        println!(
            "   📦 插件 [{}] {} ({} 个扩展名)",
            meta.id,
            path.display(),
            meta.extensions.len()
        );
    }

    config
}

/// 合并两个 PluginsConfig（后者覆盖前者，用于 内置+用户 叠加）
pub fn merge_configs(base: PluginsConfig, overlay: PluginsConfig) -> PluginsConfig {
    let mut result = base;
    for (ext, ext_cfg) in overlay.extensions {
        let entry = result.extensions.entry(ext).or_insert_with(|| {
            ExtensionConfig {
                handlers: vec![],
                active_handler_id: None,
            }
        });
        entry.handlers.extend(ext_cfg.handlers);
    }
    result
}

/// 应用用户偏好状态到运行时配置
pub fn apply_plugins_state(config: &mut PluginsConfig, state: &PluginsState) {
    for (ext, handler_id) in &state.active_handlers {
        if let Some(ext_cfg) = config.extensions.get_mut(ext) {
            if ext_cfg.handlers.iter().any(|h| h.handler_id == *handler_id) {
                ext_cfg.active_handler_id = Some(handler_id.clone());
            }
        }
    }
}

/// 完整扫描流程：内置插件目录 + 用户插件目录 + 内置兜底 + 应用状态
pub fn scan_and_build_config(
    builtin_dir: &Path,
    user_dir: &Path,
    state: &PluginsState,
) -> PluginsConfig {
    println!("🔍 开始扫描插件目录...");
    println!("   📁 内置插件: {}", builtin_dir.display());
    println!("   📁 用户插件: {}", user_dir.display());

    let mut config = scan_plugins_dir(builtin_dir, PluginSource::Builtin);

    let user_config = scan_plugins_dir(user_dir, PluginSource::Custom);
    config = merge_configs(config, user_config);

    apply_plugins_state(&mut config, state);

    println!(
        "✅ 插件扫描完成，共 {} 个扩展名条目",
        config.extensions.len()
    );
    config
}