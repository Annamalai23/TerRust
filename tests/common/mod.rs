use terrust::config::{AIConfig, AIProviderConfig, AIProvidersConfig, Config, KeybindingsConfig, LocalAIConfig, TerminalConfig, ThemeConfig};
use terrust::terminal::{Cell, Grid, ShellConfig};
use std::path::PathBuf;

pub fn test_config() -> Config {
    Config {
        general: terrust::config::GeneralConfig {
            theme: "tokyo-night".to_string(),
            font: "monospace".to_string(),
            opacity: 1.0,
            scrollback_limit: 10000,
            startup_commands: vec![],
            default_working_dir: PathBuf::from("/tmp"),
        },
        terminal: TerminalConfig {
            shell: "/bin/bash".to_string(),
            shell_args: vec![],
            cursor_style: "block".to_string(),
            cursor_blink: true,
            bell_notification: false,
            mouse_support: true,
            true_color: true,
        },
        ai: AIConfig {
            enabled: false,
            default_provider: "mock".to_string(),
            timeout: 30,
            max_context_tokens: 4096,
            show_typing_indicator: false,
            providers: AIProvidersConfig {
                openai: None,
                anthropic: None,
                claude: Some(AIProviderConfig {
                    api_key: "test".to_string(),
                    endpoint: String::new(),
                    model: String::new(),
                    headers: std::collections::HashMap::new(),
                }),
                local: Some(LocalAIConfig {
                    command: "echo".to_string(),
                    model: "test".to_string(),
                    endpoint: String::new(),
                    timeout: 30,
                }),
                custom: std::collections::HashMap::new(),
            },
        },
        plugins: terrust::config::PluginsConfig {
            plugin_dir: PathBuf::from("/tmp/plugins"),
            enabled: vec![],
            auto_update: false,
            configs: std::collections::HashMap::new(),
        },
        theme: ThemeConfig::tokyo_night(),
        keybindings: KeybindingsConfig::default(),
        telemetry: terrust::config::TelemetryConfig {
            enabled: false,
            anonymous_usage_stats: false,
            crash_reports: false,
        },
    }
}

pub fn test_grid(cols: u16, rows: u16) -> Grid {
    Grid::new(cols, rows)
}

pub fn test_shell_config() -> ShellConfig {
    ShellConfig {
        shell: "/bin/bash".to_string(),
        args: vec![],
    }
}

pub fn row_from_str(s: &str) -> Vec<Cell> {
    s.chars().map(|c| Cell {
        character: c,
        foreground: None,
        background: None,
        attributes: terrust::terminal::Attributes::default(),
    }).collect()
}
