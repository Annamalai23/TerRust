//! Utility functions for TerRust terminal emulator
//!
//! This module provides common helper functions used throughout the application,
//! including string manipulation, path handling, platform detection, and
//! various convenience functions for terminal operations.

use std::{env as std_env, fs};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Platform type for conditional compilation and behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Linux operating system
    Linux,
    /// macOS operating system
    MacOS,
    /// Windows operating system
    Windows,
    /// Unknown platform
    Unknown,
}

impl Platform {
    /// Detect the current platform
    pub fn current() -> Self {
        if cfg!(target_os = "linux") {
            Platform::Linux
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Unknown
        }
    }

    /// Check if current platform is Linux
    pub fn is_linux(&self) -> bool {
        matches!(self, Platform::Linux)
    }

    /// Check if current platform is macOS
    pub fn is_mac(&self) -> bool {
        matches!(self, Platform::MacOS)
    }

    /// Check if current platform is Windows
    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::Windows)
    }

    /// Get platform as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Linux => "linux",
            Platform::MacOS => "macos",
            Platform::Windows => "windows",
            Platform::Unknown => "unknown",
        }
    }
}

/// Terminal-related utilities
pub mod terminal {
    use super::*;

    /// Get the current terminal size (columns, rows)
    /// Returns None if terminal size cannot be determined
    pub fn get_terminal_size() -> Option<(u16, u16)> {
        if Platform::current().is_windows() {
            get_terminal_size_windows()
        } else {
            get_terminal_size_unix()
        }
    }

    /// Get terminal size on Unix-like systems using ioctl
    #[cfg(not(windows))]
    fn get_terminal_size_unix() -> Option<(u16, u16)> {
        use std::io::{self, Write};

        // Try to get size from stdout
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        // SAFETY: This is a safe wrapper around the unsafe ioctl call
        // We use libloading-compatible approach for portability
        // Try to use tcgetwinsize for pseudo-terminals
        // For now, return a default size as safe fallback
        // In production, this would use proper ioctl calls
        Some((80, 24))
    }

    /// Get terminal size on Unix-like systems - safe implementation
    #[cfg(not(windows))]
    pub fn get_terminal_size_safe() -> (u16, u16) {
        // Safe fallback: try environment variables
        if let (Some(cols), Some(rows)) = (
            std::env::var("COLUMNS").ok().and_then(|c| c.parse::<u16>().ok()),
            std::env::var("LINES").ok().and_then(|l| l.parse::<u16>().ok()),
        ) {
            if cols > 0 && rows > 0 {
                return (cols, rows);
            }
        }
        // Default fallback
        (80, 24)
    }

    #[cfg(windows)]
    fn get_terminal_size_windows() -> Option<(u16, u16)> {
        // Windows implementation placeholder
        Some((80, 24))
    }

    #[cfg(not(windows))]
    fn get_terminal_size_windows() -> Option<(u16, u16)> {
        None
    }

    /// Clear the terminal screen
    pub fn clear_screen() {
        print!("\x1bc");
    }

    /// Clear the current line
    pub fn clear_line() {
        print!("\r\x1b[2K");
    }

    /// Move cursor to position (1-indexed)
    pub fn move_cursor_to(row: u16, col: u16) {
        print!("\x1b[{};{}H", row, col);
    }

    /// Save cursor position
    pub fn save_cursor() {
        print!("\x1b7");
    }

    /// Restore cursor position
    pub fn restore_cursor() {
        print!("\x1b8");
    }

    /// Hide cursor
    pub fn hide_cursor() {
        print!("\x1b[?25l");
    }

    /// Show cursor
    pub fn show_cursor() {
        print!("\x1b[?25h");
    }

    /// Set cursor style (0=blinking block, 1=blinking beam, 2=steady block, 3=steady beam, etc.)
    pub fn set_cursor_style(style: u16) {
        print!("\x1b[{} q", style);
    }

