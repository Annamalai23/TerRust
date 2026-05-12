//! Terminal cell representation

use super::color::Color;
use ratatui::style::Modifier;

/// Represents a single character cell in the terminal grid
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    /// The character to display
    pub character: char,
    /// Foreground color
    pub foreground: Option<Color>,
    /// Background color
    pub background: Option<Color>,
    /// Character attributes (bold, italic, underline, etc.)
    pub attributes: Attributes,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            character: ' ',
            foreground: None,
            background: None,
            attributes: Attributes::default(),
        }
    }
}

/// Character attributes (SGR - Select Graphic Rendition)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Attributes {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub crossed_out: bool,
}

impl Attributes {
    /// Convert to ratatui style modifier
    pub fn to_ratatui_modifier(&self) -> Modifier {
        let mut modifier = Modifier::empty();
        if self.bold { modifier = modifier.union(Modifier::BOLD); }
        if self.dim { modifier = modifier.union(Modifier::DIM); }
        if self.italic { modifier = modifier.union(Modifier::ITALIC); }
        if self.underline { modifier = modifier.union(Modifier::UNDERLINED); }
        if self.blink { modifier = modifier.union(Modifier::SLOW_BLINK); }
        if self.reverse { modifier = modifier.union(Modifier::REVERSED); }
        if self.hidden { modifier = modifier.union(Modifier::HIDDEN); }
        if self.crossed_out { modifier = modifier.union(Modifier::CROSSED_OUT); }
        modifier
    }
}
