//! Hello World Plugin
//!
//! A simple example plugin demonstrating the RustPress plugin system.
//! Shows how to:
//! - Register a plugin
//! - Add shortcodes
//! - Add widgets
//! - Use hooks (actions and filters)
//! - Store plugin settings

use async_trait::async_trait;
use chrono::Utc;
use rustpress_core::context::AppContext;
use rustpress_core::error::Result;
use rustpress_core::hook::HookRegistry;
use rustpress_core::plugin::{Plugin, PluginInfo, PluginState};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Plugin settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloWorldSettings {
    pub greeting_text: String,
    pub show_date: bool,
    pub custom_css: String,
}

impl Default for HelloWorldSettings {
    fn default() -> Self {
        Self {
            greeting_text: "Hello, World!".to_string(),
            show_date: true,
            custom_css: String::new(),
        }
    }
}

/// The Hello World plugin
pub struct HelloWorldPlugin {
    info: PluginInfo,
    state: RwLock<PluginState>,
    settings: RwLock<HelloWorldSettings>,
}

impl HelloWorldPlugin {
    /// Create a new instance of the plugin
    pub fn new() -> Self {
        let info = PluginInfo::new(
            "hello-world",
            "Hello World",
            Version::new(1, 0, 0),
        )
        .with_description("A simple example plugin that adds a greeting shortcode and widget")
        .with_author("RustPress Team");

        Self {
            info,
            state: RwLock::new(PluginState::Inactive),
            settings: RwLock::new(HelloWorldSettings::default()),
        }
    }

    /// Get current settings
    pub fn settings(&self) -> HelloWorldSettings {
        self.settings.read().clone()
    }

    /// Update settings
    pub fn update_settings(&self, settings: HelloWorldSettings) {
        *self.settings.write() = settings;
    }

    /// Register shortcodes
    fn register_shortcodes(&self, hooks: &HookRegistry) {
        // [hello] shortcode
        let settings = self.settings.read().clone();
        hooks.add_filter("shortcode_hello", move |_content: String| {
            let mut output = format!(
                r#"<div class="hello-world-greeting">{}</div>"#,
                settings.greeting_text
            );

            if settings.show_date {
                output.push_str(&format!(
                    r#"<div class="hello-world-date">Today is {}</div>"#,
                    Utc::now().format("%B %d, %Y")
                ));
            }

            output
        }, 10);
    }

    /// Register widgets
    fn register_widgets(&self, hooks: &HookRegistry) {
        // Hello World widget
        let settings = self.settings.read().clone();
        hooks.add_filter("widget_hello_world", move |_content: String| {
            format!(
                r#"<div class="widget hello-world-widget">
                    <h3 class="widget-title">Greeting</h3>
                    <div class="widget-content">
                        <p>{}</p>
                    </div>
                </div>"#,
                settings.greeting_text
            )
        }, 10);
    }

    /// Add custom CSS to head
    fn add_head_css(&self, hooks: &HookRegistry) {
        let settings = self.settings.read().clone();
        hooks.add_action("wp_head", move || {
            let css = if settings.custom_css.is_empty() {
                r#"
                .hello-world-greeting {
                    padding: 20px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                    border-radius: 8px;
                    margin: 20px 0;
                    font-size: 1.5em;
                    text-align: center;
                }
                .hello-world-date {
                    text-align: center;
                    color: #666;
                    font-style: italic;
                }
                .hello-world-widget {
                    background: #f5f5f5;
                    padding: 15px;
                    border-radius: 4px;
                }
                "#.to_string()
            } else {
                settings.custom_css.clone()
            };

            println!("<style>{}</style>", css);
        }, 10);
    }

    /// Content filter example
    fn add_content_filter(&self, hooks: &HookRegistry) {
        hooks.add_filter("the_content", |content: String| {
            // Add a small footer to all content
            format!(
                r#"{}
                <div class="hello-world-footer" style="font-size: 0.8em; color: #999; margin-top: 20px; padding-top: 10px; border-top: 1px solid #eee;">
                    Powered by Hello World Plugin
                </div>"#,
                content
            )
        }, 99); // Low priority to run last
    }
}

impl Default for HelloWorldPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for HelloWorldPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn state(&self) -> PluginState {
        *self.state.read()
    }

    async fn activate(&self, ctx: &AppContext) -> Result<()> {
        tracing::info!("Activating Hello World plugin");

        // Load settings from database (if available)
        // For now, use defaults

        // Register with hook system
        if let Some(hooks) = ctx.get::<Arc<RwLock<HookRegistry>>>() {
            let registry = hooks.read();
            self.register_shortcodes(&registry);
            self.register_widgets(&registry);
            self.add_head_css(&registry);
            self.add_content_filter(&registry);
        }

        *self.state.write() = PluginState::Active;
        tracing::info!("Hello World plugin activated successfully");

        Ok(())
    }

    async fn deactivate(&self, _ctx: &AppContext) -> Result<()> {
        tracing::info!("Deactivating Hello World plugin");

        // Clean up hooks would happen here
        // In a real implementation, we'd remove our registered hooks

        *self.state.write() = PluginState::Inactive;
        tracing::info!("Hello World plugin deactivated");

        Ok(())
    }

    async fn on_startup(&self, _ctx: &AppContext) -> Result<()> {
        tracing::debug!("Hello World plugin startup");
        Ok(())
    }

    async fn on_shutdown(&self, _ctx: &AppContext) -> Result<()> {
        tracing::debug!("Hello World plugin shutdown");
        Ok(())
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "greeting_text": {
                    "type": "string",
                    "title": "Greeting Text",
                    "description": "The text to display in the greeting",
                    "default": "Hello, World!"
                },
                "show_date": {
                    "type": "boolean",
                    "title": "Show Date",
                    "description": "Whether to show the current date",
                    "default": true
                },
                "custom_css": {
                    "type": "string",
                    "title": "Custom CSS",
                    "description": "Custom CSS styles for the plugin",
                    "default": ""
                }
            }
        }))
    }
}

/// Plugin entry point - called by the plugin loader
pub fn create_plugin() -> Arc<dyn Plugin> {
    Arc::new(HelloWorldPlugin::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = HelloWorldPlugin::new();
        assert_eq!(plugin.info().id, "hello-world");
        assert_eq!(plugin.info().name, "Hello World");
    }

    #[test]
    fn test_settings() {
        let plugin = HelloWorldPlugin::new();
        let settings = plugin.settings();
        assert_eq!(settings.greeting_text, "Hello, World!");
        assert!(settings.show_date);
    }

    #[test]
    fn test_update_settings() {
        let plugin = HelloWorldPlugin::new();
        plugin.update_settings(HelloWorldSettings {
            greeting_text: "Howdy!".to_string(),
            show_date: false,
            custom_css: ".test { color: red; }".to_string(),
        });

        let settings = plugin.settings();
        assert_eq!(settings.greeting_text, "Howdy!");
        assert!(!settings.show_date);
    }
}
