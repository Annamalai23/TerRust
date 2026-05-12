//! Reusable UI components for TerRust
//!
//! Provides common UI building blocks like layouts, alignments,
//! and styled text components.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Justified,
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::Left
    }
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub alignment: Alignment,
    pub spacing: u16,
    pub padding: u16,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            alignment: Alignment::Left,
            spacing: 1,
            padding: 0,
        }
    }
}

impl Layout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    pub fn align_text(&self, text: &str, width: u16) -> String {
        let text_width = unicode_width::UnicodeWidthStr::width(text) as u16;
        if text_width >= width {
            return text.to_string();
        }

        let padding = width - text_width;
        match self.alignment {
            Alignment::Left => format!("{}{}", text, " ".repeat(padding as usize)),
            Alignment::Right => format!("{}{}", " ".repeat(padding as usize), text),
            Alignment::Center => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!(
                    "{}{}{}",
                    " ".repeat(left_pad as usize),
                    text,
                    " ".repeat(right_pad as usize)
                )
            }
            Alignment::Justified => text.to_string(),
        }
    }

    pub fn align_lines<'a>(&self, lines: Vec<&'a str>, width: u16) -> Vec<String> {
        lines
            .into_iter()
            .map(|line| self.align_text(line, width))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct StyledText {
    pub parts: Vec<StyledSegment>,
}

#[derive(Debug, Clone)]
pub struct StyledSegment {
    pub text: String,
    pub style: Style,
}

impl StyledText {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            parts: vec![StyledSegment {
                text: text.into(),
                style: Style::default(),
            }],
        }
    }

    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self {
            parts: vec![StyledSegment {
                text: text.into(),
                style,
            }],
        }
    }

    pub fn push(mut self, text: impl Into<String>, style: Style) -> Self {
        self.parts.push(StyledSegment {
            text: text.into(),
            style,
        });
        self
    }

    pub fn push_plain(mut self, text: impl Into<String>) -> Self {
        self.parts.push(StyledSegment {
            text: text.into(),
            style: Style::default(),
        });
        self
    }

    pub fn push_bold(mut self, text: impl Into<String>) -> Self {
        self.parts.push(StyledSegment {
            text: text.into(),
            style: Style::default().add_modifier(Modifier::BOLD),
        });
        self
    }

    pub fn push_italic(mut self, text: impl Into<String>) -> Self {
        self.parts.push(StyledSegment {
            text: text.into(),
            style: Style::default().add_modifier(Modifier::ITALIC),
        });
        self
    }

    pub fn push_fg(mut self, text: impl Into<String>, color: Color) -> Self {
        self.parts.push(StyledSegment {
            text: text.into(),
            style: Style::default().fg(color),
        });
        self
    }

    pub fn push_bg(mut self, text: impl Into<String>, color: Color) -> Self {
        self.parts.push(StyledSegment {
            text: text.into(),
            style: Style::default().bg(color),
        });
        self
    }

    pub fn to_spans(&self) -> Vec<Span<'_>> {
        self.parts
            .iter()
            .map(|seg| Span::styled(seg.text.as_str(), seg.style))
            .collect()
    }

    pub fn to_lines(&self) -> Vec<Line<'_>> {
        vec![Line::from(self.to_spans())]
    }

    pub fn len(&self) -> usize {
        self.parts.iter().map(|p| p.text.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty() || self.parts.iter().all(|p| p.text.is_empty())
    }
}

impl Default for StyledText {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub current: u64,
    pub total: u64,
    pub width: u16,
    pub fill_char: char,
    pub empty_char: char,
    pub show_percentage: bool,
    pub style: Style,
}

impl ProgressBar {
    pub fn new(current: u64, total: u64, width: u16) -> Self {
        Self {
            current,
            total,
            width,
            fill_char: '█',
            empty_char: '░',
            show_percentage: true,
            style: Style::default(),
        }
    }

    pub fn with_fill_char(mut self, char: char) -> Self {
        self.fill_char = char;
        self
    }

    pub fn with_empty_char(mut self, char: char) -> Self {
        self.empty_char = char;
        self
    }

    pub fn with_show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn set_progress(&mut self, current: u64, total: u64) {
        self.current = current;
        self.total = total;
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.current as f64 / self.total as f64) * 100.0
    }

    pub fn filled_width(&self) -> u16 {
        if self.total == 0 {
            return 0;
        }
        ((self.current as f64 / self.total as f64) * (self.width - 1) as f64) as u16
    }

    pub fn render(&self) -> String {
        let filled = self.filled_width();
        let empty = self.width.saturating_sub(filled).saturating_sub(1);

        let percentage_str = if self.show_percentage {
            format!(" {:.0}% ", self.percentage())
        } else {
            String::new()
        };

        format!(
            "{}{}{}",
            self.fill_char.to_string().repeat(filled as usize),
            self.empty_char.to_string().repeat(empty as usize),
            percentage_str
        )
    }
}

#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub segments: Vec<String>,
    pub separator: String,
}

impl Breadcrumb {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            separator: " > ".to_string(),
        }
    }

    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    pub fn push(mut self, segment: impl Into<String>) -> Self {
        self.segments.push(segment.into());
        self
    }

    pub fn pop(&mut self) -> Option<String> {
        self.segments.pop()
    }

    pub fn clear(&mut self) {
        self.segments.clear();
    }

    pub fn render(&self) -> String {
        if self.segments.is_empty() {
            return String::new();
        }
        self.segments.join(&self.separator)
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }
}

impl Default for Breadcrumb {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Component;

impl Component {
    pub fn title_block(title: &str, width: u16) -> Vec<Line<'static>> {
        let inner_width = width.saturating_sub(4);
        let title_len = unicode_width::UnicodeWidthStr::width(title) as u16;

