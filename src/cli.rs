use crate::config::theme::{available_themes, load_theme, save_theme, ThemeConfig};
use anyhow::{Context, Result};

fn normalize(name: &str) -> String {
    name.to_lowercase().replace(['-', ' ', '_'], "")
}

/// Resolve a theme by name: check built-ins first, then themes directory.
fn resolve_theme(name: &str) -> Result<ThemeConfig> {
    let normalized = normalize(name);
    for t in available_themes() {
        if normalize(&t.name) == normalized {
            return Ok(t);
        }
    }
    let themes_dir = crate::config::Config::themes_dir()
        .context("Could not determine themes directory")?;
    let candidate = themes_dir.join(format!("{}.toml", name));
    if candidate.exists() {
        return load_theme(&candidate)
            .with_context(|| format!("Failed to load theme from {:?}", candidate));
    }
    anyhow::bail!(
        "Theme '{}' not found. Use 'terrust theme list' to see available themes.",
        name
    )
}

/// List available themes (built-in + filesystem).
pub fn list_themes() -> Result<()> {
    let builtins = available_themes();
    println!("Built-in themes:");
    for t in &builtins {
        println!("  {} — {}", t.name, t.author);
    }

    let themes_dir = crate::config::Config::themes_dir();
    if let Some(dir) = themes_dir {
        if dir.exists() {
            let mut user_themes: Vec<_> = Vec::new();
            for entry in std::fs::read_dir(&dir)
                .with_context(|| format!("Failed to read themes dir {:?}", dir))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "toml").unwrap_or(false) {
                    if let Ok(t) = load_theme(&path) {
                        user_themes.push((path, t));
                    }
                }
            }
            if !user_themes.is_empty() {
                println!("\nUser themes (from {}):", dir.display());
                for (path, t) in user_themes {
                    let name = path.file_stem().unwrap().to_string_lossy();
                    println!("  {} — {} ({})", name, t.author, path.display());
                }
            }
        }
    }
    Ok(())
}

/// Show a theme's configuration as TOML.
pub fn show_theme(name: &str) -> Result<()> {
    let theme = resolve_theme(name)?;
    let toml_str = toml::to_string_pretty(&theme)
        .with_context(|| "Failed to serialize theme")?;
    println!("{toml_str}");
    Ok(())
}

/// Edit a single field in a theme and save to file.
pub fn edit_theme(name: &str, key_value: &str) -> Result<()> {
    let mut theme = resolve_theme(name)?;

    let (key, value) = key_value
        .split_once('=')
        .context("Format must be <key>=<value>, e.g. background=#ff0000")?;

    let trimmed_value = value.trim().to_string();

    let mut found = false;

    // Top-level fields
    macro_rules! try_set_field {
        ($obj:expr, $field:ident) => {{
            if key == stringify!($field) {
                $obj.$field = trimmed_value.clone();
                found = true;
            }
        }};
    }

    try_set_field!(theme, name);
    try_set_field!(theme, author);
    try_set_field!(theme, background);
    try_set_field!(theme, foreground);
    try_set_field!(theme, cursor);
    try_set_field!(theme, selection);

    // Block fields
    macro_rules! try_set_block {
        ($field:ident) => {{
            if key == stringify!($field) {
                theme.blocks.$field = trimmed_value.clone();
                found = true;
            }
        }};
    }

    try_set_block!(command_bg);
    try_set_block!(command_fg);
    try_set_block!(output_bg);
    try_set_block!(output_fg);
    try_set_block!(ai_bg);
    try_set_block!(ai_fg);
    try_set_block!(border);
    try_set_block!(success);
    try_set_block!(error);

    // Syntax fields
    macro_rules! try_set_syntax {
        ($field:ident) => {{
            if key == stringify!($field) {
                theme.syntax.$field = trimmed_value.clone();
                found = true;
            }
        }};
    }

    try_set_syntax!(comment);
    try_set_syntax!(keyword);
    try_set_syntax!(string);
    try_set_syntax!(number);
    try_set_syntax!(function);
    try_set_syntax!(type_color);
    try_set_syntax!(variable);
    try_set_syntax!(operator);
    try_set_syntax!(attribute);

    if !found {
        anyhow::bail!("Unknown key '{}'. Valid keys: name, author, background, foreground, cursor, selection, command_bg, command_fg, output_bg, output_fg, ai_bg, ai_fg, border, success, error, comment, keyword, string, number, function, type_color, variable, operator, attribute", key);
    }

    let themes_dir = crate::config::Config::themes_dir()
        .context("Could not determine themes directory")?;
    std::fs::create_dir_all(&themes_dir)
        .with_context(|| format!("Failed to create themes directory {:?}", themes_dir))?;
    let out_path = themes_dir.join(format!("{}.toml", name));
    save_theme(&theme, &out_path)
        .with_context(|| format!("Failed to save theme to {:?}", out_path))?;
    println!("Updated theme '{}' saved to {:?}", name, out_path);
    Ok(())
}

