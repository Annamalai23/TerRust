//! TerRust - Next-Generation Terminal Environment
//!
//! A fast, native, CLI application built in Rust that combines traditional terminal
//! workflows with AI assistance and visual enhancements.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use terrust::config::Config;
use std::path::PathBuf;
use tracing::{error, info};

/// TerRust - Next-Generation Terminal Environment
#[derive(Parser, Debug)]
#[command(name = "terrust")]
#[command(author = "TerRust Team")]
#[command(version = "0.1.0")]
#[command(about = "A terminal emulator with AI assistance and visual enhancements")]
#[command(long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Disable AI features
    #[arg(long)]
    no_ai: bool,

    /// Shell to use (default: from config or SHELL environment variable)
    #[arg(short, long, value_name = "SHELL")]
    shell: Option<String>,

    /// Start in fullscreen mode
    #[arg(long)]
    fullscreen: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// List available plugins
    #[arg(long)]
    list_plugins: bool,

    /// Plugin directory
    #[arg(long, value_name = "DIR")]
    plugin_dir: Option<PathBuf>,

    /// Optional subcommand (theme management, etc.)
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Theme management commands
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
}

#[derive(Subcommand, Debug)]
enum ThemeAction {
    /// List available themes (built-in and user themes)
    List,
    /// Preview a theme's colors in the terminal
    Preview {
        /// Theme name (e.g. "tokyo-night", "dracula")
        name: String,
    },
    /// Show a theme's full configuration as TOML
    Show {
        /// Theme name
        name: String,
    },
    /// Edit a single field in a theme and save to user themes directory
    Edit {
        /// Theme name
        name: String,
        /// Key=value pair, e.g. background=#ff0000
        key_value: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting TerRust v0.1.0");

    // Load configuration
    let config_path = args.config.clone().unwrap_or_else(|| {
        Config::default_config_path().unwrap_or_else(|| PathBuf::from("./terrust.toml"))
    });

    let mut config = if config_path.exists() {
        Config::from_file(&config_path)
            .with_context(|| format!("Failed to load config from {:?}", config_path))?
    } else {
        info!("No config file found at {:?}, using defaults", config_path);
        Config::default()
    };

    // Validate configuration
    for warning in config.validate() {
        info!("Config warning: {}", warning);
    }

    // Override config with command line arguments
    if let Some(shell) = args.shell {
        config.terminal.shell = shell;
    }

    // Handle subcommands (theme management, etc.)
    if let Some(cmd) = args.command {
        match cmd {
            Commands::Theme { action } => match action {
                ThemeAction::List => terrust::cli::list_themes()?,
                ThemeAction::Preview { name } => terrust::cli::preview_theme(&name)?,
                ThemeAction::Show { name } => terrust::cli::show_theme(&name)?,
                ThemeAction::Edit { name, key_value } => terrust::cli::edit_theme(&name, &key_value)?,
            },
        }
        return Ok(());
    }

    // List plugins and exit if requested
    if args.list_plugins {
        let plugin_manager = terrust::plugins::PluginManager::new(
            args.plugin_dir.clone().unwrap_or_else(|| config.plugins.plugin_dir.clone()),
        );
        let plugins = plugin_manager.list_plugins()?;
        println!("Available plugins:");
        for plugin in plugins {
            println!("  - {} (v{}) - {}", plugin.name, plugin.version, plugin.description);
        }
        return Ok(());
    }

    // Initialize and run the application
    let result = terrust::app::App::new(config, args.no_ai, args.fullscreen, args.plugin_dir)
        .with_context(|| "Failed to initialize application")?
        .run()
        .await;

    if let Err(e) = result {
        error!("Application error: {}", e);
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["terrust"]);
        assert!(!args.no_ai);
        assert!(!args.fullscreen);
        assert!(!args.verbose);
        assert!(args.command.is_none());
    }

    #[test]
    fn test_args_with_flags() {
        let args = Args::parse_from(["terrust", "--no-ai", "--fullscreen", "-v"]);
        assert!(args.no_ai);
        assert!(args.fullscreen);
        assert!(args.verbose);
    }

    #[test]
    fn test_args_with_shell() {
        let args = Args::parse_from(["terrust", "-s", "zsh"]);
        assert_eq!(args.shell, Some("zsh".to_string()));
    }

    #[test]
    fn test_theme_list_subcommand() {
        let args = Args::parse_from(["terrust", "theme", "list"]);
        let cmd = args.command.unwrap();
        match cmd {
            Commands::Theme { action } => match action {
                ThemeAction::List => {} // expected
                _ => panic!("Expected List action"),
            },
        }
    }

    #[test]
    fn test_theme_preview_subcommand() {
        let args = Args::parse_from(["terrust", "theme", "preview", "tokyo-night"]);
        let cmd = args.command.unwrap();
        match cmd {
            Commands::Theme { action } => match action {
                ThemeAction::Preview { name } => assert_eq!(name, "tokyo-night"),
                _ => panic!("Expected Preview action"),
            },
        }
    }

    #[test]
    fn test_theme_show_subcommand() {
        let args = Args::parse_from(["terrust", "theme", "show", "dracula"]);
        let cmd = args.command.unwrap();
        match cmd {
            Commands::Theme { action } => match action {
                ThemeAction::Show { name } => assert_eq!(name, "dracula"),
                _ => panic!("Expected Show action"),
            },
        }
    }

    #[test]
    fn test_theme_edit_subcommand() {
        let args = Args::parse_from(["terrust", "theme", "edit", "my-theme", "background=#000000"]);
        let cmd = args.command.unwrap();
        match cmd {
            Commands::Theme { action } => match action {
                ThemeAction::Edit { name, key_value } => {
                    assert_eq!(name, "my-theme");
                    assert_eq!(key_value, "background=#000000");
                }
                _ => panic!("Expected Edit action"),
            },
        }
    }
}
