//! Terminal color representation

/// Terminal color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    /// Named color (black, red, green, etc.)
    Named(ColorName),
    /// ANSI index (0-255)
    Index(u8),
    /// RGB color (0-255 for each component)
    Rgb(u8, u8, u8),
}

/// Named ANSI colors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorName {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Default,
}

impl Color {
    /// Parse a color from a hex string (#RRGGBB, #RGB, #RRGGBBAA)
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Color::Rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::Rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::Rgb(r, g, b))
            }
            _ => None,
        }
    }

    /// Parse a color from an ANSI color code (0-255)
    pub fn from_ansi_code(code: u8) -> Self {
        match code {
            0..=15 => Color::Index(code),
            16..=231 => {
                let code = code - 16;
                let r = if code < 216 {
                    let idx = code / 36;
                    if idx < 6 { (idx * 40 + 55) as u8 } else { 255 }
                } else {
                    let gray = (code - 216) * 10 + 8;
                    gray as u8
                };
                let g = if code < 216 {
                    let idx = (code % 36) / 6;
                    if idx < 6 { (idx * 40 + 55) as u8 } else { 255 }
                } else {
                    r
                };
                let b = if code < 216 {
                    let idx = code % 6;
                    if idx < 6 { (idx * 40 + 55) as u8 } else { 255 }
                } else {
                    r
                };
                Color::Rgb(r, g, b)
            }
            232..=255 => {
                let gray = ((code - 232) as u8 * 10) + 8;
                Color::Rgb(gray, gray, gray)
            }
        }
    }

    /// Convert to a ratatui color for UI rendering
    pub fn to_ratatui_color(&self) -> ratatui::style::Color {
        match self {
            Color::Named(name) => match name {
                ColorName::Black => ratatui::style::Color::Black,
                ColorName::Red => ratatui::style::Color::Red,
                ColorName::Green => ratatui::style::Color::Green,
                ColorName::Yellow => ratatui::style::Color::Yellow,
                ColorName::Blue => ratatui::style::Color::Blue,
                ColorName::Magenta => ratatui::style::Color::Magenta,
                ColorName::Cyan => ratatui::style::Color::Cyan,
                ColorName::White => ratatui::style::Color::White,
                ColorName::BrightBlack => ratatui::style::Color::DarkGray,
                ColorName::BrightRed => ratatui::style::Color::LightRed,
                ColorName::BrightGreen => ratatui::style::Color::LightGreen,
                ColorName::BrightYellow => ratatui::style::Color::LightYellow,
                ColorName::BrightBlue => ratatui::style::Color::LightBlue,
                ColorName::BrightMagenta => ratatui::style::Color::LightMagenta,
                ColorName::BrightCyan => ratatui::style::Color::LightCyan,
                ColorName::BrightWhite => ratatui::style::Color::White,
                ColorName::Default => ratatui::style::Color::Reset,
            },
            Color::Index(idx) => ratatui::style::Color::Indexed(*idx),
            Color::Rgb(r, g, b) => ratatui::style::Color::Rgb(*r, *g, *b),
        }
    }
}
