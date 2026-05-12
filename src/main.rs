//! TerRust - Next-Generation Terminal Environment
//!
//! A fast, native, CLI application built in Rust that combines traditional terminal
//! workflows with AI assistance and visual enhancements.

mod app;
mod config;
mod history;
mod plugins;
mod terminal;
mod ui;
mod utils;

#[cfg(feature = "ai")]
mod ai;

use anyhow::{Context, Result};
use clap::Parser;
use config::Config;
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

    // Override config with command line arguments
    if let Some(shell) = args.shell {
        config.terminal.shell = shell;
    }

    // List plugins and exit if requested
    if args.list_plugins {
        let plugin_manager = plugins::PluginManager::new(
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
    let result = app::App::new(config, args.no_ai, args.fullscreen, args.plugin_dir)
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

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["terrust"]);
        assert!(!args.no_ai);
        assert!(!args.fullscreen);
        assert!(!args.verbose);
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
}
