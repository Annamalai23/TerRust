//! Block-based output rendering for TerRust
//!
//! Each command output is displayed in a visually distinct block with
//! rich formatting, clickable links, and easy text copy capabilities.

use crate::config::ThemeConfig;
use crate::terminal::Cell;
use chrono::{DateTime, Local};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Command,
    Output,
    Error,
    AI,
    System,
}

impl BlockType {
    pub fn is_error(&self) -> bool {
        matches!(self, BlockType::Error)
    }

    pub fn is_ai(&self) -> bool {
        matches!(self, BlockType::AI)
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: Uuid,
    pub block_type: BlockType,
    pub title: Option<String>,
    pub content: Vec<Vec<Cell>>,
    pub timestamp: DateTime<Local>,
    pub exit_code: Option<i32>,
    pub start_time: Option<DateTime<Local>>,
    pub duration: Option<std::time::Duration>,
    pub cwd: Option<String>,
    pub shell: Option<String>,
    pub command: Option<String>,
    pub collapsed: bool,
    pub pinned: bool,
    pub selectable: bool,
    pub hyperlinks: HashMap<String, String>,
    pub scroll_offset: u16,
    pub width: u16,
    pub height: u16,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            id: Uuid::new_v4(),
            block_type,
            title: None,
            content: Vec::new(),
            timestamp: Local::now(),
            exit_code: None,
            start_time: None,
            duration: None,
            cwd: None,
            shell: None,
            command: None,
            collapsed: false,
            pinned: false,
            selectable: true,
            hyperlinks: HashMap::new(),
            scroll_offset: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_content(mut self, content: Vec<Vec<Cell>>) -> Self {
        self.content = content;
        self
    }

    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        if code != 0 {
            self.block_type = BlockType::Error;
        }
        self
    }

    pub fn with_command_metadata(
        mut self,
        command: String,
        cwd: String,
        shell: String,
    ) -> Self {
        self.start_time = Some(Local::now());
        self.command = Some(command);
        self.cwd = Some(cwd);
        self.shell = Some(shell);
        self
    }

    pub fn set_duration(&mut self, duration: std::time::Duration) {
        self.duration = Some(duration);
    }

    pub fn pin(&mut self) {
        self.pinned = true;
    }

    pub fn unpin(&mut self) {
        self.pinned = false;
    }

    pub fn toggle_pin(&mut self) {
        self.pinned = !self.pinned;
    }

    pub fn collapse(&mut self) {
        self.collapsed = true;
    }

    pub fn expand(&mut self) {
        self.collapsed = false;
    }

    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub fn add_hyperlink(&mut self, url: String, id: String) {
        self.hyperlinks.insert(id, url);
    }

    pub fn lines(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty() || self.content.iter().all(|row| row.is_empty())
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }

    pub fn scroll_up(&mut self, lines: u16) {
        self.scroll_offset = self.scroll_offset.saturating_add(lines);
    }

    pub fn scroll_down(&mut self, lines: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.content.len().saturating_sub(self.height as usize) as u16;
    }

    pub fn get_display_content(&self) -> Vec<&[Cell]> {
        if self.collapsed {
            return Vec::new();
        }
        self.content
            .iter()
            .skip(self.scroll_offset as usize)
            .take(self.height as usize)
            .map(|row| row.as_slice())
            .collect()
    }
}

#[derive(Clone)]
pub struct BlockManager {
    blocks: Vec<Block>,
    current_block: Option<Uuid>,
    max_blocks: usize,
    #[allow(dead_code)]
    scrollback_limit: usize,
}

impl BlockManager {
    pub fn new(max_blocks: usize, scrollback_limit: usize) -> Self {
        Self {
            blocks: Vec::new(),
            current_block: None,
            max_blocks,
            scrollback_limit,
        }
    }

