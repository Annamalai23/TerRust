//! Rendering helpers for terminal grids and block output.

use crate::config::ThemeConfig;
use crate::terminal::{Cell, Grid};
use crate::ui::blocks::{get_block_style, Block, BlockManager};
use ratatui::text::{Line, Span};

/// Convert a terminal cell row into a ratatui line while preserving basic style.
pub fn cells_to_line(row: &[Cell], fallback_fg: ratatui::style::Color) -> Line<'static> {
    let trimmed_len = row
        .iter()
        .rposition(|cell| cell.character != ' ')
        .map(|idx| idx + 1)
        .unwrap_or(0);

    if trimmed_len == 0 {
        return Line::from(String::new());
    }

    let spans = row[..trimmed_len]
        .iter()
        .map(|cell| {
            let mut style = ratatui::style::Style::default().fg(cell
                .foreground
                .map(|c| c.to_ratatui_color())
                .unwrap_or(fallback_fg));

            if let Some(background) = cell.background {
                style = style.bg(background.to_ratatui_color());
            }

            style = style.add_modifier(cell.attributes.to_ratatui_modifier());
            Span::styled(cell.character.to_string(), style)
        })
        .collect::<Vec<_>>();

    Line::from(spans)
}

/// Convert the live terminal grid into display lines.
pub fn grid_to_lines(grid: &Grid, theme: &ThemeConfig) -> Vec<Line<'static>> {
    let fg = theme.fg();
    grid.cells
        .iter()
        .map(|row| cells_to_line(row, fg))
        .collect()
}

/// Convert a block into display lines with a compact visual boundary.
pub fn block_to_lines(block: &Block, theme: &ThemeConfig, width: u16) -> Vec<Line<'static>> {
    let style = get_block_style(block.block_type, theme);
    let title = block.title.as_deref().unwrap_or(match block.block_type {
        crate::ui::blocks::BlockType::Command => "Command",
        crate::ui::blocks::BlockType::Output => "Output",
        crate::ui::blocks::BlockType::Error => "Error",
        crate::ui::blocks::BlockType::AI => "AI",
        crate::ui::blocks::BlockType::System => "System",
    });

    let mut lines = Vec::new();
    let usable_width = width.max(4) as usize;
    let title_text = format!(" {} ", title);
    let remaining = usable_width.saturating_sub(title_text.chars().count() + 1);
    let header = format!(
        "{}{}{}",
        style.border_chars.tee_right,
        title_text,
        style.border_chars.horizontal.repeat(remaining)
    );
    lines.push(Line::from(Span::styled(
        header,
        ratatui::style::Style::default().fg(style.border_color),
    )));

    if block.collapsed {
        lines.push(Line::from(Span::styled(
            "  [collapsed]".to_string(),
            ratatui::style::Style::default().fg(style.foreground_color),
        )));
        return lines;
    }

    if block.content.is_empty() {
        lines.push(Line::from(String::new()));
        return lines;
    }

    for row in &block.content {
        lines.push(cells_to_line(row, style.foreground_color));
    }

    lines
}

/// Convert all blocks into one scrollable list, newest content shown at the bottom.
pub fn blocks_to_lines(
    manager: &BlockManager,
    theme: &ThemeConfig,
    width: u16,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for (idx, block) in manager.blocks().iter().enumerate() {
        if idx > 0 {
            lines.push(Line::from(String::new()));
        }
        lines.extend(block_to_lines(block, theme, width));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::Cell;
    use crate::ui::blocks::{Block, BlockManager, BlockType};

    #[test]
    fn trims_trailing_blank_cells() {
        let mut row = vec![Cell::default(); 5];
        row[0].character = 'o';
        row[1].character = 'k';

        let line = cells_to_line(&row, ratatui::style::Color::White);
        assert_eq!(line.width(), 2);
    }

    #[test]
    fn renders_block_title_and_content() {
        let mut row = vec![Cell::default(); 3];
        row[0].character = 'h';
        row[1].character = 'i';

        let block = Block::new(BlockType::Output)
            .with_title("Result")
            .with_content(vec![row]);

        let lines = block_to_lines(&block, &ThemeConfig::default(), 20);
        assert!(lines.len() >= 2);
        assert!(format!("{:?}", lines[0]).contains("Result"));
    }

    #[test]
    fn renders_all_blocks_in_order() {
        let mut manager = BlockManager::new(10, 100);
        manager.add_command_block("pwd");
        manager.add_block(Block::new(BlockType::Output).with_title("Output"));

        let lines = blocks_to_lines(&manager, &ThemeConfig::default(), 30);
        assert!(lines.len() >= 3);
    }
}
