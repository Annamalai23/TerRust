//! Terminal configuration structures

/// Configuration for spawning a shell process
#[derive(Debug, Clone)]
pub struct ShellConfig {
    /// Path to the shell executable
    pub shell: String,
    /// Arguments to pass to the shell
    pub args: Vec<String>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
            args: vec![],
        }
    }
}
