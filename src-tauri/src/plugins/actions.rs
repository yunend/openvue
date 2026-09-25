//! PluginsConfig 操作方法：激活/切换默认/添加/删除自定义插件

use super::types::{ExtensionConfig, ExtensionHandler, PluginSource, PluginsConfig};

impl PluginsConfig {
    /// 激活指定 handler
    pub fn activate_handler(&mut self, ext: &str, handler_id: &str) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let config = self
            .extensions
            .get_mut(&ext_key)
            .ok_or_else(|| format!("扩展名 .{} 不存在", ext_key))?;
        if !config.handlers.iter().any(|h| h.handler_id == handler_id) {
            return Err(format!(
                "扩展名 .{} 下找不到处理器 id={}",
                ext_key, handler_id
            ));
        }
        config.active_handler_id = Some(handler_id.to_string());
        Ok(())
    }

    /// 切回浏览器默认
    pub fn set_browser_default(&mut self, ext: &str) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let config = self
            .extensions
            .get_mut(&ext_key)
            .ok_or_else(|| format!("扩展名 .{} 不存在", ext_key))?;
        config.active_handler_id = None;
        Ok(())
    }

    /// 添加自定义插件处理器
    pub fn add_custom_handler(&mut self, ext: &str, folder_name: &str) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let handler_id = folder_name.to_string();
        let plugin_id = folder_name.to_string();
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

        let config = self.extensions.entry(ext_key.clone()).or_insert_with(|| {
            ExtensionConfig {
                handlers: vec![],
                active_handler_id: None,
            }
        });
        config.handlers.push(new_handler);
        config.active_handler_id = Some(handler_id);
        Ok(())
    }

    /// 删除自定义插件处理器
    pub fn remove_custom_handler(&mut self, ext: &str, folder_name: &str) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let handler_id = folder_name.to_string();
        let config = self
            .extensions
            .get_mut(&ext_key)
            .ok_or_else(|| format!("扩展名 .{} 没有任何插件处理器", ext_key))?;

        let idx = config.handlers.iter().position(|h| h.handler_id == handler_id);
        match idx {
            Some(i) => {
                let handler = &config.handlers[i];
                if handler.source != PluginSource::Custom {
                    return Err(format!(
                        "❌ 内置插件不可删除（.{} / {}）",
                        ext_key, handler_id
                    ));
                }
                config.handlers.remove(i);
                if config.active_handler_id.as_deref() == Some(&handler_id) {
                    config.active_handler_id = None;
                }
                if config.handlers.is_empty() {
                    self.extensions.remove(&ext_key);
                }
                Ok(())
            }
            None => Err(format!("自定义插件不存在：.{} / {}", ext_key, handler_id)),
        }
    }
}