//! Configuration management for TerRust
//!
//! Handles loading, parsing, and managing application configuration from
//! TOML files. Supports default paths and custom configuration locations.

use crate::terminal::ShellConfig;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub mod keybindings;
pub mod theme;

pub use keybindings::KeybindingsConfig;
pub use theme::ThemeConfig;

/// Main configuration structure for TerRust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General application settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// Terminal-specific settings
    #[serde(default)]
    pub terminal: TerminalConfig,

    /// AI-related settings
    #[serde(default)]
    pub ai: AIConfig,

    /// Plugin settings
    #[serde(default)]
    pub plugins: PluginsConfig,

    /// UI theme settings
    #[serde(default = "default_theme")]
    pub theme: ThemeConfig,

    /// Keybindings configuration
    #[serde(default)]
    pub keybindings: KeybindingsConfig,

    /// Telemetry settings
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            terminal: TerminalConfig::default(),
            ai: AIConfig::default(),
            plugins: PluginsConfig::default(),
            theme: default_theme(),
            keybindings: KeybindingsConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

fn default_theme() -> ThemeConfig {
    ThemeConfig::tokyo_night()
}

/// General application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Application theme name
    #[serde(default = "default_theme_name")]
    pub theme: String,

    /// Font family for the terminal
    #[serde(default = "default_font")]
    pub font: String,

    /// Window opacity (0.0 - 1.0)
    #[serde(default = "default_opacity")]
    pub opacity: f32,

    /// Maximum number of lines in scrollback buffer
    #[serde(default = "default_scrollback_limit")]
    pub scrollback_limit: usize,

    /// Commands to execute on startup
    #[serde(default)]
    pub startup_commands: Vec<String>,

    /// Default working directory
    #[serde(default = "default_working_dir")]
    pub default_working_dir: PathBuf,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            theme: default_theme_name(),
            font: default_font(),
            opacity: default_opacity(),
            scrollback_limit: default_scrollback_limit(),
            startup_commands: default_startup_commands(),
            default_working_dir: default_working_dir(),
        }
    }
}

fn default_theme_name() -> String {
    "tokyo-night".to_string()
}

fn default_font() -> String {
    "FiraCode Nerd Font Mono 14".to_string()
}

fn default_opacity() -> f32 {
    0.95
}

fn default_scrollback_limit() -> usize {
    10000
}

fn default_startup_commands() -> Vec<String> {
    vec![]
}

fn default_working_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
}

/// Terminal-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Path to the shell executable
    #[serde(default = "default_shell")]
    pub shell: String,

    /// Shell arguments
    #[serde(default)]
    pub shell_args: Vec<String>,

    /// Cursor style (block, beam, underline)
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,

    /// Whether the cursor should blink
    #[serde(default = "default_cursor_blink")]
    pub cursor_blink: bool,

    /// Whether to play a bell sound on notifications
    #[serde(default = "default_bell")]
    pub bell_notification: bool,

    /// Enable mouse support
    #[serde(default = "default_mouse_support")]
    pub mouse_support: bool,

    /// Enable true color (24-bit) support
    #[serde(default = "default_true_color")]
    pub true_color: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: default_shell(),
            shell_args: default_shell_args(),
            cursor_style: default_cursor_style(),
            cursor_blink: default_cursor_blink(),
            bell_notification: default_bell(),
            mouse_support: default_mouse_support(),
            true_color: default_true_color(),
        }
    }
}

impl From<TerminalConfig> for ShellConfig {
    fn from(config: TerminalConfig) -> Self {
        ShellConfig {
            shell: config.shell,
            args: config.shell_args,
        }
    }
}

fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
}

fn default_shell_args() -> Vec<String> {
    vec![]
}

fn default_cursor_style() -> String {
    "block".to_string()
}

fn default_cursor_blink() -> bool {
    true
}

fn default_bell() -> bool {
    true
}

fn default_mouse_support() -> bool {
    true
}

fn default_true_color() -> bool {
    true
}

/// AI-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// Default AI provider to use
    #[serde(default = "default_ai_provider")]
    pub default_provider: String,

    /// Timeout for AI requests (in seconds)
    #[serde(default = "default_ai_timeout")]
    pub timeout: u64,

    /// Maximum context tokens for AI responses
    #[serde(default = "default_max_context_tokens")]
    pub max_context_tokens: usize,

    /// Whether to show typing indicator for AI responses
    #[serde(default = "default_show_typing")]
    pub show_typing_indicator: bool,

    /// Whether to enable AI features
    #[serde(default = "default_ai_enabled")]
    pub enabled: bool,

    /// AI provider configurations
    #[serde(default)]
    pub providers: AIProvidersConfig,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            default_provider: default_ai_provider(),
            timeout: default_ai_timeout(),
            max_context_tokens: default_max_context_tokens(),
            show_typing_indicator: default_show_typing(),
            enabled: default_ai_enabled(),
            providers: AIProvidersConfig::default(),
        }
    }
}

fn default_ai_provider() -> String {
    "claude".to_string()
}

fn default_ai_timeout() -> u64 {
    30
}

fn default_max_context_tokens() -> usize {
    4096
}

fn default_show_typing() -> bool {
    true
}

fn default_ai_enabled() -> bool {
    true
}

/// AI provider configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProvidersConfig {
    /// OpenAI configuration
    #[serde(default)]
    pub openai: Option<AIProviderConfig>,

    /// Anthropic configuration
    #[serde(default)]
    pub anthropic: Option<AIProviderConfig>,

    /// Claude configuration
    #[serde(default)]
    pub claude: Option<AIProviderConfig>,

    /// Local LLM configuration
    #[serde(default)]
    pub local: Option<LocalAIConfig>,

    /// Custom providers
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, AIProviderConfig>,
}

