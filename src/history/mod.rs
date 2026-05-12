//! Command history management for TerRust
//!
//! Provides history navigation for the input bar (up/down arrows)

use std::collections::VecDeque;

/// Command history storage with navigation support
#[derive(Debug, Clone)]
pub struct History {
    /// Stored command entries
    entries: VecDeque<String>,
    /// Maximum number of entries to keep
    max_size: usize,
    /// Current navigation position (None = at newest entry or not navigating)
    position: Option<usize>,
}

impl History {
    /// Create a new History instance with a maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_size),
            max_size,
            position: None,
        }
    }

    /// Add a new entry to the history
    /// If the entry is empty or whitespace, it's ignored
    pub fn add(&mut self, entry: String) {
        let trimmed = entry.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        self.entries.push_back(trimmed);
        if self.entries.len() > self.max_size {
            self.entries.pop_front();
        }
        self.position = None;
    }

    /// Navigate to the previous (older) entry in history
    /// Returns the entry text if available
    pub fn previous(&mut self) -> Option<&str> {
        let len = self.entries.len();
        if len == 0 {
            return None;
        }

        match self.position {
            None => {
                // Start from the most recent entry
                self.position = Some(len - 1);
                self.entries.get(len - 1).map(|s| s.as_str())
            }
            Some(pos) if pos > 0 => {
                self.position = Some(pos - 1);
                self.entries.get(pos - 1).map(|s| s.as_str())
            }
            Some(0) => {
                // Already at oldest entry
                self.entries.get(0).map(|s| s.as_str())
            }
            Some(pos) => self.entries.get(pos).map(|s| s.as_str()),
        }
    }

    /// Navigate to the next (newer) entry in history
    /// Returns the entry text if available
    pub fn next(&mut self) -> Option<&str> {
        match self.position {
            None => None, // Not navigating
            Some(pos) => {
                let len = self.entries.len();
                if pos + 1 < len {
                    self.position = Some(pos + 1);
                    self.entries.get(pos + 1).map(|s| s.as_str())
                } else {
                    // At newest entry, clear position
                    self.position = None;
                    None
                }
            }
        }
    }

    /// Reset the navigation position (e.g., when user starts typing new command)
    pub fn reset_position(&mut self) {
        self.position = None;
    }

    /// Clear all history entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.position = None;
    }

    /// Get the current entry without changing position
    pub fn current(&self) -> Option<&str> {
        self.position.and_then(|pos| self.entries.get(pos).map(|s| s.as_str()))
    }

    /// Get the number of entries in history
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all entries as a vector (oldest first)
    pub fn entries(&self) -> Vec<String> {
        self.entries.iter().cloned().collect()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_navigate() {
        let mut history = History::new(10);
        history.add("command1".to_string());
        history.add("command2".to_string());
        history.add("command3".to_string());

        assert_eq!(history.len(), 3);

        assert_eq!(history.previous(), Some("command3"));
        assert_eq!(history.previous(), Some("command2"));
        assert_eq!(history.previous(), Some("command1"));

        assert_eq!(history.next(), Some("command2"));
        assert_eq!(history.next(), Some("command3"));
        assert_eq!(history.next(), None);
    }

    #[test]
    fn test_max_size() {
        let mut history = History::new(2);
        history.add("a".to_string());
        history.add("b".to_string());
        history.add("c".to_string());

        assert_eq!(history.len(), 2);
        assert_eq!(history.entries(), vec!["b", "c"]);
    }

    #[test]
    fn test_empty_entries_ignored() {
        let mut history = History::new(10);
        history.add("  ".to_string());
        history.add("command".to_string());
        history.add("".to_string());

        assert_eq!(history.len(), 1);
    }
}