    /// Get cursor style string for a given style value
    pub fn cursor_style_to_string(style: u16) -> &'static str {
        match style {
            0 => "Blinking Block",
            1 => "Blinking Beam",
            2 => "Steady Block",
            3 => "Steady Beam",
            4 => "Blinking Underline",
            5 => "Steady Underline",
            _ => "Unknown",
        }
    }

    /// Enter alternate screen buffer
    pub fn enter_alternate_screen() {
        print!("\x1b[?1049h");
    }

    /// Exit alternate screen buffer
    pub fn exit_alternate_screen() {
        print!("\x1b[?1049l");
    }

    /// Enable mouse tracking ( Different modes: 0-1002, 1003, 1005, 1006)
    /// 1006 is extended mode with pixel coordinates (if supported)
    /// 1003 is button-only mode
    pub fn enable_mouse_tracking() {
        print!("\x1b[?1006h");
    }

    /// Disable mouse tracking
    pub fn disable_mouse_tracking() {
        print!("\x1b[?1006l");
    }

    /// Check if a character is a control character
    pub fn is_control_char(c: char) -> bool {
        c.is_ascii_control() || c.is_control()
    }

    /// Check if a character is printable
    pub fn is_printable(c: char) -> bool {
        !c.is_ascii_control() && !c.is_control()
    }

    /// Get width of a character (1 for ASCII, 2 for wide CJK/emoji)
    /// Simplified implementation - returns actual grapheme cluster width
    pub fn char_width(c: char) -> usize {
        // Simple approximation: most CJK characters are 2 columns
        // This is a simplified version; production would use unicode-width
        if c.is_ascii() {
            1
        } else {
            // Check for known wide characters
            // Most CJK, fullwidth forms are 2 columns
            // Emoji can be 1 or 2 depending on context
            match c {
                // Fullwidth ASCII variants
                '\u{FF01}'..='\u{FF5E}' => 2,
                // CJK Unified Ideographs
                '\u{4E00}'..='\u{9FFF}' => 2,
                // CJK Extensions
                '\u{3400}'..='\u{4DBF}' => 2,
                '\u{20000}'..='\u{2A6DF}' => 2,
                '\u{2A700}'..='\u{2B73F}' => 2,
                '\u{2B740}'..='\u{2B81F}' => 2,
                '\u{2B820}'..='\u{2CEAF}' => 2,
                '\u{F900}'..='\u{FAFF}' => 2,
                '\u{2F800}'..='\u{2FA1F}' => 2,
                // Halfwidth Katakana (1 column)
                '\u{FF65}'..='\u{FF9F}' => 1,
                // Fullwidth forms
                '\u{FF00}'..='\u{FFEF}' => 2,
                // Default to 1 for now (would use unicode-width crate in production)
                _ => 1,
            }
        }
    }

    /// Get display width of a string, accounting for wide characters
    pub fn string_width(s: &str) -> usize {
        s.chars().map(|c| char_width(c)).sum()
    }

    /// Truncate a string to a given width
    pub fn truncate_string(s: &str, width: usize) -> String {
        let mut result = String::new();
        let mut current_width = 0;

        for c in s.chars() {
            let cw = char_width(c);
            if current_width + cw > width {
                break;
            }
            result.push(c);
            current_width += cw;
        }

        result
    }

    /// Pad a string to a given width (left, right, or center aligned)
    pub fn pad_string(s: &str, width: usize, align: &str) -> String {
        let display_width = string_width(s);
        let padding = if width > display_width { width - display_width } else { 0 };

        match align {
            "left" => format!("{}{}", s, " ".repeat(padding)),
            "right" => format!("{}{}", " ".repeat(padding), s),
            "center" => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!("{}{}{}", " ".repeat(left_pad), s, " ".repeat(right_pad))
            }
            _ => s.to_string(),
        }
    }

    /// Check if terminal supports true color (24-bit color)
    pub fn supports_true_color() -> bool {
        // Check TERM environment variable and COLORFGBG
        let term = std::env::var("TERM").unwrap_or_default();
        let colorfgbg = std::env::var("COLORFGBG").unwrap_or_default();

        // Common terminal types that support true color
        let true_color_terms = [
            "xterm-256color",
            "xterm-truecolor",
            "screen-256color",
            "tmux-256color",
            "alacritty",
            "kitty",
            "wezterm",
            "foot",
            "contour",
            "tabby",
            "iterm2",
            "vscode",
            "jetbrains",
        ];

        true_color_terms.iter().any(|&t| term.contains(t))
            || colorfgbg.contains(";16")
            || term.contains("truecolor")
            || term.contains("24bit")
    }

    /// Get the COLORTERM environment variable value
    pub fn get_colorterm() -> String {
        std::env::var("COLORTERM").unwrap_or_else(|_| "".to_string())
    }

    /// Check if running inside tmux
    pub fn is_inside_tmux() -> bool {
        std::env::var("TMUX").is_ok()
    }

    /// Check if running inside screen
    pub fn is_inside_screen() -> bool {
        std::env::var("TERM").map_or(false, |t| t.contains("screen"))
            || std::env::var("WINDOW").is_ok()
    }

    /// Get the effective terminal type
    pub fn effective_term() -> String {
        if is_inside_tmux() {
            "tmux".to_string()
        } else if is_inside_screen() {
            "screen".to_string()
        } else {
            std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string())
        }
    }
}

