//! Plugin system for TerRust
//!
//! Dynamic plugin loading and management for extending TerRust functionality

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Information about a loaded plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
}

/// Manages loading and listing of plugins
pub struct PluginManager {
    plugin_dir: PathBuf,
}

impl PluginManager {
    /// Create a new PluginManager with the specified plugin directory
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self { plugin_dir }
    }

    /// List all available plugins in the plugin directory
    ///
    /// Returns a vector of PluginInfo for each plugin found.
    /// Plugins can be either:
    /// - Dynamic libraries (.so on Unix, .dll on Windows)
    /// - Directories with a plugin.toml metadata file
    pub fn list_plugins(&self) -> Result<Vec<PluginInfo>> {
        let mut plugins = Vec::new();

        // Try to read the plugin directory
        if !self.plugin_dir.exists() {
            // Return empty list if directory doesn't exist
            return Ok(plugins);
        }

        if !self.plugin_dir.is_dir() {
            return Err(anyhow::anyhow!(
                "Plugin path {} is not a directory",
                self.plugin_dir.display()
            ));
        }

        // Read directory entries
        let entries = fs::read_dir(&self.plugin_dir)
            .with_context(|| format!("Failed to read plugin directory: {}", self.plugin_dir.display()))?;

        // Sort entries for consistent ordering
        let mut sorted_entries: Vec<_> = entries.collect::<Result<Vec<_>, _>>()?;
        sorted_entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for entry in sorted_entries {
            let path = entry.path();

            // Check if it's a dynamic library
            if self.is_dynamic_library(&path) {
                if let Some(plugin) = self.load_plugin_from_library(&path)? {
                    plugins.push(plugin);
                }
                continue;
            }

            // Check if it's a directory with plugin.toml
            if path.is_dir() {
                if let Some(plugin) = self.load_plugin_from_directory(&path)? {
                    plugins.push(plugin);
                }
            }
        }

        Ok(plugins)
    }

    /// Check if a path is a dynamic library
    fn is_dynamic_library(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| {
                let ext_str = ext.to_string_lossy();
                ext_str == "so" || ext_str == "dll" || ext_str == "dylib"
            })
            .unwrap_or(false)
    }

    /// Try to load plugin info from a dynamic library
    #[cfg(feature = "plugins")]
    fn load_plugin_from_library(&self, path: &Path) -> Result<Option<PluginInfo>> {
        use libloading::Library;

        // Try to load the library
        match unsafe { Library::new(path) } {
            Ok(_library) => {
                // Try to get plugin info function
                // Convention: plugins should export a `plugin_info` function
                // that returns a pointer to PluginInfo
                // For now, extract from filename
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();

                Ok(Some(PluginInfo {
                    name,
                    version: "1.0.0".to_string(),
                    description: "Dynamic plugin".to_string(),
                }))
            }
            Err(_) => Ok(None), // Skip libraries that can't be loaded
        }
    }

    /// Try to load plugin info from a directory with plugin.toml
    fn load_plugin_from_directory(&self, path: &Path) -> Result<Option<PluginInfo>> {
        let toml_path = path.join("plugin.toml");
        if !toml_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&toml_path)
            .with_context(|| format!("Failed to read {}", toml_path.display()))?;

        #[derive(serde::Deserialize)]
        struct PluginToml {
            name: Option<String>,
            version: Option<String>,
            description: Option<String>,
        }

        let plugin_toml: PluginToml = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", toml_path.display()))?;

        Ok(Some(PluginInfo {
            name: plugin_toml.name.unwrap_or_else(|| "unknown".to_string()),
            version: plugin_toml.version.unwrap_or_else(|| "1.0.0".to_string()),
            description: plugin_toml
                .description
                .unwrap_or_else(|| "No description".to_string()),
        }))
    }

    /// Default implementation for when plugins feature is disabled
    #[cfg(not(feature = "plugins"))]
    fn load_plugin_from_library(&self, _path: &Path) -> Result<Option<PluginInfo>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_list_plugins_empty_dir() {
        let temp_dir = tempdir().unwrap();
        let plugin_manager = PluginManager::new(temp_dir.path().to_path_buf());
        let plugins = plugin_manager.list_plugins().unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_list_plugins_from_toml() {
        let temp_dir = tempdir().unwrap();
        let plugin_dir = temp_dir.path().join("test_plugin");
        fs::create_dir(&plugin_dir).unwrap();

        let toml_content = r#"
name = "test_plugin"
version = "0.1.0"
description = "A test plugin for TerRust"
"#;
        fs::write(plugin_dir.join("plugin.toml"), toml_content).unwrap();

        let plugin_manager = PluginManager::new(temp_dir.path().to_path_buf());
        let plugins = plugin_manager.list_plugins().unwrap();

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "test_plugin");
        assert_eq!(plugins[0].version, "0.1.0");
        assert_eq!(plugins[0].description, "A test plugin for TerRust");
    }

    #[test]
    fn test_list_plugins_nonexistent_dir() {
        let plugin_manager = PluginManager::new(PathBuf::from("/nonexistent/path"));
        let plugins = plugin_manager.list_plugins().unwrap();
        assert!(plugins.is_empty());
    }
}