        if title_len >= inner_width {
            return vec![Line::from(title.to_string())];
        }

        let padding = (inner_width - title_len) / 2;
        let left_pad = "─".repeat(padding as usize);
        let right_pad = "─".repeat(inner_width as usize - title_len as usize - padding as usize);

        vec![Line::from(vec![
            Span::raw("┌"),
            Span::raw(left_pad),
            Span::styled(
                title.to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(right_pad),
            Span::raw("┐"),
        ])]
    }

    pub fn separator(width: u16, style: &str) -> String {
        match style {
            "single" => "─".repeat(width as usize),
            "double" => "═".repeat(width as usize),
            "dotted" => "·".repeat(width as usize),
            "stars" => "*".repeat(width as usize),
            _ => "─".repeat(width as usize),
        }
    }

    pub fn empty_line(width: u16) -> String {
        " ".repeat(width as usize)
    }

    pub fn truncated_text(text: &str, max_width: u16, ellipsis: &str) -> String {
        let text_width = unicode_width::UnicodeWidthStr::width(text) as u16;
        if text_width <= max_width {
            return text.to_string();
        }

        let ellipsis_width = unicode_width::UnicodeWidthStr::width(ellipsis) as u16;
        if max_width <= ellipsis_width {
            return text.chars().take(max_width as usize).collect();
        }

        let avail_width = max_width - ellipsis_width;
        let mut used_width = 0;
        let mut result = String::new();
        for c in text.chars() {
            let width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0) as u16;
            if used_width + width > avail_width {
                break;
            }
            used_width += width;
            result.push(c);
        }

        format!("{}{}", result, ellipsis)
    }

    pub fn right_align_text(text: &str, width: u16) -> String {
        let text_width = unicode_width::UnicodeWidthStr::width(text) as u16;
        if text_width >= width {
            return text.to_string();
        }
        format!("{}{}", " ".repeat((width - text_width) as usize), text)
    }

    pub fn center_text(text: &str, width: u16) -> String {
        let text_width = unicode_width::UnicodeWidthStr::width(text) as u16;
        if text_width >= width {
            return text.to_string();
        }
        let padding = (width - text_width) / 2;
        format!(
            "{}{}{}",
            " ".repeat(padding as usize),
            text,
            " ".repeat((width - text_width - padding) as usize)
        )
    }

    pub fn status_indicator(status: &str, ok: bool) -> String {
        if ok {
            format!("\x1b[32m✓\x1b[0m {}", status)
        } else {
            format!("\x1b[31m✗\x1b[0m {}", status)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_align_left() {
        let layout = Layout::new().with_alignment(Alignment::Left);
        let result = layout.align_text("hello", 10);
        assert_eq!(result, "hello     ");
    }

    #[test]
    fn test_layout_align_right() {
        let layout = Layout::new().with_alignment(Alignment::Right);
        let result = layout.align_text("hello", 10);
        assert_eq!(result, "     hello");
    }

    #[test]
    fn test_layout_align_center() {
        let layout = Layout::new().with_alignment(Alignment::Center);
        let result = layout.align_text("hello", 10);
        assert_eq!(result, "  hello   ");
    }

    #[test]
    fn test_styled_text() {
        let text = StyledText::new()
            .push_plain("Hello ")
            .push_bold("World")
            .push_fg("!", Color::Red);

        assert_eq!(text.len(), 12);
        assert_eq!(text.parts.len(), 3);
    }

    #[test]
    fn test_progress_bar() {
        let progress = ProgressBar::new(50, 100, 40);
        assert_eq!(progress.percentage(), 50.0);
        assert_eq!(progress.filled_width(), 19);
    }

    #[test]
    fn test_progress_bar_zero_total() {
        let progress = ProgressBar::new(0, 0, 40);
        assert_eq!(progress.percentage(), 0.0);
        assert_eq!(progress.filled_width(), 0);
    }

    #[test]
    fn test_breadcrumb() {
        let breadcrumb = Breadcrumb::new()
            .push("Home")
            .push("Projects")
            .push("TerRust");

        assert_eq!(breadcrumb.len(), 3);
        assert_eq!(breadcrumb.render(), "Home > Projects > TerRust");
    }

    #[test]
    fn test_breadcrumb_pop() {
        let mut breadcrumb = Breadcrumb::new().push("Home").push("Projects");
        assert_eq!(breadcrumb.pop(), Some("Projects".to_string()));
        assert_eq!(breadcrumb.len(), 1);
    }

    #[test]
    fn test_truncated_text() {
        assert_eq!(
            Component::truncated_text("hello world", 8, "..."),
            "hello..."
        );
        assert_eq!(Component::truncated_text("hello", 10, "..."), "hello");
        assert_eq!(Component::truncated_text("hi", 2, "..."), "hi");
        assert_eq!(Component::truncated_text("hello", 2, ".."), "he");
    }

    #[test]
    fn test_right_align() {
        assert_eq!(Component::right_align_text("hi", 6), "    hi");
        assert_eq!(Component::right_align_text("hello", 5), "hello");
    }

    #[test]
    fn test_center_text() {
        assert_eq!(Component::center_text("hi", 6), "  hi  ");
        assert_eq!(Component::center_text("hello", 5), "hello");
    }

    #[test]
    fn test_separator() {
        assert_eq!(Component::separator(5, "single"), "─────");
        assert_eq!(Component::separator(5, "double"), "═════");
    }

    #[test]
    fn test_status_indicator() {
        let ok = Component::status_indicator("Success", true);
        let fail = Component::status_indicator("Failed", false);
        assert!(ok.contains("✓"));
        assert!(fail.contains("✗"));
    }
}