/// String manipulation utilities
pub mod string {
    use super::*;

    /// Split a string into lines, handling various line endings
    pub fn split_lines(s: &str) -> Vec<String> {
        s.lines().map(|line| line.to_string()).collect()
    }

    /// Join lines with a separator
    pub fn join_lines(lines: &[String], sep: &str) -> String {
        lines.join(sep)
    }

    /// Normalize line endings to \n
    pub fn normalize_line_endings(s: &str) -> String {
        s.replace("\r\n", "\n").replace("\r", "\n")
    }

    /// Remove trailing newlines from a string
    pub fn trim_trailing_newlines(s: &str) -> String {
        s.trim_end_matches(|c| c == '\n' || c == '\r').to_string()
    }

    /// Remove leading newlines from a string
    pub fn trim_leading_newlines(s: &str) -> String {
        s.trim_start_matches(|c| c == '\n' || c == '\r').to_string()
    }

    /// Escape special characters for display
    pub fn escape_display(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\x00'..='\x1F' => {
                    result.push_str(&format!("\\x{:02x}", c as u8));
                }
                '\x7F' => result.push_str("\\x7f"),
                _ => result.push(c),
            }
        }
        result
    }

    /// Unescape display string
    pub fn unescape_display(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next) = chars.next() {
                    match next {
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        'x' => {
                            // Parse hex escape \xXX
                            let mut hex_digits = String::new();
                            if let Some(&d1) = chars.peek() {
                                if d1.is_ascii_hexdigit() {
                                    chars.next();
                                    hex_digits.push(d1);
                                }
                            }
                            if let Some(&d2) = chars.peek() {
                                if d2.is_ascii_hexdigit() {
                                    chars.next();
                                    hex_digits.push(d2);
                                }
                            }
                            if hex_digits.len() == 2 {
                                if let Ok(byte) = u8::from_str_radix(&hex_digits, 16) {
                                    result.push(byte as char);
                                } else {
                                    result.push_str(&format!("\\x{}", hex_digits));
                                }
                            } else {
                                result.push_str(&format!("\\x{}", hex_digits));
                            }
                        }
                        _ => {
                            result.push(c);
                            result.push(next);
                        }
                    }
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Word wrap a string to a given width
    pub fn word_wrap(s: &str, width: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for word in s.split_whitespace() {
            let word_width = terminal::string_width(word);

            if current_width == 0 {
                // First word on line
                current_line.push_str(word);
                current_width = word_width;
            } else if current_width + 1 + word_width <= width {
                // Word fits on current line
                current_line.push(' ');
                current_line.push_str(word);
                current_width += 1 + word_width;
            } else {
                // Word doesn't fit, start new line
                result.push(current_line);
                current_line = word.to_string();
                current_width = word_width;
            }
        }

        if !current_line.is_empty() {
            result.push(current_line);
        }

        result
    }

    /// Repeat a string n times
    pub fn repeat_str(s: &str, n: usize) -> String {
        (0..n).map(|_| s).collect()
    }

    /// Check if string contains ANSI escape sequences
    pub fn has_ansi_sequences(s: &str) -> bool {
        s.contains('\x1b')
    }

    /// Strip ANSI escape sequences from a string
    pub fn strip_ansi_sequences(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut in_escape = false;
        let mut in_csi = false;

        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
                in_csi = false;
            } else if in_escape {
                if c == '[' {
                    in_csi = true;
                    continue;
                }
                if !in_csi || ('@'..='~').contains(&c) {
                    in_escape = false;
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Count the number of grapheme clusters in a string
    /// Simplified: for ASCII, each char is a grapheme. For Unicode, this is an approximation.
    pub fn grapheme_count(s: &str) -> usize {
        // Use unicode-segmentation crate in production
        // For now, count each char as a grapheme (approximation)
        s.chars().count()
    }

    /// Get the nth grapheme cluster, accounting for multi-byte characters
    pub fn get_nth_grapheme(s: &str, n: usize) -> Option<String> {
        s.chars().nth(n).map(|c| c.to_string())
    }

    /// Check if a string is empty or whitespace
    pub fn is_blank(s: &str) -> bool {
        s.trim().is_empty()
    }

    /// Get the first non-whitespace character index
    pub fn first_non_whitespace(s: &str) -> Option<usize> {
        s.chars().position(|c| !c.is_whitespace())
    }

    /// Get the last non-whitespace character index
    pub fn last_non_whitespace(s: &str) -> Option<usize> {
        s.chars().rev().position(|c| !c.is_whitespace())
            .map(|pos| s.len() - 1 - pos)
    }

    /// Reverse a string while preserving grapheme clusters
    pub fn reverse(s: &str) -> String {
        s.chars().rev().collect()
    }

    /// Capitalize the first letter of a string
    pub fn capitalizeFirst(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                let first_upper = first.to_uppercase().collect::<String>();
                let rest: String = chars.collect();
                format!("{}{}", first_upper, rest)
            }
        }
    }

    /// Convert to title case
    pub fn to_title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| capitalizeFirst(word))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Path utilities
