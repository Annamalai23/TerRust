use terrust::config::*;
use tempfile::tempdir;

fn expected_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
}

#[test]
fn test_config_default_path_resolution() {
    let config = Config::default();
    assert_eq!(config.general.theme, "tokyo-night");
    assert_eq!(config.terminal.shell, expected_shell());
}

#[test]
fn test_config_serde_roundtrip() {
    let config = Config::default();
    let toml_str = toml::to_string(&config).expect("serialize");
    let deserialized: Config = toml::from_str(&toml_str).expect("deserialize");
    assert_eq!(deserialized.general.theme, config.general.theme);
    assert_eq!(deserialized.terminal.shell, config.terminal.shell);
    assert_eq!(deserialized.ai.default_provider, config.ai.default_provider);
}

#[test]
fn test_config_file_save_and_load() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("terrust.toml");

    let config = Config::default();
    let toml_str = toml::to_string(&config).unwrap();
    std::fs::write(&path, &toml_str).unwrap();

    let loaded = Config::from_file(&path).unwrap();
    assert_eq!(loaded.general.theme, "tokyo-night");
}

#[test]
fn test_theme_presets_have_distinct_colors() {
    let tokyo = ThemeConfig::tokyo_night();
    let catppuccin = ThemeConfig::catppuccin_mocha();
    let dracula = ThemeConfig::dracula();

    assert_ne!(tokyo.background, catppuccin.background);
    assert_ne!(tokyo.foreground, dracula.foreground);
    assert!(tokyo.ansi.len() >= 8);
    assert!(catppuccin.ansi.len() >= 8);
    assert!(dracula.ansi.len() >= 8);
}

#[test]
fn test_theme_color_parsing() {
    let theme = ThemeConfig::tokyo_night();
    let bg = theme.bg();
    let fg = theme.fg();
    assert_ne!(bg, fg);
}

#[test]
fn test_shell_config_conversion() {
    let terminal_cfg = TerminalConfig::default();
    let shell: terrust::terminal::ShellConfig = terminal_cfg.into();
    assert_eq!(shell.shell, expected_shell());
}

#[test]
fn test_keybinding_defaults_have_essential_actions() {
    let kb = KeybindingsConfig::default();
    assert!(kb.all_bindings().len() > 10);
}

#[test]
fn test_plugins_config_defaults() {
    let plugins = PluginsConfig::default();
    assert!(plugins.auto_update);
    assert_eq!(plugins.enabled, vec!["git_helper"]);
}

#[test]
fn test_ai_config_disable_toggle() {
    let config = AIConfig {
        enabled: false,
        ..AIConfig::default()
    };
    assert!(!config.enabled);
}

#[test]
fn test_telemetry_config_default() {
    let telemetry = TelemetryConfig::default();
    assert!(!telemetry.enabled);
}

#[test]
fn test_config_override_via_fields() {
    let mut config = Config::default();
    config.terminal.shell = "/bin/zsh".to_string();
    config.general.theme = "dracula".to_string();
    assert_eq!(config.terminal.shell, "/bin/zsh");
    assert_eq!(config.general.theme, "dracula");
}
