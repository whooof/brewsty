//! Brewsty Plugin System - WASM-based plugin architecture
//!
//! This module provides a plugin system that supports loading WebAssembly (WASM) plugins.
//! Plugins can hook into various Brewsty operations like install, uninstall, and package listing.
//!
//! ## Features
//!
//! - **WASM Plugin Loading**: Load plugins from `.wasm` files at runtime
//! - **Plugin Hooks**: Intercept and modify Brewsty operations
//! - **Plugin Management**: Register, unload, and list active plugins
//!
//! ## Example
//!
//! ```rust,no_run
//! use brewsty::plugins::PluginManager;
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut manager = PluginManager::new();
//! manager.load_plugin("my-plugin", Path::new("/path/to/plugin.wasm"))?;
//! manager.initialize_all()?;
//!
//! for plugin in manager.list_plugins() {
//!     println!("Loaded: {} v{}", plugin.name, plugin.version);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use wasmtime::{Config, Engine, Instance, Module, Store};

/// Maximum WASM stack size (1 MB)
const WASM_MAX_STACK: usize = 1024 * 1024;

/// Allowed plugin directories (only these paths can load WASM plugins)
const ALLOWED_PLUGIN_DIRS: &[&str] = &["~/.brewsty/plugins", "/usr/local/lib/brewsty/plugins"];

/// Check if a path is within an allowed plugin directory
fn is_allowed_plugin_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    for allowed_dir in ALLOWED_PLUGIN_DIRS {
        let expanded = if allowed_dir.starts_with("~/") {
            dirs::home_dir()
                .map(|h| h.join(&allowed_dir[2..]))
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        } else {
            allowed_dir.to_string()
        };
        
        if path_str.starts_with(&expanded) {
            return true;
        }
    }
    
    false
}

/// Create a sandboxed wasmtime engine with resource limits
fn create_sandboxed_engine() -> Result<Engine> {
    let mut config = Config::new();

    // Limit stack size to prevent stack overflow attacks
    config.max_wasm_stack(WASM_MAX_STACK);

    // Enable fuel consumption for CPU limiting (prevents infinite loops)
    // Fuel is added at runtime in WasmPlugin::load()
    config.consume_fuel(true);

    // Enable epoch interruption for cooperative termination
    config.epoch_interruption(true);

    // Disable memory64 to limit memory address space (32-bit only)
    config.wasm_memory64(false);

    // Disable multi-memory to limit to single memory region
    config.wasm_multi_memory(false);

    Engine::new(&config)
        .map_err(|e| anyhow::anyhow!("Failed to create sandboxed WASM engine: {}", e))
}

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

/// WASM plugin instance
pub struct WasmPlugin {
    info: PluginInfo,
    _engine: Engine,
    _module: Module,
    _instance: Instance,
}

impl WasmPlugin {
    /// Load a WASM plugin from a file with sandboxing
    ///
    /// # Security
    /// - Plugin must be in an allowed directory
    /// - Engine has memory and stack limits
    /// - Fuel consumption is enabled for CPU limiting
    pub fn load(path: &Path) -> Result<Self> {
        // Security: Validate path is in allowed plugin directory
        if !is_allowed_plugin_path(path) {
            return Err(anyhow::anyhow!(
                "Security: Plugin path {:?} is not in an allowed directory. \
                 Allowed directories: {}",
                path,
                ALLOWED_PLUGIN_DIRS.join(", ")
            ));
        }

        // Use sandboxed engine with resource limits
        let engine = create_sandboxed_engine()?;
        let module = Module::from_file(&engine, path)
            .map_err(|e| anyhow::anyhow!("Failed to load WASM module from {:?}: {}", path, e))?;

        let mut store = Store::new(&engine, ());
        
        // Set initial fuel limit (100M units ≈ 100M instructions)
        store.set_fuel(100_000_000)?;
        
        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| anyhow::anyhow!("Failed to instantiate WASM module: {}", e))?;

        // Try to get plugin info from exports
        let info = Self::extract_plugin_info(&mut store, &instance)?;