impl Default for AIProvidersConfig {
    fn default() -> Self {
        Self {
            openai: None,
            anthropic: None,
            claude: None,
            local: None,
            custom: std::collections::HashMap::new(),
        }
    }
}

/// Base AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    /// API key for the provider
    #[serde(default)]
    pub api_key: String,

    /// API endpoint URL
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    /// Model to use
    #[serde(default = "default_model")]
    pub model: String,

    /// Custom headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
}

impl Default for AIProviderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            endpoint: default_endpoint(),
            model: default_model(),
            headers: std::collections::HashMap::new(),
        }
    }
}

fn default_endpoint() -> String {
    String::new()
}

fn default_model() -> String {
    String::new()
}

/// Local AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAIConfig {
    /// Command to run the local LLM server
    #[serde(default = "default_local_command")]
    pub command: String,

    /// Model to use
    #[serde(default = "default_local_model")]
    pub model: String,

    /// Endpoint URL
    #[serde(default = "default_local_endpoint")]
    pub endpoint: String,

    /// Timeout for local requests
    #[serde(default = "default_local_timeout")]
    pub timeout: u64,
}

impl Default for LocalAIConfig {
    fn default() -> Self {
        Self {
            command: default_local_command(),
            model: default_local_model(),
            endpoint: default_local_endpoint(),
            timeout: default_local_timeout(),
        }
    }
}

fn default_local_command() -> String {
    "ollama".to_string()
}

fn default_local_model() -> String {
    "llama3".to_string()
}

fn default_local_endpoint() -> String {
    "http://localhost:11434".to_string()
}

fn default_local_timeout() -> u64 {
    60
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsConfig {
    /// Directory to load plugins from
    #[serde(default = "default_plugin_dir")]
    pub plugin_dir: PathBuf,

    /// List of enabled plugins
    #[serde(default)]
    pub enabled: Vec<String>,

    /// Whether to auto-update plugins
    #[serde(default = "default_auto_update_plugins")]
    pub auto_update: bool,

    /// Plugin-specific configurations
    #[serde(default)]
    pub configs: std::collections::HashMap<String, toml::Value>,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            plugin_dir: default_plugin_dir(),
            enabled: default_enabled_plugins(),
            auto_update: default_auto_update_plugins(),
            configs: std::collections::HashMap::new(),
        }
    }
}

fn default_plugin_dir() -> PathBuf {
    let proj_dirs = ProjectDirs::from("dev", "terrust", "TerRust")
        .or_else(|| ProjectDirs::from("com", "terrust", "TerRust"));
    
    proj_dirs
        .map(|d| d.data_local_dir().join("plugins"))
        .unwrap_or_else(|| PathBuf::from("./plugins"))
}

fn default_enabled_plugins() -> Vec<String> {
    vec!["git_helper".to_string()] // Default enabled plugins
}

fn default_auto_update_plugins() -> bool {
    true
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    #[serde(default = "default_telemetry_enabled")]
    pub enabled: bool,

    /// Whether to send anonymous usage statistics
    #[serde(default = "default_usage_stats")]
    pub anonymous_usage_stats: bool,

    /// Whether to send crash reports
    #[serde(default = "default_crash_reports")]
    pub crash_reports: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_telemetry_enabled(),
            anonymous_usage_stats: default_usage_stats(),
            crash_reports: default_crash_reports(),
        }
    }
}

fn default_telemetry_enabled() -> bool {
    false
}

fn default_usage_stats() -> bool {
    false
}

fn default_crash_reports() -> bool {
    true
}

impl Config {
    /// Get the default configuration file path
    pub fn default_config_path() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("dev", "terrust", "TerRust")
            .or_else(|| ProjectDirs::from("com", "terrust", "TerRust"));
        
        proj_dirs.map(|d| d.config_local_dir().join("terrust.toml"))
    }

    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path.as_ref()))
    }

    /// Save configuration to a file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path.as_ref()))
    }

    /// Get the path to the themes directory
    pub fn themes_dir() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("dev", "terrust", "TerRust")
            .or_else(|| ProjectDirs::from("com", "terrust", "TerRust"));
        
        proj_dirs.map(|d| d.data_local_dir().join("themes"))
    }

    /// Get the path to the plugins directory
    pub fn plugins_dir(&self) -> PathBuf {
        self.plugins.plugin_dir.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.theme, "tokyo-night");
        assert!(config.ai.enabled);
        assert_eq!(config.ai.default_provider, "claude");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config.general.theme, deserialized.general.theme);
    }

    #[test]
    fn test_load_save_config() {
        let config = Config {
            general: GeneralConfig {
                theme: "catppuccin".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        
        config.to_file(path).unwrap();
        let loaded = Config::from_file(path).unwrap();
        
        assert_eq!(loaded.general.theme, "catppuccin");
    }

    #[test]
    fn test_ai_config_defaults() {
        let ai_config = AIConfig::default();
        assert_eq!(ai_config.default_provider, "claude");
        assert_eq!(ai_config.timeout, 30);
        assert_eq!(ai_config.max_context_tokens, 4096);
    }

    #[test]
    fn test_terminal_config_defaults() {
        let terminal_config = TerminalConfig::default();
        assert!(terminal_config.cursor_blink);
        assert!(terminal_config.mouse_support);
    }

    #[test]
    fn test_plugin_config_defaults() {
        let plugin_config = PluginsConfig::default();
        assert!(plugin_config.auto_update);
        assert!(plugin_config.enabled.contains(&"git_helper".to_string()));
    }
}