    pub fn add_block(&mut self, mut block: Block) -> Uuid {
        block.id = Uuid::new_v4();
        let id = block.id;

        if self.blocks.len() >= self.max_blocks {
            if self.blocks.iter().any(|b| b.pinned) {
                return id;
            }
            if let Some(pos) = self.blocks.iter().position(|b| !b.pinned) {
                self.blocks.remove(pos);
            }
        }

        self.blocks.push(block);
        self.current_block = Some(id);
        id
    }

    pub fn add_command_block(&mut self, command: &str) -> Uuid {
        let mut block = Block::new(BlockType::Command).with_title(format!("$ {}", command));
        block.selectable = false;
        self.add_block(block)
    }

    pub fn add_output_block(&mut self) -> Uuid {
        self.add_block(Block::new(BlockType::Output))
    }

    pub fn add_ai_block(&mut self) -> Uuid {
        self.add_block(Block::new(BlockType::AI))
    }

    pub fn add_error_block(&mut self) -> Uuid {
        self.add_block(Block::new(BlockType::Error))
    }

    pub fn add_system_block(&mut self, message: &str) -> Uuid {
        let block = Block::new(BlockType::System).with_title(message.to_string());
        self.add_block(block)
    }

    pub fn get_block(&self, id: Uuid) -> Option<&Block> {
        self.blocks.iter().find(|b| b.id == id)
    }

    pub fn get_block_mut(&mut self, id: Uuid) -> Option<&mut Block> {
        self.blocks.iter_mut().find(|b| b.id == id)
    }

    pub fn get_current_block(&self) -> Option<&Block> {
        self.current_block.and_then(|id| self.get_block(id))
    }

    pub fn get_current_block_mut(&mut self) -> Option<&mut Block> {
        self.current_block.and_then(|id| self.get_block_mut(id))
    }

    pub fn set_current_block(&mut self, id: Uuid) {
        if self.blocks.iter().any(|b| b.id == id) {
            self.current_block = Some(id);
        }
    }

    pub fn next_block(&mut self) -> Option<Uuid> {
        let current = self.current_block?;
        let pos = self.blocks.iter().position(|b| b.id == current)?;
        let next_pos = (pos + 1) % self.blocks.len();
        let next_id = self.blocks[next_pos].id;
        self.current_block = Some(next_id);
        Some(next_id)
    }

    pub fn prev_block(&mut self) -> Option<Uuid> {
        let current = self.current_block?;
        let pos = self.blocks.iter().position(|b| b.id == current)?;
        let prev_pos = if pos == 0 {
            self.blocks.len() - 1
        } else {
            pos - 1
        };
        let prev_id = self.blocks[prev_pos].id;
        self.current_block = Some(prev_id);
        Some(prev_id)
    }

    pub fn pin_current_block(&mut self) -> bool {
        if let Some(block) = self.get_current_block_mut() {
            block.toggle_pin();
            return true;
        }
        false
    }

    pub fn collapse_current_block(&mut self) -> bool {
        if let Some(block) = self.get_current_block_mut() {
            block.toggle_collapse();
            return true;
        }
        false
    }

    pub fn remove_block(&mut self, id: Uuid) -> bool {
        let pos = self.blocks.iter().position(|b| b.id == id);
        if let Some(pos) = pos {
            self.blocks.remove(pos);
            if self.current_block == Some(id) {
                self.current_block = self.blocks.last().map(|b| b.id);
            }
            return true;
        }
        false
    }

    pub fn clear_all(&mut self) {
        self.blocks.retain(|b| b.pinned);
        self.current_block = self.blocks.last().map(|b| b.id);
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn blocks_mut(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }

    pub fn pinned_blocks(&self) -> Vec<&Block> {
        self.blocks.iter().filter(|b| b.pinned).collect()
    }

    pub fn unpinned_blocks(&self) -> Vec<&Block> {
        self.blocks.iter().filter(|b| !b.pinned).collect()
    }

    pub fn error_blocks(&self) -> Vec<&Block> {
        self.blocks
            .iter()
            .filter(|b| b.block_type.is_error())
            .collect()
    }

    pub fn ai_blocks(&self) -> Vec<&Block> {
        self.blocks
            .iter()
            .filter(|b| b.block_type.is_ai())
            .collect()
    }

    pub fn search_blocks(&self, query: &str) -> Vec<(Uuid, Vec<usize>)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for block in &self.blocks {
            let mut line_matches = Vec::new();
            for (line_idx, row) in block.content.iter().enumerate() {
                let line_text: String = row.iter().map(|c| c.character).collect();
                if line_text.to_lowercase().contains(&query_lower) {
                    line_matches.push(line_idx);
                }
            }
            if !line_matches.is_empty() {
                results.push((block.id, line_matches));
            }
        }

        results
    }

