//! Theme configuration for TerRust
//!
//! Defines color schemes and styling for the terminal UI.

use ratatui::style::{Color, Style};
use serde::de::Error as DeError;
use serde::{Deserialize, Serialize};

/// Theme configuration for TerRust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme name
    pub name: String,
    
    /// Theme author
    #[serde(default)]
    pub author: String,

    /// Background color
    #[serde(default = "default_bg")]
    pub background: String,

    /// Foreground color
    #[serde(default = "default_fg")]
    pub foreground: String,

    /// Cursor color
    #[serde(default = "default_cursor")]
    pub cursor: String,

    /// Selection color
    #[serde(default = "default_selection")]
    pub selection: String,

    /// ANSI color palette
    #[serde(default = "default_ansi_colors")]
    pub ansi: Vec<String>,

    /// Bright ANSI colors
    #[serde(default = "default_bright_ansi_colors")]
    pub bright_ansi: Vec<String>,

    /// Block-specific colors
    #[serde(default)]
    pub blocks: BlockColors,

    /// Syntax highlighting colors
    #[serde(default)]
    pub syntax: SyntaxColors,
}

fn default_bg() -> String {
    "#1a1b26".to_string()
}

fn default_fg() -> String {
    "#c0caf5".to_string()
}

fn default_cursor() -> String {
    "#f7768e".to_string()
}

fn default_selection() -> String {
    "#363746".to_string()
}

fn default_ansi_colors() -> Vec<String> {
    vec![
        "#45475a", // black
        "#f38ba8", // red
        "#a6e3a1", // green
        "#f9e2af", // yellow
        "#89b4fa", // blue
        "#f5c2e7", // magenta
        "#94e2d5", // cyan
        "#a9b1d6", // white
    ].into_iter().map(String::from).collect()
}

fn default_bright_ansi_colors() -> Vec<String> {
    vec![
        "#585b70", // bright black
        "#f38ba8", // bright red
        "#a6e3a1", // bright green
        "#f9e2af", // bright yellow
        "#89b4fa", // bright blue
        "#f5c2e7", // bright magenta
        "#94e2d5", // bright cyan
        "#a6adc8", // bright white
    ].into_iter().map(String::from).collect()
}

/// Block-specific colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockColors {
    /// Command block background
    #[serde(default = "default_command_bg")]
    pub command_bg: String,

    /// Command block foreground
    #[serde(default = "default_command_fg")]
    pub command_fg: String,

    /// Output block background
    #[serde(default = "default_output_bg")]
    pub output_bg: String,

    /// Output block foreground
    #[serde(default = "default_output_fg")]
    pub output_fg: String,

    /// AI block background
    #[serde(default = "default_ai_bg")]
    pub ai_bg: String,

    /// AI block foreground
    #[serde(default = "default_ai_fg")]
    pub ai_fg: String,

    /// Border color
    #[serde(default = "default_border")]
    pub border: String,

    /// Success color (for exit code 0)
    #[serde(default = "default_success")]
    pub success: String,

    /// Error color (for non-zero exit codes)
    #[serde(default = "default_error")]
    pub error: String,
}

fn default_command_bg() -> String {
    "#313244".to_string()
}

fn default_command_fg() -> String {
    "#cdd6f4".to_string()
}

fn default_output_bg() -> String {
    "#1e1e2e".to_string()
}

fn default_output_fg() -> String {
    "#cdd6f4".to_string()
}

fn default_ai_bg() -> String {
    "#313244".to_string()
}

fn default_ai_fg() -> String {
    "#cba6f7".to_string()
}

fn default_border() -> String {
    "#313244".to_string()
}

fn default_success() -> String {
    "#a6e3a1".to_string()
}

fn default_error() -> String {
    "#f38ba8".to_string()
}

impl Default for BlockColors {
    fn default() -> Self {
        Self {
            command_bg: default_command_bg(),
            command_fg: default_command_fg(),
            output_bg: default_output_bg(),
            output_fg: default_output_fg(),
            ai_bg: default_ai_bg(),
            ai_fg: default_ai_fg(),
            border: default_border(),
            success: default_success(),
            error: default_error(),
        }
    }
}