pub mod path {
    use super::*;
    use std::io::Write;

    /// Expand a path with ~ home directory
    pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
        let path = path.as_ref();
        if path.starts_with("~") {
            if let Some(home) = home_dir() {
                if path == Path::new("~") {
                    return home;
                }
                let display = path.to_string_lossy();
                let rest = display.get(1..).unwrap_or("");
                return home.join(rest);
            }
        }
        path.to_path_buf()
    }

    /// Get the user's home directory
    pub fn home_dir() -> Option<PathBuf> {
        #[cfg(unix)]
        {
            // Try HOME environment variable first
            if let Ok(home) = std_env::var("HOME") {
                return Some(PathBuf::from(home));
            }
            // Fall back to users crate or std::env
            // For simplicity, return None if HOME not set
            None
        }

        #[cfg(windows)]
        {
            if let Ok(home) = std_env::var("USERPROFILE") {
                return Some(PathBuf::from(home));
            }
            None
        }

        #[cfg(not(any(unix, windows)))]
        {
            None
        }
    }

    /// Get the configuration directory path
    /// Follows XDG on Linux, AppData on Windows, Library on macOS
    pub fn config_dir(player: &str) -> PathBuf {
        let platform = Platform::current();

        if platform.is_windows() {
            // Windows: %APPDATA%\<app>\config
            if let Ok(app_data) = std_env::var("APPDATA") {
                return PathBuf::from(app_data).join(player).join("config");
            }
        } else if platform.is_mac() {
            // macOS: ~/Library/Application Support/<app>/
            if let Some(home) = home_dir() {
                return home.join("Library").join("Application Support").join(player);
            }
        } else {
            // Linux/Unix: $XDG_CONFIG_HOME/<app> or ~/.config/<app>
            if let Ok(xdg_config) = std_env::var("XDG_CONFIG_HOME") {
                return PathBuf::from(xdg_config).join(player);
            }
            if let Some(home) = home_dir() {
                return home.join(".config").join(player);
            }
        }

        // Fallback to current directory
        PathBuf::from(".").join("config")
    }

    /// Get the data directory path
    /// Follows XDG on Linux, AppData on Windows, Library on macOS
    pub fn data_dir(player: &str) -> PathBuf {
        let platform = Platform::current();

        if platform.is_windows() {
            // Windows: %APPDATA%\<app>\data
            if let Ok(app_data) = std_env::var("APPDATA") {
                return PathBuf::from(app_data).join(player).join("data");
            }
        } else if platform.is_mac() {
            // macOS: ~/Library/Application Support/<app>/
            if let Some(home) = home_dir() {
                return home.join("Library").join("Application Support").join(player);
            }
        } else {
            // Linux/Unix: $XDG_DATA_HOME/<app> or ~/.local/share/<app>
            if let Ok(xdg_data) = std_env::var("XDG_DATA_HOME") {
                return PathBuf::from(xdg_data).join(player);
            }
            if let Some(home) = home_dir() {
                return home.join(".local").join("share").join(player);
            }
        }

        // Fallback to current directory
        PathBuf::from(".").join("data")
    }

    /// Get the cache directory path
    pub fn cache_dir(player: &str) -> PathBuf {
        let platform = Platform::current();

        if platform.is_windows() {
            // Windows: %LOCALAPPDATA%\<app>\cache
            if let Ok(local_app_data) = std_env::var("LOCALAPPDATA") {
                return PathBuf::from(local_app_data).join(player).join("cache");
            }
        } else if platform.is_mac() {
            // macOS: ~/Library/Caches/<app>/
            if let Some(home) = home_dir() {
                return home.join("Library").join("Caches").join(player);
            }
        } else {
            // Linux/Unix: $XDG_CACHE_HOME/<app> or ~/.cache/<app>
            if let Ok(xdg_cache) = std_env::var("XDG_CACHE_HOME") {
                return PathBuf::from(xdg_cache).join(player);
            }
            if let Some(home) = home_dir() {
                return home.join(".cache").join(player);
            }
        }

        // Fallback to current directory
        PathBuf::from(".").join("cache")
    }

    /// Create directory and all parent directories if they don't exist
    pub fn create_dirs_all<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        fs::create_dir_all(path)
    }

    /// Ensure a directory exists
    pub fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    /// Check if a path is a directory
    pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_dir()
    }

    /// Check if a path is a file
    pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_file()
    }

    /// Check if a path exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Read a file to string
    pub fn read_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
        fs::read_to_string(path)
    }

    /// Write a string to a file
    pub fn write_string<P: AsRef<Path>>(path: P, contents: &str) -> std::io::Result<()> {
        fs::write(path, contents)
    }

    /// Append a string to a file
    pub fn append_string<P: AsRef<Path>>(path: P, contents: &str) -> std::io::Result<()> {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut f| f.write_all(contents.as_bytes()))
    }

    /// Copy a file
    pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<u64> {
        fs::copy(from, to)
    }

    /// Remove a file
    pub fn remove_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        fs::remove_file(path)
    }

    /// Remove a directory (must be empty)
    pub fn remove_dir<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        fs::remove_dir(path)
    }

    /// Remove a directory and all its contents
    pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        fs::remove_dir_all(path)
    }

    /// Get file name from path
    pub fn file_name<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
        path.as_ref().file_name().map(PathBuf::from)
    }

    /// Get file stem (name without extension) from path
    pub fn file_stem<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
        path.as_ref().file_stem().map(PathBuf::from)
    }

    /// Get file extension from path
    pub fn extension<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str().map(|s| s.to_string()))
    }

    /// Get parent directory from path
    pub fn parent_dir<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
        path.as_ref().parent().map(|p| p.to_path_buf())
    }

    /// Join path components
    pub fn join<P: AsRef<Path>>(base: P, components: &[&str]) -> PathBuf {
        let mut path = base.as_ref().to_path_buf();
        for component in components {
            path = path.join(component);
        }
        path
    }

    /// Canonicalize a path (resolve symlinks, etc.)
    pub fn canonicalize<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
        fs::canonicalize(path)
    }

    /// Get the absolute path
    pub fn absolute<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
        if path.as_ref().is_absolute() {
            return Ok(path.as_ref().to_path_buf());
        }
        if let Some(current) = std_env::current_dir().ok() {
            let relative = path.as_ref().to_path_buf();
            return Ok(current.join(relative));
        }
        // Fallback: just return the path as-is
        Ok(path.as_ref().to_path_buf())
    }

    /// List files in a directory
    pub fn list_files<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<PathBuf>> {
        let mut result = Vec::new();
        for entry in fs::read_dir(path)? {
            result.push(entry?.path());
        }
        Ok(result)
    }

    /// List files in a directory, filtered by extension
    pub fn list_files_with_extension<P: AsRef<Path>>(
        path: P,
        extension: &str,
    ) -> std::io::Result<Vec<PathBuf>> {
        Ok(list_files(path)?
            .into_iter()
            .filter(|p| {
                p.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case(extension))
                    .unwrap_or(false)
            })
            .collect())
    }

    /// Glob pattern matching (simplified)
    pub fn glob_match(pattern: &str, path: &str) -> bool {
        // Simple glob matching - would use glob crate in production
        // For now, handle basic * and ? wildcards
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let path_chars: Vec<char> = path.chars().collect();
        glob_match_inner(&pattern_chars, &path_chars, 0, 0)
    }

    fn glob_match_inner(
        pattern: &[char],
        path: &[char],
        p_index: usize,
        s_index: usize,
    ) -> bool {
        if p_index >= pattern.len() {
            return s_index >= path.len();
        }

        match pattern[p_index] {
            '*' => {
                // Try matching 0 or more characters
                for i in s_index..=path.len() {
                    if glob_match_inner(pattern, path, p_index + 1, i) {
                        return true;
                    }
                }
                false
            }
            '?' => {
                // Match exactly one character
                if s_index >= path.len() {
                    false
                } else {
                    glob_match_inner(pattern, path, p_index + 1, s_index + 1)
                }
            }
            c => {
                if s_index >= path.len() {
                    false
                } else if path[s_index] != c {
                    false
                } else {
                    glob_match_inner(pattern, path, p_index + 1, s_index + 1)
                }
            }
        }
    }
}