/// Render a full color swatch preview of a theme.
pub fn preview_theme(name: &str) -> Result<()> {
    let theme = resolve_theme(name)?;

    fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
        if let Ok(rgb) = crate::config::theme::parse_hex_color(hex) {
            rgb
        } else {
            (0, 0, 0)
        }
    }

    fn fg_escape(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    }
    fn bg_escape(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[48;2;{};{};{}m", r, g, b)
    }
    fn swatch(hex: &str) -> String {
        let (r, g, b) = hex_to_rgb(hex);
        // Choose foreground for readability based on luminance
        let lum = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
        let fg = if lum > 150.0 { (0, 0, 0) } else { (255, 255, 255) };
        format!(
            "{bg}{fg}  {hex}  \x1b[0m",
            bg = bg_escape(r, g, b),
            fg = fg_escape(fg.0, fg.1, fg.2),
            hex = hex,
        )
    }
    fn mini_swatch(hex: &str) -> String {
        let (r, g, b) = hex_to_rgb(hex);
        format!("{bg}  \x1b[0m", bg = bg_escape(r, g, b))
    }

    println!("\n  Theme: {} ({})", theme.name, theme.author);
    println!("  {}", "─".repeat(50));

    println!("  Background    {}", swatch(&theme.background));
    println!("  Foreground    {}", swatch(&theme.foreground));
    println!("  Cursor        {}", swatch(&theme.cursor));
    println!("  Selection     {}", swatch(&theme.selection));

    println!("\n  ANSI Colors:");
    let names = ["Black", "Red", "Green", "Yellow", "Blue", "Magenta", "Cyan", "White"];
    for (i, hex) in theme.ansi.iter().enumerate() {
        println!("    {:<13} {}", names[i], swatch(hex));
    }

    println!("\n  Bright ANSI Colors:");
    for (i, hex) in theme.bright_ansi.iter().enumerate() {
        println!("    Bright {:<8} {}", names[i], swatch(hex));
    }

    // Compact ANSI palette row
    print!("\n  Palette: ");
    for hex in &theme.ansi {
        print!("{}", mini_swatch(hex));
    }
    print!("  ");
    for hex in &theme.bright_ansi {
        print!("{}", mini_swatch(hex));
    }
    println!("\x1b[0m");

    println!("\n  Block Colors:");
    println!("    Command      bg={}  fg={}", mini_swatch(&theme.blocks.command_bg), mini_swatch(&theme.blocks.command_fg));
    println!("    Output       bg={}  fg={}", mini_swatch(&theme.blocks.output_bg), mini_swatch(&theme.blocks.output_fg));
    println!("    AI           bg={}  fg={}", mini_swatch(&theme.blocks.ai_bg), mini_swatch(&theme.blocks.ai_fg));
    println!("    Border       {}", swatch(&theme.blocks.border));
    println!("    Success      {}", swatch(&theme.blocks.success));
    println!("    Error        {}", swatch(&theme.blocks.error));

    println!("\n  Syntax Colors:");
    let syntabs = ["Comment", "Keyword", "String", "Number", "Function", "Type", "Variable", "Operator", "Attribute"];
    let synvals = [
        &theme.syntax.comment, &theme.syntax.keyword, &theme.syntax.string,
        &theme.syntax.number, &theme.syntax.function, &theme.syntax.type_color,
        &theme.syntax.variable, &theme.syntax.operator, &theme.syntax.attribute,
    ];
    for (i, hex) in synvals.iter().enumerate() {
        println!("    {:<10} {}", syntabs[i], swatch(hex));
    }
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_theme_builtin() {
        let theme = resolve_theme("tokyo-night").unwrap();
        assert_eq!(theme.name, "Tokyo Night");
    }

    #[test]
    fn test_resolve_theme_builtin_case_insensitive() {
        let theme = resolve_theme("TOKYO-NIGHT").unwrap();
        assert_eq!(theme.name, "Tokyo Night");
    }

    #[test]
    fn test_resolve_theme_not_found() {
        let result = resolve_theme("nonexistent-theme-xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_list_themes() {
        let result = list_themes();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_theme_builtin() {
        let output = std::panic::catch_unwind(|| {
            show_theme("tokyo-night").unwrap();
        });
        // Should not panic; output goes to stdout
        assert!(output.is_ok());
    }

    #[test]
    fn test_show_theme_not_found() {
        let result = show_theme("nonexistent-theme-xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_edit_theme_invalid_key() {
        let result = edit_theme("tokyo-night", "invalid_key=#ff0000");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Unknown key"));
    }

    #[test]
    fn test_edit_theme_invalid_format() {
        let result = edit_theme("tokyo-night", "noequalsign");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be"));
    }

    #[test]
    fn test_preview_theme_builtin() {
        let output = std::panic::catch_unwind(|| {
            preview_theme("catppuccin-mocha").unwrap();
        });
        assert!(output.is_ok());
    }

    #[test]
    fn test_preview_theme_not_found() {
        let result = preview_theme("nonexistent-theme-xyz");
        assert!(result.is_err());
    }
}