/// Syntax highlighting colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    /// Comment color
    #[serde(default = "default_comment")]
    pub comment: String,

    /// Keyword color
    #[serde(default = "default_keyword")]
    pub keyword: String,

    /// String color
    #[serde(default = "default_string")]
    pub string: String,

    /// Number color
    #[serde(default = "default_number")]
    pub number: String,

    /// Function color
    #[serde(default = "default_function")]
    pub function: String,

    /// Type color
    #[serde(default = "default_type")]
    pub type_color: String,

    /// Variable color
    #[serde(default = "default_variable")]
    pub variable: String,

    /// Operator color
    #[serde(default = "default_operator")]
    pub operator: String,

    /// Attribute color
    #[serde(default = "default_attribute")]
    pub attribute: String,
}

fn default_comment() -> String {
    "#6c7086".to_string()
}

fn default_keyword() -> String {
    "#7aa2f7".to_string()
}

fn default_string() -> String {
    "#9ece6a".to_string()
}

fn default_number() -> String {
    "#f9e2af".to_string()
}

fn default_function() -> String {
    "#89b4fa".to_string()
}

fn default_type() -> String {
    "#f5c2e7".to_string()
}

fn default_variable() -> String {
    "#cdd6f4".to_string()
}

fn default_operator() -> String {
    "#f7768e".to_string()
}

fn default_attribute() -> String {
    "#94e2d5".to_string()
}