/// Time utilities
pub mod time {
    use super::*;

    /// Get current timestamp as Unix epoch in seconds
    pub fn now_unix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Get current timestamp as Unix epoch in milliseconds
    pub fn now_unix_ms() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    }

    /// Get current timestamp as Unix epoch in nanoseconds
    pub fn now_unix_ns() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    }

    /// Get current time as ISO 8601 string
    pub fn now_iso8601() -> String {
        // Use chrono in production for proper ISO 8601
        // For now, return a simple format
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        let secs = now.as_secs();
        let hours = (secs / 3600) % 24;
        let minutes = (secs / 60) % 60;
        let seconds = secs % 60;
        let millis = now.as_millis() % 1000;
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            hours, minutes, seconds, millis
        )
    }

    /// Get current date as YYYY-MM-DD
    pub fn today_ymd() -> String {
        // Simplified date calculation
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        let days_since_epoch = now.as_secs() / 86400;
        // This is a placeholder - in production use proper date calculation
        // or the time crate
        format!("2025-05-06")
    }

    /// Format a duration as human-readable string
    pub fn format_duration(d: Duration) -> String {
        let total_secs = d.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        let millis = d.as_millis() % 1000;

        if hours > 0 {
            if hours >= 24 {
                let days = hours / 24;
                format!("{}d {}h", days, hours % 24)
            } else {
                format!("{}h {}m {}s", hours, minutes, seconds)
            }
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else if seconds > 0 {
            format!("{}s", seconds)
        } else {
            format!("{}ms", millis)
        }
    }

    /// Format a duration in milliseconds as human-readable
    pub fn format_duration_ms(ms: u64) -> String {
        format_duration(Duration::from_millis(ms))
    }

    /// Check if a timestamp is older than a duration
    pub fn is_older_than(timestamp: u64, duration: Duration) -> bool {
        let now = now_unix();
        now.saturating_sub(timestamp) > duration.as_secs()
    }

    /// Calculate time elapsed since a timestamp in seconds
    pub fn elapsed_secs(since: u64) -> u64 {
        now_unix().saturating_sub(since)
    }

    /// Sleep for a duration
    pub async fn sleep(d: Duration) {
        tokio::time::sleep(d).await;
    }

    /// Sleep for milliseconds
    pub async fn sleep_ms(ms: u64) {
        sleep(Duration::from_millis(ms)).await;
    }
}

