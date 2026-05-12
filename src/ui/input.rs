//! Command input bar for TerRust
//!
//! Provides an interactive input bar with command history navigation,
//! syntax highlighting, and prompt customization.

use std::collections::VecDeque;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyModifiers};

const MAX_HISTORY_SIZE: usize = 1000;
const MAX_INPUT_LENGTH: usize = 4096;

#[derive(Debug, Clone)]
pub struct InputBar {
    pub prompt: String,
    pub input: String,
    pub cursor_position: usize,
    pub history: VecDeque<String>,
    pub history_index: Option<usize>,
    pub history_search: Option<String>,
    pub cursor_x: u16,
    pub max_width: u16,
    pub working_directory: Option<PathBuf>,
    pub git_branch: Option<String>,
    pub show_git_status: bool,
}

impl InputBar {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            input: String::new(),
            cursor_position: 0,
            history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            history_index: None,
            history_search: None,
            cursor_x: 0,
            max_width: 80,
            working_directory: None,
            git_branch: None,
            show_git_status: true,
        }
    }

    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_string();
    }

    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_directory = Some(dir);
        self
    }

    pub fn with_git_branch(mut self, branch: Option<String>) -> Self {
        self.git_branch = branch;
        self
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn get_content(&self) -> &String {
        &self.input
    }

    pub fn is_active(&self) -> bool {
        true
    }

    pub fn cancel(&mut self) {
        self.history_index = None;
        self.history_search = None;
    }

    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        match (key, modifiers) {
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                self.insert_str(&c.to_string());
                true
            }
            (KeyCode::Backspace, _) => {
                self.backspace();
                true
            }
            (KeyCode::Delete, _) => {
                self.delete_char();
                true
            }
            (KeyCode::Left, KeyModifiers::CONTROL) => {
                self.move_word_left();
                true
            }
            (KeyCode::Right, KeyModifiers::CONTROL) => {
                self.move_word_right();
                true
            }
            (KeyCode::Left, _) => {
                self.move_cursor_left();
                true
            }
            (KeyCode::Right, _) => {
                self.move_cursor_right();
                true
            }
            (KeyCode::Home, _) => {
                self.move_cursor_to_start();
                true
            }
            (KeyCode::End, _) => {
                self.move_cursor_to_end();
                true
            }
            (KeyCode::Up, _) => self.history_up(),
            (KeyCode::Down, _) => self.history_down(),
            (KeyCode::Enter, _) => true,
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        self.input.len()
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
        self.history_index = None;
        self.history_search = None;
    }

    pub fn push_char(&mut self, c: char) {
        if self.input.len() >= MAX_INPUT_LENGTH {
            return;
        }
        self.input.push(c);
        self.cursor_position += 1;
    }

    pub fn insert_str(&mut self, s: &str) {
        let remaining = MAX_INPUT_LENGTH.saturating_sub(self.input.len());
        let s = &s[..s.len().min(remaining)];
        self.input.insert_str(self.cursor_position, s);
        self.cursor_position += s.len();
    }

    pub fn delete_char(&mut self) -> Option<char> {
        if self.cursor_position >= self.input.len() {
            return None;
        }
        let c = self.input.remove(self.cursor_position);
        Some(c)
    }

    pub fn backspace(&mut self) -> Option<char> {
        if self.cursor_position == 0 {
            return None;
        }
        self.cursor_position -= 1;
        Some(self.input.remove(self.cursor_position))
    }

    pub fn delete_word(&mut self) -> Option<String> {
        if self.input.is_empty() || self.cursor_position >= self.input.len() {
            return None;
        }

        let start = self.cursor_position;
        let mut end = start;

        while end < self.input.len()
            && self
                .input
                .chars()
                .nth(end)
                .map_or(false, |c| !c.is_whitespace())
        {
            end += 1;
        }
        if end == start {
            return None;
        }

        let mut removed: String = self.input.drain(start..end).collect();
        if self
            .input
            .chars()
            .nth(start)
            .map_or(false, |c| c.is_whitespace())
        {
            removed.push(' ');
        }
        Some(removed)
    }

    pub fn delete_to_start(&mut self) -> Option<String> {
        if self.cursor_position == 0 {
            return None;
        }
        let removed = self.input.drain(0..self.cursor_position).collect();
        self.cursor_position = 0;
        Some(removed)
    }

    pub fn delete_to_end(&mut self) -> Option<String> {
        if self.cursor_position >= self.input.len() {
            return None;
        }
        Some(self.input.drain(self.cursor_position..).collect())
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        } else if !self.input.is_empty() {
            self.cursor_position = 1;
        }
    }

    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.input.len();
    }

    pub fn move_word_left(&mut self) {
        while self.cursor_position > 0 {
            self.cursor_position -= 1;
            if self.cursor_position == 0
                || self
                    .input
                    .chars()
                    .nth(self.cursor_position - 1)
                    .map_or(false, |c| c.is_whitespace())
            {
                break;
            }
        }
    }

    pub fn move_word_right(&mut self) {
        while self.cursor_position < self.input.len() {
            self.cursor_position += 1;
            if self.cursor_position >= self.input.len()
                || self
                    .input
                    .chars()
                    .nth(self.cursor_position - 1)
                    .map_or(false, |c| c.is_whitespace())
            {
                break;
            }
        }
    }

    pub fn history_push(&mut self, command: impl Into<String>) {
        let command = command.into();
        if command.trim().is_empty() {
            return;
        }
        if let Some(last) = self.history.front() {
            if last == &command {
                return;
            }
        }
        self.history.push_front(command);
        if self.history.len() > MAX_HISTORY_SIZE {
            self.history.pop_back();
        }
        self.history_index = None;
        self.history_search = None;
    }

    pub fn history_up(&mut self) -> bool {
        if self.history.is_empty() {
            return false;
        }
        let new_index = match self.history_index {
            None => {
                self.history_search = Some(self.input.clone());
                0
            }
            Some(i) => {
                if i >= self.history.len() - 1 {
                    return false;
                }
                i + 1
            }
        };
        self.history_index = Some(new_index);
        if let Some(cmd) = self.history.get(new_index) {
            self.input = cmd.clone();
            self.cursor_position = self.input.len();
            true
        } else {
            false
        }
    }

    pub fn history_down(&mut self) -> bool {
        let new_index = match self.history_index {
            None => return false,
            Some(0) => {
                self.history_index = None;
                self.input = self.history_search.take().unwrap_or_default();
                self.cursor_position = self.input.len();
                return true;
            }
            Some(i) => i - 1,
        };
        self.history_index = Some(new_index);
        if let Some(cmd) = self.history.get(new_index) {
            self.input = cmd.clone();
            self.cursor_position = self.input.len();
            true
        } else {
            false
        }
    }

    pub fn history_search_up(&mut self) -> bool {
        if self.history.is_empty() {
            return false;
        }
        let search_term = self.input.clone();
        let start_index = self.history_index.unwrap_or(0);

        for i in 0..self.history.len() {
            let check_index = (start_index + i + 1) % self.history.len();
            if let Some(cmd) = self.history.get(check_index) {
                if cmd.starts_with(&search_term) {
                    self.history_index = Some(check_index);
                    self.input = cmd.clone();
                    self.cursor_position = self.input.len();
                    return true;
                }
            }
        }
        false
    }

    pub fn history_search_down(&mut self) -> bool {
        if self.history.is_empty() {
            return false;
        }
        if self.history_index.is_none() {
            return false;
        }

        let search_term = self.input.clone();
        let current_index = self.history_index.unwrap();

        for i in (0..self.history.len()).rev() {
            if i >= current_index {
                continue;
            }
            if let Some(cmd) = self.history.get(i) {
                if cmd.starts_with(&search_term) {
                    self.history_index = Some(i);
                    self.input = cmd.clone();
                    self.cursor_position = self.input.len();
                    return true;
                }
            }
        }

        self.history_index = None;
        self.input = search_term;
        self.cursor_position = self.input.len();
        true
    }

    pub fn get_display_prompt(&self) -> String {
        let mut prompt = self.prompt.clone();

        if self.show_git_status {
            if let Some(ref branch) = self.git_branch {
                prompt = format!("{} \x1b[32m{}:{}\x1b[0m", prompt, branch, self.short_path());
            } else {
                prompt = format!("{} {}:{}", prompt, self.short_path(), "$");
            }
        } else {
            prompt = format!("{} {}:{}", prompt, self.short_path(), "$");
        }

        prompt
    }

    fn short_path(&self) -> String {
        if let Some(ref dir) = self.working_directory {
            let home = std::env::var("HOME").unwrap_or_default();
            if dir.to_string_lossy().starts_with(&home) {
                let display = dir.to_string_lossy();
                let rel = display.strip_prefix(&home).unwrap_or("");
                return format!("~{}", rel);
            }
            dir.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| dir.to_string_lossy().to_string())
        } else {
            "~".to_string()
        }
    }

    pub fn get_visual_cursor_column(&self, prompt_width: u16) -> u16 {
        let input_before_cursor = &self.input[..self.cursor_position];
        let visual_width = unicode_width::UnicodeWidthStr::width(input_before_cursor);
        (prompt_width as usize + visual_width) as u16
    }

    pub fn truncate_display(&self, max_width: u16) -> String {
        if self.input.len() as u16 > max_width {
            let start = self.input.len() - max_width as usize;
            format!("...{}", &self.input[start..])
        } else {
            self.input.clone()
        }
    }

    pub fn set_max_width(&mut self, width: u16) {
        self.max_width = width;
    }

    pub fn set_show_git_status(&mut self, show: bool) {
        self.show_git_status = show;
    }

    pub fn get_command_parts(&self) -> CommandParts {
        let trimmed = self.input.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let command = parts.first().map(|s| s.to_string());
        let args = parts
            .get(1..)
            .map(|v| v.iter().map(|s| s.to_string()).collect());

        CommandParts { command, args }
    }
}