    pub fn total_lines(&self) -> usize {
        self.blocks.iter().map(|b| b.lines()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }
}

impl Default for BlockManager {
    fn default() -> Self {
        Self::new(1000, 10000)
    }
}

pub fn get_block_style(block_type: BlockType, theme: &ThemeConfig) -> BlockStyle {
    match block_type {
        BlockType::Command => BlockStyle {
            border_color: theme.border_style().fg.unwrap_or(theme.fg()),
            background_color: theme.command_block_style().bg.unwrap_or(theme.bg()),
            foreground_color: theme.command_block_style().fg.unwrap_or(theme.fg()),
            show_border: true,
            border_chars: BorderChars::single(),
        },
        BlockType::Output => BlockStyle {
            border_color: theme.border_style().fg.unwrap_or(theme.fg()),
            background_color: theme.output_block_style().bg.unwrap_or(theme.bg()),
            foreground_color: theme.output_block_style().fg.unwrap_or(theme.fg()),
            show_border: true,
            border_chars: BorderChars::single(),
        },
        BlockType::Error => BlockStyle {
            border_color: theme.error_style().fg.unwrap_or(theme.fg()),
            background_color: theme.error_style().bg.unwrap_or(theme.bg()),
            foreground_color: theme.error_style().fg.unwrap_or(theme.fg()),
            show_border: true,
            border_chars: BorderChars::single(),
        },
        BlockType::AI => BlockStyle {
            border_color: theme.ai_block_style().fg.unwrap_or(theme.fg()),
            background_color: theme.ai_block_style().bg.unwrap_or(theme.bg()),
            foreground_color: theme.ai_block_style().fg.unwrap_or(theme.fg()),
            show_border: true,
            border_chars: BorderChars::rounded(),
        },
        BlockType::System => BlockStyle {
            border_color: theme.border_style().fg.unwrap_or(theme.fg()),
            background_color: theme.bg(),
            foreground_color: theme.fg(),
            show_border: false,
            border_chars: BorderChars::single(),
        },
    }
}

#[derive(Debug, Clone)]
pub struct BlockStyle {
    pub border_color: ratatui::style::Color,
    pub background_color: ratatui::style::Color,
    pub foreground_color: ratatui::style::Color,
    pub show_border: bool,
    pub border_chars: BorderChars,
}

#[derive(Debug, Clone, Copy)]
pub struct BorderChars {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub horizontal: &'static str,
    pub vertical: &'static str,
    pub cross: &'static str,
    pub tee_right: &'static str,
    pub tee_left: &'static str,
}

impl BorderChars {
    pub fn single() -> Self {
        Self {
            top_left: "┌",
            top_right: "┐",
            bottom_left: "└",
            bottom_right: "┘",
            horizontal: "─",
            vertical: "│",
            cross: "┼",
            tee_right: "├",
            tee_left: "┤",
        }
    }

    pub fn rounded() -> Self {
        Self {
            top_left: "╭",
            top_right: "╮",
            bottom_left: "╰",
            bottom_right: "╯",
            horizontal: "─",
            vertical: "│",
            cross: "┼",
            tee_right: "├",
            tee_left: "┤",
        }
    }

    pub fn double() -> Self {
        Self {
            top_left: "╔",
            top_right: "╗",
            bottom_left: "╚",
            bottom_right: "╝",
            horizontal: "═",
            vertical: "║",
            cross: "╬",
            tee_right: "╠",
            tee_left: "╣",
        }
    }