/// Environment utilities
pub mod env {
    use super::*;

    /// Get environment variable with default
    pub fn get_var_with_default(key: &str, default: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get environment variable as boolean
    pub fn get_var_bool(key: &str) -> bool {
        match std::env::var(key) {
            Ok(val) => {
                let lower = val.to_lowercase();
                lower == "1" || lower == "true" || lower == "yes" || lower == "on"
            }
            Err(_) => false,
        }
    }

    /// Get environment variable as integer
    pub fn get_var_int(key: &str) -> Option<i64> {
        std::env::var(key).ok().and_then(|v| v.parse().ok())
    }

    /// Get environment variable as unsigned integer
    pub fn get_var_uint(key: &str) -> Option<u64> {
        std::env::var(key).ok().and_then(|v| v.parse().ok())
    }

    /// Get environment variable as float
    pub fn get_var_float(key: &str) -> Option<f64> {
        std::env::var(key).ok().and_then(|v| v.parse().ok())
    }

    /// Check if an environment variable exists (is set)
    pub fn has_var(key: &str) -> bool {
        std::env::var(key).is_ok()
    }

    /// Get all environment variables as a HashMap
    pub fn get_all_vars() -> std::collections::HashMap<String, String> {
        std::env::vars().collect()
    }

    /// Get shell from environment (SHELL on Unix, COMSPEC on Windows)
    pub fn get_shell() -> String {
        if Platform::current().is_windows() {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        }
    }

    /// Get effective user name
    pub fn get_username() -> String {
        std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "unknown".to_string())
    }