impl Default for SyntaxColors {
    fn default() -> Self {
        Self {
            comment: default_comment(),
            keyword: default_keyword(),
            string: default_string(),
            number: default_number(),
            function: default_function(),
            type_color: default_type(),
            variable: default_variable(),
            operator: default_operator(),
            attribute: default_attribute(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            author: "TerRust Team".to_string(),
            background: default_bg(),
            foreground: default_fg(),
            cursor: default_cursor(),
            selection: default_selection(),
            ansi: default_ansi_colors(),
            bright_ansi: default_bright_ansi_colors(),
            blocks: BlockColors::default(),
            syntax: SyntaxColors::default(),
        }
    }
}

impl ThemeConfig {
    /// Create a Tokyo Night theme
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".to_string(),
            author: "zatchheems".to_string(),
            background: "#1a1b26".to_string(),
            foreground: "#c0caf5".to_string(),
            cursor: "#f7768e".to_string(),
            selection: "#363746".to_string(),
            ansi: vec![
                "#45475a", // black
                "#f38ba8", // red
                "#a6e3a1", // green
                "#f9e2af", // yellow
                "#89b4fa", // blue
                "#f5c2e7", // magenta
                "#94e2d5", // cyan
                "#a9b1d6", // white
            ].into_iter().map(String::from).collect(),
            bright_ansi: vec![
                "#585b70", // bright black
                "#f38ba8", // bright red
                "#a6e3a1", // bright green
                "#f9e2af", // bright yellow
                "#89b4fa", // bright blue
                "#f5c2e7", // bright magenta
                "#94e2d5", // bright cyan
                "#a6adc8", // bright white
            ].into_iter().map(String::from).collect(),
            blocks: BlockColors {
                command_bg: "#313244".to_string(),
                command_fg: "#cdd6f4".to_string(),
                output_bg: "#1e1e2e".to_string(),
                output_fg: "#cdd6f4".to_string(),
                ai_bg: "#313244".to_string(),
                ai_fg: "#cba6f7".to_string(),
                border: "#313244".to_string(),
                success: "#a6e3a1".to_string(),
                error: "#f38ba8".to_string(),
            },
            syntax: SyntaxColors {
                comment: "#6c7086".to_string(),
                keyword: "#7aa2f7".to_string(),
                string: "#9ece6a".to_string(),
                number: "#f9e2af".to_string(),
                function: "#89b4fa".to_string(),
                type_color: "#f5c2e7".to_string(),
                variable: "#cdd6f4".to_string(),
                operator: "#f7768e".to_string(),
                attribute: "#94e2d5".to_string(),
            },
        }
    }

    /// Create a Catppuccin Mocha theme
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha".to_string(),
            author: "Catppuccin Team".to_string(),
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),
            cursor: "#f5e0dc".to_string(),
            selection: "#313244".to_string(),
            ansi: vec![
                "#45475a", // black
                "#f38ba8", // red
                "#a6e3a1", // green
                "#f9e2af", // yellow
                "#89b4fa", // blue
                "#f5c2e7", // magenta
                "#94e2d5", // cyan
                "#bac2de", // white
            ].into_iter().map(String::from).collect(),
            bright_ansi: vec![
                "#585b70", // bright black
                "#f38ba8", // bright red
                "#a6e3a1", // bright green
                "#f9e2af", // bright yellow
                "#89b4fa", // bright blue
                "#f5c2e7", // bright magenta
                "#94e2d5", // bright cyan
                "#a6adc8", // bright white
            ].into_iter().map(String::from).collect(),
            blocks: BlockColors {
                command_bg: "#313244".to_string(),
                command_fg: "#cdd6f4".to_string(),
                output_bg: "#1e1e2e".to_string(),
                output_fg: "#cdd6f4".to_string(),
                ai_bg: "#313244".to_string(),
                ai_fg: "#cba6f7".to_string(),
                border: "#313244".to_string(),
                success: "#a6e3a1".to_string(),
                error: "#f38ba8".to_string(),
            },
            syntax: SyntaxColors {
                comment: "#6c7086".to_string(),
                keyword: "#7aa2f7".to_string(),
                string: "#9ece6a".to_string(),
                number: "#f9e2af".to_string(),
                function: "#89b4fa".to_string(),
                type_color: "#f5c2e7".to_string(),
                variable: "#cdd6f4".to_string(),
                operator: "#f38ba8".to_string(),
                attribute: "#94e2d5".to_string(),
            },
        }
    }

    /// Create a Dracula theme
    pub fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            author: "Dracula Team".to_string(),
            background: "#282a36".to_string(),
            foreground: "#f8f8f2".to_string(),
            cursor: "#ff79c6".to_string(),
            selection: "#44475a".to_string(),
            ansi: vec![
                "#21222c", // black
                "#ff5555", // red
                "#50fa7b", // green
                "#f1fa8c", // yellow
                "#bd93f9", // blue
                "#ff79c6", // magenta
                "#8be9fd", // cyan
                "#f8f8f2", // white
            ].into_iter().map(String::from).collect(),
            bright_ansi: vec![
                "#6272a4", // bright black
                "#ff6e6e", // bright red
                "#69ff94", // bright green
                "#ffffa5", // bright yellow
                "#d6acff", // bright blue
                "#ff92df", // bright magenta
                "#a4ffff", // bright cyan
                "#ffffff", // bright white
            ].into_iter().map(String::from).collect(),
            blocks: BlockColors {
                command_bg: "#44475a".to_string(),
                command_fg: "#f8f8f2".to_string(),
                output_bg: "#282a36".to_string(),
                output_fg: "#f8f8f2".to_string(),
                ai_bg: "#44475a".to_string(),
                ai_fg: "#bd93f9".to_string(),
                border: "#44475a".to_string(),
                success: "#50fa7b".to_string(),
                error: "#ff5555".to_string(),
            },
            syntax: SyntaxColors {
                comment: "#6272a4".to_string(),
                keyword: "#bd93f9".to_string(),
                string: "#f1fa8c".to_string(),
                number: "#ffb86c".to_string(),
                function: "#8be9fd".to_string(),
                type_color: "#ff79c6".to_string(),
                variable: "#f8f8f2".to_string(),
                operator: "#ff5555".to_string(),
                attribute: "#50fa7b".to_string(),
            },
        }
    }

    /// Parse a color string into a ratatui Color
    pub fn parse_color(&self, color_str: &str) -> Color {
        if let Ok(rgb) = parse_hex_color(color_str) {
            Color::Rgb(rgb.0, rgb.1, rgb.2)
        } else if let Ok(index) = color_str.parse::<u8>() {
            if index < 8 {
                Color::Indexed(index)
            } else {
                Color::Indexed(index - 8 + 8) // Bright colors
            }
        } else {
            match color_str.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "white" => Color::White,
                "gray" | "grey" => Color::Gray,
                "reset" => Color::Reset,
                _ => Color::White,
            }
        }
    }

    /// Get the background color
    pub fn bg(&self) -> Color {
        self.parse_color(&self.background)
    }

    /// Get the foreground color
    pub fn fg(&self) -> Color {
        self.parse_color(&self.foreground)
    }

    /// Get the cursor color
    pub fn cursor(&self) -> Color {
        self.parse_color(&self.cursor)
    }

    /// Get the selection color
    pub fn selection(&self) -> Color {
        self.parse_color(&self.selection)
    }

    /// Get ANSI colors
    pub fn ansi_colors(&self) -> Vec<Color> {
        self.ansi.iter().map(|c| self.parse_color(c)).collect()
    }

    /// Get bright ANSI colors
    pub fn bright_ansi_colors(&self) -> Vec<Color> {
        self.bright_ansi.iter().map(|c| self.parse_color(c)).collect()
    }

    /// Get command block style
    pub fn command_block_style(&self) -> Style {
        Style::new()
            .bg(self.parse_color(&self.blocks.command_bg))
            .fg(self.parse_color(&self.blocks.command_fg))
    }

    /// Get output block style
    pub fn output_block_style(&self) -> Style {
        Style::new()
            .bg(self.parse_color(&self.blocks.output_bg))
            .fg(self.parse_color(&self.blocks.output_fg))
    }

    /// Get AI block style
    pub fn ai_block_style(&self) -> Style {
        Style::new()
            .bg(self.parse_color(&self.blocks.ai_bg))
            .fg(self.parse_color(&self.blocks.ai_fg))
    }

    /// Get border style
    pub fn border_style(&self) -> Style {
        Style::new().fg(self.parse_color(&self.blocks.border))
    }

    /// Get success style
    pub fn success_style(&self) -> Style {
        Style::new().fg(self.parse_color(&self.blocks.success))
    }

    /// Get error style
    pub fn error_style(&self) -> Style {
        Style::new().fg(self.parse_color(&self.blocks.error))
    }
}

