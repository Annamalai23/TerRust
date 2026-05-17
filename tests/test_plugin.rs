use terrust::plugins::PluginManager;
use tempfile::tempdir;
use std::path::PathBuf;
use std::fs;

#[test]
fn test_plugin_manager_creation() {
    let manager = PluginManager::new(PathBuf::from("/tmp/plugins"));
    let plugins = manager.list_plugins().unwrap();
    assert!(plugins.is_empty());
}

#[test]
fn test_plugin_manager_empty_dir() {
    let dir = tempdir().unwrap();
    let manager = PluginManager::new(dir.path().to_path_buf());
    let plugins = manager.list_plugins().unwrap();
    assert!(plugins.is_empty());
}

#[test]
fn test_plugin_manager_nonexistent_dir() {
    let manager = PluginManager::new(PathBuf::from("/nonexistent/plugins"));
    let plugins = manager.list_plugins().unwrap();
    assert!(plugins.is_empty());
}

#[test]
fn test_plugin_discovery_from_toml() {
    let dir = tempdir().unwrap();
    let plugin_dir = dir.path().join("my_plugin");
    fs::create_dir(&plugin_dir).unwrap();

    let toml = r#"
name = "my_plugin"
version = "1.0.0"
description = "A test plugin"
"#;
    fs::write(plugin_dir.join("plugin.toml"), toml).unwrap();

    let manager = PluginManager::new(dir.path().to_path_buf());
    let plugins = manager.list_plugins().unwrap();
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].name, "my_plugin");
    assert_eq!(plugins[0].version, "1.0.0");
}

#[test]
fn test_plugin_discovery_multiple_plugins() {
    let dir = tempdir().unwrap();

    for i in 0..3 {
        let plugin_dir = dir.path().join(format!("plugin_{}", i));
        fs::create_dir(&plugin_dir).unwrap();
        let toml = format!(
            "name = \"plugin_{}\"\nversion = \"0.{}.0\"\ndescription = \"Plugin {}\"\n",
            i, i, i
        );
        fs::write(plugin_dir.join("plugin.toml"), toml).unwrap();
    }

    let manager = PluginManager::new(dir.path().to_path_buf());
    let plugins = manager.list_plugins().unwrap();
    assert_eq!(plugins.len(), 3);
}

#[test]
fn test_plugin_skip_dir_without_toml() {
    let dir = tempdir().unwrap();
    let plugin_dir = dir.path().join("empty_plugin");
    fs::create_dir(&plugin_dir).unwrap();

    let manager = PluginManager::new(dir.path().to_path_buf());
    let plugins = manager.list_plugins().unwrap();
    assert!(plugins.is_empty());
}

#[test]
fn test_plugin_manager_not_a_directory() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("not_a_dir");
    fs::write(&file_path, "content").unwrap();

    let manager = PluginManager::new(file_path);
    let result = manager.list_plugins();
    assert!(result.is_err());
}

#[test]
fn test_plugin_info_fields() {
    use terrust::plugins::PluginInfo;

    let info = PluginInfo {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        description: "desc".to_string(),
    };
    assert_eq!(info.name, "test");
    assert_eq!(info.version, "0.1.0");
}