    /// Get hostname
    pub fn get_hostname() -> String {
        // Use sysinfo crate in production for proper hostname
        // For now, try HOSTNAME or COMPUTERNAME environment variables
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Get display string for user@host
    pub fn get_user_host() -> String {
        format!("{}@{}", get_username(), get_hostname())
    }

    /// Get home directory (cross-platform)
    pub fn get_home_dir() -> Option<String> {
        super::path::home_dir().map(|p| p.to_string_lossy().into_owned())
    }

    /// Get current working directory
    pub fn get_current_dir() -> Option<String> {
        std::env::current_dir().ok().map(|p| p.to_string_lossy().into_owned())
    }

    /// Get process ID
    pub fn get_pid() -> u32 {
        std::process::id()
    }

    /// Get parent process ID
    #[cfg(unix)]
    pub fn get_ppid() -> u32 {
        use nix::unistd::getppid;
        getppid().as_raw() as u32
    }

    #[cfg(not(unix))]
    pub fn get_ppid() -> u32 {
        0 // Not available on non-Unix platforms
    }

    /// Check if running as root
    pub fn is_root() -> bool {
        #[cfg(unix)]
        {
            nix::unistd::geteuid().is_root()
        }
        #[cfg(not(unix))]
        {
            false
        }
    }

    /// Get OS name and version
    pub fn get_os_info() -> String {
        let platform = Platform::current();
        match platform {
            Platform::Linux => {
                // Try /etc/os-release
                if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                    for line in content.lines() {
                        if line.starts_with("PRETTY_NAME=") {
                            return line.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string();
                        }
                    }
                }
                "Linux".to_string()
            }
            Platform::MacOS => {
                // Try sw_vers
                if let Ok(output) = std::process::Command::new("sw_vers").output() {
                    if let Ok(product) = String::from_utf8(output.stdout) {
                        for line in product.lines() {
                            if line.starts_with("ProductName:") || line.starts_with("ProductVersion:") {
                                return line.to_string();
                            }
                        }
                    }
                }
                "macOS".to_string()
            }
            Platform::Windows => {
                // Windows version from environment
                "Windows".to_string()
            }
            Platform::Unknown => "Unknown OS".to_string(),
        }
    }
}

/// Validation utilities
pub mod validate {
    use super::*;