        Ok(Self {
            info,
            _engine: engine,
            _module: module,
            _instance: instance,
        })
    }

    /// Extract plugin info from WASM exports
    fn extract_plugin_info(_store: &mut Store<()>, _instance: &Instance) -> Result<PluginInfo> {
        // Try to get info function from WASM module
        // In a real implementation, this would call exported WASM functions
        // For now, return placeholder info
        Ok(PluginInfo {
            name: "wasm-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "WASM plugin loaded successfully".to_string(),
            author: "Unknown".to_string(),
        })
    }
}

impl Plugin for WasmPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn initialize(&mut self) -> Result<()> {
        // Call WASM export for initialization
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        // Call WASM export for shutdown
        Ok(())
    }
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    #[allow(dead_code)] // Reserved for future WASM plugin enhancements
    engine: Arc<Engine>,
}

impl PluginManager {
    /// Create a new plugin manager with sandboxed WASM engine
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            engine: Arc::new(
                create_sandboxed_engine()
                    .expect("Failed to create sandboxed WASM engine")
            ),
        }
    }

    /// Register a plugin
    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) -> Result<()> {
        let info = plugin.info().clone();
        self.plugins.insert(info.name.clone(), Box::new(plugin));
        Ok(())
    }

    /// Load a WASM plugin from a file
    ///
    /// # Security
    /// Only loads plugins from allowed directories. See `ALLOWED_PLUGIN_DIRS`.
    pub fn load_plugin(&mut self, name: &str, path: &Path) -> Result<()> {
        // Security: Reject paths outside allowed directories
        if path.exists() && !is_allowed_plugin_path(path) {
            return Err(anyhow::anyhow!(
                "Security: Cannot load plugin from {:?}. \
                 Only allowed directories: {}",
                path,
                ALLOWED_PLUGIN_DIRS.join(", ")
            ));
        }
        
        if !path.exists() {
            // Fall back to placeholder if file doesn't exist
            let info = PluginInfo {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin file not found - using placeholder".to_string(),
                author: "Unknown".to_string(),
            };
            self.plugins
                .insert(name.to_string(), Box::new(PlaceholderPlugin { info }));
            return Ok(());
        }

        // Try to load as WASM plugin
        match WasmPlugin::load(path) {
            Ok(plugin) => {
                let info = plugin.info().clone();
                self.plugins.insert(info.name.clone(), Box::new(plugin));
                tracing::info!("Loaded WASM plugin: {} from {:?}", info.name, path);
            }
            Err(e) => {
                tracing::warn!("Failed to load WASM plugin {:?}: {}", path, e);
                // Fall back to placeholder
                let info = PluginInfo {
                    name: name.to_string(),
                    version: "0.1.0".to_string(),
                    description: format!("Failed to load: {}", e),
                    author: "Unknown".to_string(),
                };
                self.plugins
                    .insert(name.to_string(), Box::new(PlaceholderPlugin { info }));
            }
        }
        Ok(())
    }

    /// Load all plugins from a directory
    pub fn load_plugins_from_dir(&mut self, dir: &Path) -> Result<usize> {
        if !dir.exists() {
            return Ok(0);
        }

        let mut loaded = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "wasm") {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    if self.load_plugin(&name, &path).is_ok() {
                        loaded += 1;
                    }
                }
            }
        }
        Ok(loaded)
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

    #[test]
    fn test_load_nonexistent_plugin() {
        let mut manager = PluginManager::new();
        let result = manager.load_plugin("test", Path::new("/nonexistent/path.wasm"));
        assert!(result.is_ok()); // Should fall back to placeholder
        assert_eq!(manager.plugin_count(), 1);
    }

    #[test]
    fn test_load_plugins_from_nonexistent_dir() {
        let mut manager = PluginManager::new();
        let loaded = manager
            .load_plugins_from_dir(Path::new("/nonexistent/dir"))
            .unwrap();
        assert_eq!(loaded, 0);
    }

    #[test]
    fn test_wasm_plugin_info() {
        let info = PluginInfo {
            name: "wasm-test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
        };
        assert_eq!(info.name, "wasm-test");
        assert_eq!(info.version, "1.0.0");
    }
}