#[derive(Debug, Clone)]
pub struct CommandParts {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
}

impl CommandParts {
    pub fn is_empty(&self) -> bool {
        self.command.is_none()
    }

    pub fn has_args(&self) -> bool {
        self.args.as_ref().map_or(false, |a| !a.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_bar_basic() {
        let mut input = InputBar::new("$ ");
        assert_eq!(input.len(), 0);
        assert!(input.is_empty());

        input.push_char('h');
        input.push_char('i');
        assert_eq!(input.input(), "hi");
        assert_eq!(input.len(), 2);
        assert_eq!(input.cursor_position, 2);
    }

    #[test]
    fn test_input_bar_backspace() {
        let mut input = InputBar::new("$ ");
        input.push_char('a');
        input.push_char('b');
        assert_eq!(input.cursor_position, 2);

        let deleted = input.backspace();
        assert_eq!(deleted, Some('b'));
        assert_eq!(input.input(), "a");
        assert_eq!(input.cursor_position, 1);
    }

    #[test]
    fn test_input_bar_delete_word() {
        let mut input = InputBar::new("$ ");
        input.insert_str("hello world test");
        input.move_cursor_to_start();
        input.move_word_right();
        assert_eq!(input.cursor_position, 6);

        let word = input.delete_word();
        assert_eq!(word, Some("world ".to_string()));
        assert_eq!(input.input(), "hello  test");
    }

    #[test]
    fn test_input_bar_cursor_movement() {
        let mut input = InputBar::new("$ ");
        input.insert_str("hello");

        input.move_cursor_left();
        assert_eq!(input.cursor_position, 4);

        input.move_cursor_right();
        assert_eq!(input.cursor_position, 5);

        input.move_cursor_to_start();
        assert_eq!(input.cursor_position, 0);

        input.move_cursor_to_end();
        assert_eq!(input.cursor_position, 5);
    }

    #[test]
    fn test_input_bar_history() {
        let mut input = InputBar::new("$ ");
        input.history_push("ls -la");
        input.history_push("git status");
        input.history_push("cargo build");

        assert_eq!(input.history.len(), 3);

        input.clear();
        input.insert_str("cargo");

        assert!(input.history_up());
        assert_eq!(input.input(), "cargo build");

        assert!(input.history_down());
        assert_eq!(input.input(), "cargo");
    }

    #[test]
    fn test_input_bar_history_navigation() {
        let mut input = InputBar::new("$ ");
        input.history_push("cmd1");
        input.history_push("cmd2");
        input.history_push("cmd3");

        assert!(input.history_up());
        assert_eq!(input.input(), "cmd3");
        assert!(input.history_up());
        assert_eq!(input.input(), "cmd2");
        assert!(input.history_up());
        assert_eq!(input.input(), "cmd1");
        assert!(!input.history_up());
        assert!(input.history_down());
        assert_eq!(input.input(), "cmd2");
    }

    #[test]
    fn test_command_parts() {
        let mut input = InputBar::new("$ ");
        input.insert_str("git commit -m \"initial\"");

        let parts = input.get_command_parts();
        assert_eq!(parts.command, Some("git".to_string()));
        assert_eq!(
            parts.args,
            Some(vec![
                "commit".to_string(),
                "-m".to_string(),
                "\"initial\"".to_string()
            ])
        );
    }

    #[test]
    fn test_input_bar_max_length() {
        let mut input = InputBar::new("$ ");
        let long_str = "a".repeat(MAX_INPUT_LENGTH + 100);
        input.insert_str(&long_str);
        assert!(input.len() <= MAX_INPUT_LENGTH);
    }

    #[test]
    fn test_input_bar_delete_to_start() {
        let mut input = InputBar::new("$ ");
        input.insert_str("hello world");
        input.move_cursor_right();
        input.move_cursor_right();
        input.move_cursor_right();

        let deleted = input.delete_to_start();
        assert_eq!(deleted, Some("hel".to_string()));
        assert_eq!(input.input(), "lo world");
        assert_eq!(input.cursor_position, 0);
    }

    #[test]
    fn test_input_bar_delete_to_end() {
        let mut mut_input = InputBar::new("$ ");
        mut_input.insert_str("hello world");
        mut_input.move_cursor_to_start();

        let deleted = mut_input.delete_to_end();
        assert_eq!(deleted, Some("hello world".to_string()));
        assert_eq!(mut_input.input(), "");
        assert_eq!(mut_input.cursor_position, 0);
    }

    #[test]
    fn test_short_path() {
        let input = InputBar::new("$ ");
        assert_eq!(input.short_path(), "~");

        let mut input = InputBar::new("$ ");
        input.working_directory = Some(PathBuf::from("/tmp/test"));
        assert_eq!(input.short_path(), "test");

        let mut input = InputBar::new("$ ");
        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() {
            let home_dir = PathBuf::from(&home);
            input.working_directory = Some(home_dir.join("projects/terrust"));
            let short = input.short_path();
            assert!(short.starts_with("~/") || short == "~");
        }
    }
}