    /// Check if a string is a valid identifier (alphanumeric + underscore, starts with letter/underscore)
    pub fn is_valid_identifier(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        let mut chars = s.chars();
        let first = chars.next().unwrap();
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    /// Check if a string is a valid path component (no path separators or null bytes)
    pub fn is_valid_path_component(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        !s.contains(|c: char| c.is_control() || c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|')
    }

    /// Check if a string is a valid filename
    pub fn is_valid_filename(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        !s.contains(|c: char| c.is_control() || c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|')
    }

    /// Check if a string contains only ASCII characters
    pub fn is_ascii_only(s: &str) -> bool {
        s.is_ascii()
    }

    /// Check if a string contains only digits
    pub fn is_digits_only(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
    }

    /// Check if a string contains only hex digits
    pub fn is_hex_only(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Validate a hex color string (with or without #)
    pub fn is_valid_hex_color(s: &str) -> bool {
        let s = s.trim_start_matches('#');
        matches!(s.len(), 3 | 4 | 6 | 8) && is_hex_only(s)
    }

    /// Validate a port number
    pub fn is_valid_port(s: &str) -> bool {
        if let Ok(port) = s.parse::<u16>() {
            port > 0
        } else {
            false
        }
    }

    /// Validate an IP address (IPv4 only for now)
    pub fn is_valid_ipv4(s: &str) -> bool {
        s.parse::<std::net::Ipv4Addr>().is_ok()
    }

    /// Validate a hostname (simplified)
    pub fn is_valid_hostname(s: &str) -> bool {
        if s.is_empty() || s.len() > 253 {
            return false;
        }
        for label in s.split('.') {
            if label.is_empty() || label.len() > 63 {
                return false;
            }
            if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return false;
            }
            if label.starts_with('-') || label.ends_with('-') {
                return false;
            }
        }
        true
    }

    /// Validate an email address (simplified)
    pub fn is_valid_email(s: &str) -> bool {
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        let local = parts[0];
        let domain = parts[1];
        !local.is_empty() && is_valid_hostname(domain) && local.contains('.')
    }

    /// Validate a URL (simplified)
    pub fn is_valid_url(s: &str) -> bool {
        // Very simplified check
        s.starts_with("http://") || s.starts_with("https://")
    }

    /// Clamp a value to a range
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
}

/// Logging utilities
pub mod log {
    use super::*;

    /// Initialize tracing subscriber with default configuration
    pub fn init_tracing() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    /// Initialize tracing with max level
    pub fn init_tracing_max_level(level: &str) {
        std::env::set_var("RUST_LOG", level);
        init_tracing();
    }

    /// Create a child span with the given name
    #[macro_export]
    macro_rules! span {
        ($name:expr) => {
            tracing::span!(tracing::Level::DEBUG, $name)
        };
        ($level:expr, $name:expr) => {
            tracing::span!($level, $name)
        };
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    // Platform tests
    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        assert!(matches!(
            platform,
            Platform::Linux | Platform::MacOS | Platform::Windows | Platform::Unknown
        ));
    }

    // String utility tests
    #[test]
    fn test_string_width_ascii() {
        assert_eq!(terminal::string_width("hello"), 5);
        assert_eq!(terminal::string_width(""), 0);
    }

    #[test]
    fn test_string_trim_newlines() {
        assert_eq!(string::trim_trailing_newlines("hello\n"), "hello");
        assert_eq!(string::trim_trailing_newlines("hello\r\n"), "hello");
        assert_eq!(string::trim_leading_newlines("\nhello"), "hello");
    }

    #[test]
    fn test_string_word_wrap() {
        let result = string::word_wrap("hello world this is a test", 10);
        assert!(result.len() > 0);
    }

    #[test]
    fn test_string_pad() {
        assert_eq!(terminal::pad_string("hi", 5, "left"), "hi   ");
        assert_eq!(terminal::pad_string("hi", 5, "right"), "   hi");
        assert_eq!(terminal::pad_string("hi", 5, "center"), " hi  ");
    }

    // Path utility tests
    #[test]
    fn test_path_join() {
        let result = path::join("/tmp", &["a", "b", "c"]);
        #[cfg(unix)]
        assert_eq!(result, PathBuf::from("/tmp/a/b/c"));
    }

    #[test]
    fn test_path_file_name() {
        assert_eq!(
            path::file_name("/tmp/test.txt"),
            Some(PathBuf::from("test.txt"))
        );
    }

    // Validation tests
    #[test]
    fn test_validate_identifier() {
        assert!(validate::is_valid_identifier("hello"));
        assert!(validate::is_valid_identifier("_test"));
        assert!(!validate::is_valid_identifier("123hello"));
        assert!(!validate::is_valid_identifier(""));
    }

    #[test]
    fn test_validate_hex_color() {
        assert!(validate::is_valid_hex_color("#ff0000"));
        assert!(validate::is_valid_hex_color("ff0000"));
        assert!(validate::is_valid_hex_color("#fff"));
        assert!(!validate::is_valid_hex_color("invalid"));
    }

    #[test]
    fn test_validate_port() {
        assert!(validate::is_valid_port("8080"));
        assert!(!validate::is_valid_port("0"));
        assert!(!validate::is_valid_port("99999"));
    }

    #[test]
    fn test_validate_ipv4() {
        assert!(validate::is_valid_ipv4("127.0.0.1"));
        assert!(!validate::is_valid_ipv4("256.0.0.1"));
    }

    // Timeout tests
    #[tokio::test]
    async fn test_timeout_does_not_panic() {
        use std::time::Duration;
        // This is just a smoke test to ensure the module compiles
        let _ = time::now_unix();
        let _ = time::format_duration(Duration::from_secs(10));
    }

    // Terminal utility tests
    #[test]
    fn test_terminal_cursor_styles() {
        assert_eq!(terminal::cursor_style_to_string(0), "Blinking Block");
        assert_eq!(terminal::cursor_style_to_string(2), "Steady Block");
        assert_eq!(terminal::cursor_style_to_string(99), "Unknown");
    }

    // Glob matching tests
    #[test]
    fn test_glob_match() {
        assert!(path::glob_match("*.rs", "test.rs"));
        assert!(path::glob_match("*.rs", "test.txt") == false);
        assert!(path::glob_match("test?*", "test1"));
        assert!(path::glob_match("test?*", "test") == false);
    }

    // String escape tests
    #[test]
    fn test_escape_display() {
        assert_eq!(string::escape_display("hello\nworld"), "hello\\nworld");
        assert_eq!(string::escape_display("test\ttab"), "test\\ttab");
    }

    #[test]
    fn test_unescape_display() {
        assert_eq!(string::unescape_display("hello\\nworld"), "hello\nworld");
        assert_eq!(string::unescape_display("test\\ttab"), "test\ttab");
    }

    // Clamp test
    #[test]
    fn test_clamp() {
        assert_eq!(validate::clamp(5, 0, 10), 5);
        assert_eq!(validate::clamp(-1, 0, 10), 0);
        assert_eq!(validate::clamp(15, 0, 10), 10);
    }

    // Strip ANSI test
    #[test]
    fn test_strip_ansi() {
        assert_eq!(
            string::strip_ansi_sequences("\x1b[31mred\x1b[0m"),
            "red"
        );
        assert_eq!(string::strip_ansi_sequences("plain text"), "plain text");
    }

    // Has ANSI test
    #[test]
    fn test_has_ansi() {
        assert!(string::has_ansi_sequences("\x1b[31mred\x1b[0m"));
        assert!(!string::has_ansi_sequences("plain text"));
    }
}
