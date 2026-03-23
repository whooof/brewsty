//! Brewsty Plugin System - Plugin architecture (simplified version)

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// Plugin trait that all plugins must implement
pub trait Plugin {
    fn info(&self) -> &PluginInfo;
    fn initialize(&mut self) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) -> Result<()> {
        let info = plugin.info().clone();
        self.plugins.insert(info.name.clone(), Box::new(plugin));
        Ok(())
    }

    /// Load a plugin from a file (placeholder for WASM loading)
    pub fn load_plugin(&mut self, name: &str, _path: &Path) -> Result<()> {
        // Placeholder - in real implementation this would load WASM
        let info = PluginInfo {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: "Loaded plugin".to_string(),
            author: "Unknown".to_string(),
        };

        // For now, just register a placeholder
        self.plugins
            .insert(name.to_string(), Box::new(PlaceholderPlugin { info }));
        Ok(())
    }

    /// Get a loaded plugin
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins.values().map(|p| p.info()).collect()
    }

    /// Unload a plugin
    pub fn unload_plugin(&mut self, name: &str) -> bool {
        self.plugins.remove(name).is_some()
    }

    /// Get the number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Initialize all plugins
    pub fn initialize_all(&mut self) -> Result<()> {
        for plugin in self.plugins.values_mut() {
            plugin.initialize()?;
        }
        Ok(())
    }

    /// Shutdown all plugins
    pub fn shutdown_all(&mut self) -> Result<()> {
        for plugin in self.plugins.values_mut() {
            plugin.shutdown()?;
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder plugin for testing
struct PlaceholderPlugin {
    info: PluginInfo,
}

impl Plugin for PlaceholderPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Plugin hook types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginHook {
    /// Called when a package is about to be installed
    PreInstall,
    /// Called after a package is installed
    PostInstall,
    /// Called when a package is about to be uninstalled
    PreUninstall,
    /// Called after a package is uninstalled
    PostUninstall,
    /// Called when listing packages
    ListPackages,
    /// Called when searching for packages
    SearchPackages,
}

impl PluginHook {
    pub fn name(&self) -> &'static str {
        match self {
            PluginHook::PreInstall => "pre_install",
            PluginHook::PostInstall => "post_install",
            PluginHook::PreUninstall => "pre_uninstall",
            PluginHook::PostUninstall => "post_uninstall",
            PluginHook::ListPackages => "list_packages",
            PluginHook::SearchPackages => "search_packages",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.plugin_count(), 0);
    }

    #[test]
    fn test_plugin_hook_names() {
        assert_eq!(PluginHook::PreInstall.name(), "pre_install");
        assert_eq!(PluginHook::PostInstall.name(), "post_install");
        assert_eq!(PluginHook::ListPackages.name(), "list_packages");
    }

    #[test]
    fn test_plugin_manager_empty() {
        let manager = PluginManager::new();
        assert!(manager.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_lifecycle() {
        let mut manager = PluginManager::new();

        let info = PluginInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test".to_string(),
        };

        let plugin = PlaceholderPlugin { info };
        manager.register(plugin).unwrap();

        assert_eq!(manager.plugin_count(), 1);
        assert!(manager.get_plugin("test").is_some());

        manager.initialize_all().unwrap();
        manager.shutdown_all().unwrap();

        assert!(manager.unload_plugin("test"));
        assert_eq!(manager.plugin_count(), 0);
    }
}