    pub fn heavy() -> Self {
        Self {
            top_left: "┏",
            top_right: "┓",
            bottom_left: "┗",
            bottom_right: "┓",
            horizontal: "━",
            vertical: "┃",
            cross: "╋",
            tee_right: "┣",
            tee_left: "┫",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = Block::new(BlockType::Command);
        assert_eq!(block.block_type, BlockType::Command);
        assert!(!block.collapsed);
        assert!(!block.pinned);
    }

    #[test]
    fn test_block_title() {
        let block = Block::new(BlockType::Output).with_title("Test Output");
        assert_eq!(block.title, Some("Test Output".to_string()));
    }

    #[test]
    fn test_block_pin_toggle() {
        let mut block = Block::new(BlockType::Output);
        assert!(!block.pinned);
        block.pin();
        assert!(block.pinned);
        block.unpin();
        assert!(!block.pinned);
    }

    #[test]
    fn test_block_collapse() {
        let mut block = Block::new(BlockType::Output);
        assert!(!block.collapsed);
        block.collapse();
        assert!(block.collapsed);
    }

    #[test]
    fn test_block_manager_add_block() {
        let mut manager = BlockManager::new(100, 1000);
        let id = manager.add_block(Block::new(BlockType::Output));
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.get_block(id).unwrap().id, id);
    }

    #[test]
    fn test_block_manager_max_blocks() {
        let mut manager = BlockManager::new(2, 1000);
        let id1 = manager.add_block(Block::new(BlockType::Output));
        let id2 = manager.add_block(Block::new(BlockType::Output));
        let id3 = manager.add_block(Block::new(BlockType::Output));
        assert_eq!(manager.len(), 2);
        assert!(manager.get_block(id1).is_none());
        assert!(manager.get_block(id2).is_some());
        assert!(manager.get_block(id3).is_some());
    }

    #[test]
    fn test_block_manager_pinned_preserved() {
        let mut manager = BlockManager::new(2, 1000);
        let pinned_id = manager.add_block(Block::new(BlockType::Output));
        manager.get_block_mut(pinned_id).unwrap().pin();
        let id2 = manager.add_block(Block::new(BlockType::Output));
        let id3 = manager.add_block(Block::new(BlockType::Output));
        assert_eq!(manager.len(), 2);
        assert!(manager.get_block(pinned_id).is_some());
        assert!(manager.get_block(id2).is_some());
        assert!(manager.get_block(id3).is_none());
    }

    #[test]
    fn test_block_manager_navigation() {
        let mut manager = BlockManager::new(100, 1000);
        let id1 = manager.add_block(Block::new(BlockType::Output));
        let _id2 = manager.add_block(Block::new(BlockType::Output));
        let id3 = manager.add_block(Block::new(BlockType::Output));

        assert_eq!(manager.current_block, Some(id3));
        assert_eq!(manager.next_block(), Some(id1));
        assert_eq!(manager.prev_block(), Some(id3));
    }

    #[test]
    fn test_block_type_detection() {
        assert!(!BlockType::Command.is_error());
        assert!(BlockType::Error.is_error());
        assert!(!BlockType::AI.is_error());
        assert!(BlockType::AI.is_ai());
    }

    #[test]
    fn test_border_chars() {
        let single = BorderChars::single();
        assert_eq!(single.top_left, "┌");
        assert_eq!(single.horizontal, "─");

        let rounded = BorderChars::rounded();
        assert_eq!(rounded.top_left, "╭");
        assert_eq!(rounded.top_right, "╮");
    }

    #[test]
    fn test_block_scroll() {
        let mut block = Block::new(BlockType::Output);
        block.height = 10;
        block.scroll_up(5);
        assert_eq!(block.scroll_offset, 5);
        block.scroll_down(3);
        assert_eq!(block.scroll_offset, 2);
        block.scroll_to_top();
        assert_eq!(block.scroll_offset, 0);
    }
}
