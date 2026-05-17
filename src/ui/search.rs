//! Search engine for scrollback content
//!
//! Provides incremental search across blocks, terminal grid, and scrollback
//! with match highlighting and navigation between results.

use crate::ui::blocks::BlockManager;
use uuid::Uuid;

/// A single search match location
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub block_id: Uuid,
    pub line_idx: usize,
    pub col_idx: usize,
    pub length: usize,
    pub line_text: String,
}

/// Search engine for finding text across scrollback content
#[derive(Debug, Clone)]
pub struct SearchEngine {
    query: String,
    matches: Vec<SearchMatch>,
    current_match: usize,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            current_match: 0,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn matches(&self) -> &[SearchMatch] {
        &self.matches
    }

    pub fn current_match(&self) -> usize {
        self.current_match
    }

    pub fn total_matches(&self) -> usize {
        self.matches.len()
    }

    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }

    pub fn current_match_info(&self) -> Option<&SearchMatch> {
        self.matches.get(self.current_match)
    }

    pub fn set_query(&mut self, query: &str, block_manager: &BlockManager) {
        self.query = query.to_string();
        self.matches.clear();
        self.current_match = 0;

        if query.is_empty() {
            return;
        }

        let query_lower = query.to_lowercase();
        for block in block_manager.blocks() {
            for (line_idx, row) in block.content.iter().enumerate() {
                let line_text: String = row.iter().map(|c| c.character).collect();
                let line_lower = line_text.to_lowercase();

                let mut search_start = 0;
                while let Some(col_idx) = line_lower[search_start..].find(&query_lower) {
                    let absolute_col = search_start + col_idx;
                    self.matches.push(SearchMatch {
                        block_id: block.id,
                        line_idx,
                        col_idx: absolute_col,
                        length: query.len(),
                        line_text: line_text.clone(),
                    });
                    search_start = absolute_col + 1;
                    if search_start >= line_lower.len() {
                        break;
                    }
                }
            }
        }
    }

    pub fn next_match(&mut self) -> bool {
        if self.matches.is_empty() {
            return false;
        }
        self.current_match = (self.current_match + 1) % self.matches.len();
        true
    }

    pub fn prev_match(&mut self) -> bool {
        if self.matches.is_empty() {
            return false;
        }
        self.current_match = if self.current_match == 0 {
            self.matches.len() - 1
        } else {
            self.current_match - 1
        };
        true
    }

    pub fn reset(&mut self) {
        self.query.clear();
        self.matches.clear();
        self.current_match = 0;
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a cell at (line_idx, col_idx) falls within a search match range
pub fn is_cell_in_match(
    line_idx: usize,
    col_idx: usize,
    matches: &[SearchMatch],
    _current_match: usize,
) -> bool {
    for m in matches.iter() {
        if m.line_idx == line_idx
            && col_idx >= m.col_idx
            && col_idx < m.col_idx + m.length
        {
            return true;
        }
    }
    false
}

/// Check if a cell is the current (active) search match
pub fn is_cell_in_current_match(
    line_idx: usize,
    col_idx: usize,
    matches: &[SearchMatch],
    current_match: usize,
) -> bool {
    matches.get(current_match).map_or(false, |m| {
        m.line_idx == line_idx
            && col_idx >= m.col_idx
            && col_idx < m.col_idx + m.length
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::Cell;
    use crate::ui::blocks::{Block, BlockManager, BlockType};

    fn cell_row(text: &str) -> Vec<Cell> {
        text.chars()
            .map(|c| Cell {
                character: c,
                foreground: None,
                background: None,
                attributes: crate::terminal::Attributes::default(),
            })
            .collect()
    }

    #[test]
    fn test_empty_query_returns_no_matches() {
        let mut manager = BlockManager::new(10, 100);
        let mut block = Block::new(BlockType::Output).with_title("test");
        block.content = vec![cell_row("hello world")];
        manager.add_block(block);

        let mut engine = SearchEngine::new();
        engine.set_query("", &manager);
        assert_eq!(engine.total_matches(), 0);
    }

    #[test]
    fn test_basic_search() {
        let mut manager = BlockManager::new(10, 100);
        let mut block = Block::new(BlockType::Output).with_title("test");
        block.content = vec![cell_row("hello world")];
        manager.add_block(block);

        let mut engine = SearchEngine::new();
        engine.set_query("world", &manager);
        assert_eq!(engine.total_matches(), 1);
        assert_eq!(engine.matches()[0].col_idx, 6);
        assert_eq!(engine.matches()[0].length, 5);
    }

    #[test]
    fn test_case_insensitive_search() {
        let mut manager = BlockManager::new(10, 100);
        let mut block = Block::new(BlockType::Command).with_title("test");
        block.content = vec![cell_row("Hello World")];
        manager.add_block(block);

        let mut engine = SearchEngine::new();
        engine.set_query("hello", &manager);
        assert_eq!(engine.total_matches(), 1);
    }

    #[test]
    fn test_multiple_matches() {
        let mut manager = BlockManager::new(10, 100);
        let mut block = Block::new(BlockType::Output).with_title("test");
        block.content = vec![
            cell_row("the cat and the dog"),
            cell_row("the bird flew"),
        ];
        manager.add_block(block);

        let mut engine = SearchEngine::new();
        engine.set_query("the", &manager);
        assert_eq!(engine.total_matches(), 3);
    }

    #[test]
    fn test_match_navigation() {
        let mut manager = BlockManager::new(10, 100);
        let mut block = Block::new(BlockType::Output).with_title("test");
        block.content = vec![cell_row("a b a b")];
        manager.add_block(block);

        let mut engine = SearchEngine::new();
        engine.set_query("a", &manager);
        assert_eq!(engine.total_matches(), 2);
        assert_eq!(engine.current_match, 0);

        assert!(engine.next_match());
        assert_eq!(engine.current_match, 1);

        assert!(engine.next_match());
        assert_eq!(engine.current_match, 0);

        assert!(engine.prev_match());
        assert_eq!(engine.current_match, 1);
    }

    #[test]
    fn test_reset() {
        let mut manager = BlockManager::new(10, 100);
        let mut block = Block::new(BlockType::Output).with_title("test");
        block.content = vec![cell_row("hello")];
        manager.add_block(block);

        let mut engine = SearchEngine::new();
        engine.set_query("hello", &manager);
        assert!(engine.has_matches());

        engine.reset();
        assert!(!engine.has_matches());
        assert_eq!(engine.query(), "");
    }

    #[test]
    fn test_no_match_returns_empty() {
        let mut manager = BlockManager::new(10, 100);
        let mut block = Block::new(BlockType::Output).with_title("test");
        block.content = vec![cell_row("hello world")];
        manager.add_block(block);

        let mut engine = SearchEngine::new();
        engine.set_query("zzzz", &manager);
        assert_eq!(engine.total_matches(), 0);
        assert!(!engine.has_matches());
    }

    #[test]
    fn test_is_cell_in_match() {
        let block_id = Uuid::new_v4();
        let matches = vec![SearchMatch {
            block_id,
            line_idx: 0,
            col_idx: 6,
            length: 5,
            line_text: "hello world".to_string(),
        }];

        assert!(is_cell_in_match(0, 6, &matches, 0));
        assert!(is_cell_in_match(0, 10, &matches, 0));
        assert!(!is_cell_in_match(0, 5, &matches, 0));
        assert!(!is_cell_in_match(0, 11, &matches, 0));
        assert!(!is_cell_in_match(1, 6, &matches, 0));
    }
}