/// Parse a hex color string into RGB components
/// Supports formats: #RGB, #RRGGBB, #RRGGBBAA
pub fn parse_hex_color(color: &str) -> Result<(u8, u8, u8), ()> {
    let clean = color.trim_start_matches('#');
    
    match clean.len() {
        3 => {
            // #RGB format
            let r = u8::from_str_radix(&clean[0..1], 16).map_err(|_| ())? * 17;
            let g = u8::from_str_radix(&clean[1..2], 16).map_err(|_| ())? * 17;
            let b = u8::from_str_radix(&clean[2..3], 16).map_err(|_| ())? * 17;
            Ok((r, g, b))
        }
        6 => {
            // #RRGGBB format
            let r = u8::from_str_radix(&clean[0..2], 16).map_err(|_| ())?;
            let g = u8::from_str_radix(&clean[2..4], 16).map_err(|_| ())?;
            let b = u8::from_str_radix(&clean[4..6], 16).map_err(|_| ())?;
            Ok((r, g, b))
        }
        8 => {
            // #RRGGBBAA format (ignore alpha)
            let r = u8::from_str_radix(&clean[0..2], 16).map_err(|_| ())?;
            let g = u8::from_str_radix(&clean[2..4], 16).map_err(|_| ())?;
            let b = u8::from_str_radix(&clean[4..6], 16).map_err(|_| ())?;
            Ok((r, g, b))
        }
        _ => Err(()),
    }
}

/// Available theme presets
pub fn available_themes() -> Vec<ThemeConfig> {
    vec![
        ThemeConfig::tokyo_night(),
        ThemeConfig::catppuccin_mocha(),
        ThemeConfig::dracula(),
    ]
}

/// Load a theme from a file
pub fn load_theme<P: AsRef<std::path::Path>>(path: P) -> Result<ThemeConfig, toml::de::Error> {
    let content = std::fs::read_to_string(&path)
        .map_err(|_| toml::de::Error::custom("Failed to read theme file"))?;
    toml::from_str(&content)
}

/// Save a theme to a file
pub fn save_theme<P: AsRef<std::path::Path>>(
    theme: &ThemeConfig,
    path: P,
) -> Result<(), std::io::Error> {
    let content = toml::to_string(theme).map_err(|_| std::io::Error::other("Serialization error"))?;
    std::fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_colors() {
        assert_eq!(parse_hex_color("#ff0000"), Ok((255, 0, 0)));
        assert_eq!(parse_hex_color("#00ff00"), Ok((0, 255, 0)));
        assert_eq!(parse_hex_color("#0000ff"), Ok((0, 0, 255)));
        assert_eq!(parse_hex_color("#fff"), Ok((255, 255, 255)));
        assert_eq!(parse_hex_color("#000"), Ok((0, 0, 0)));
    }

    #[test]
    fn test_theme_colors() {
        let theme = ThemeConfig::tokyo_night();
        assert_eq!(theme.name, "Tokyo Night");
        assert_eq!(theme.background, "#1a1b26");
        assert_eq!(theme.foreground, "#c0caf5");
    }

    #[test]
    fn test_theme_serialization() {
        let theme = ThemeConfig::catppuccin_mocha();
        let serialized = toml::to_string(&theme).unwrap();
        let deserialized: ThemeConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(theme.name, deserialized.name);
    }

    #[test]
    fn test_color_parsing() {
        let theme = ThemeConfig::default();
        assert_eq!(theme.parse_color("#ff0000"), Color::Rgb(255, 0, 0));
        assert_eq!(theme.parse_color("red"), Color::Red);
        assert_eq!(theme.parse_color("0"), Color::Indexed(0));
    }
}
