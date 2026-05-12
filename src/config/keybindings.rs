//! Keybindings configuration for TerRust
//!
//! Handles keyboard shortcuts and their mappings to application actions.

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Represents a keyboard key combination
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Keybinding {
    /// The key code
    pub key: String,
    /// Key modifiers (Ctrl, Alt, Shift)
    #[serde(default)]
    pub modifiers: Vec<String>,
}

impl Keybinding {
    /// Create a new keybinding
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            modifiers: Vec::new(),
        }
    }

    /// Add a modifier
    pub fn with_modifier(mut self, modifier: &str) -> Self {
        self.modifiers.push(modifier.to_string());
        self
    }

    /// Check if this keybinding matches a key event
    pub fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        // Check key code
        let key_matches = match &self.key.to_lowercase() as &str {
            "enter" | "return" => code == KeyCode::Enter,
            "tab" => code == KeyCode::Tab,
            "esc" | "escape" => code == KeyCode::Esc,
            "backspace" => code == KeyCode::Backspace,
            "delete" => code == KeyCode::Delete,
            "insert" => code == KeyCode::Insert,
            "up" => code == KeyCode::Up,
            "down" => code == KeyCode::Down,
            "left" => code == KeyCode::Left,
            "right" => code == KeyCode::Right,
            "home" => code == KeyCode::Home,
            "end" => code == KeyCode::End,
            "pageup" | "pgup" => code == KeyCode::PageUp,
            "pagedown" | "pgdown" => code == KeyCode::PageDown,
            "f1" => code == KeyCode::F(1),
            "f2" => code == KeyCode::F(2),
            "f3" => code == KeyCode::F(3),
            "f4" => code == KeyCode::F(4),
            "f5" => code == KeyCode::F(5),
            "f6" => code == KeyCode::F(6),
            "f7" => code == KeyCode::F(7),
            "f8" => code == KeyCode::F(8),
            "f9" => code == KeyCode::F(9),
            "f10" => code == KeyCode::F(10),
            "f11" => code == KeyCode::F(11),
            "f12" => code == KeyCode::F(12),
            "space" => code == KeyCode::Char(' '),
            _ => {
                if self.key.len() == 1 {
                    if let KeyCode::Char(c) = code {
                        c.to_lowercase().to_string() == self.key.to_lowercase()
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        };

        if !key_matches {
            return false;
        }

        // Check modifiers
        let has_ctrl = self.modifiers.iter().any(|m| m.to_lowercase() == "ctrl" || m == "^");
        let has_alt = self.modifiers.iter().any(|m| m.to_lowercase() == "alt" || m == "%");
        let has_shift = self.modifiers.iter().any(|m| m.to_lowercase() == "shift" || m == "+");

        let required_ctrl = has_ctrl == modifiers.contains(KeyModifiers::CONTROL);
        let required_alt = has_alt == modifiers.contains(KeyModifiers::ALT);
        let required_shift = has_shift == modifiers.contains(KeyModifiers::SHIFT);

        // For single-character keys with Shift, the character case matters
        if let KeyCode::Char(c) = code {
            if has_shift && !c.is_uppercase() && c != ' ' {
                return false;
            }
        }

        required_ctrl && required_alt && required_shift
    }
}

impl fmt::Display for Keybinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts: Vec<String> = Vec::new();
        
        for modif in &self.modifiers {
            match modif.to_lowercase().as_str() {
                "ctrl" | "^" => parts.push("Ctrl+".to_string()),
                "alt" | "%" => parts.push("Alt+".to_string()),
                "shift" | "+" => parts.push("Shift+".to_string()),
                _ => parts.push(format!("{}+", modif)),
            }
        }
        
        // Format the key
        let key = match self.key.to_lowercase().as_str() {
            "space" => "Space".to_string(),
            "enter" | "return" => "Enter".to_string(),
            "tab" => "Tab".to_string(),
            "esc" | "escape" => "Esc".to_string(),
            "backspace" => "Backspace".to_string(),
            "delete" => "Del".to_string(),
            "insert" => "Ins".to_string(),
            "up" => "↑".to_string(),
            "down" => "↓".to_string(),
            "left" => "←".to_string(),
            "right" => "→".to_string(),
            "home" => "Home".to_string(),
            "end" => "End".to_string(),
            "pageup" | "pgup" => "PgUp".to_string(),
            "pagedown" | "pgdown" => "PgDn".to_string(),
            _ => {
                if self.key.len() == 1 {
                    self.key.to_uppercase()
                } else {
                    self.key.clone()
                }
            }
        };
        
        parts.push(key);
        write!(f, "{}", parts.join(""))
    }
}

/// Action that can be triggered by a keybinding
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // Navigation
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PageUp,
    PageDown,
    MoveToStart,
    MoveToEnd,
    
    // Editing
    Backspace,
    Delete,
    DeleteWord,
    DeleteToStart,
    DeleteToEnd,
    Insert,
    
    // Selection
    SelectUp,
    SelectDown,
    SelectLeft,
    SelectRight,
    SelectWord,
    SelectAll,
    Copy,
    Paste,
    
    // Command execution
    Execute,
    NewLine,
    
    // AI
    AIPrompt,
    AIComplete,
    AINextSuggestion,
    AIPrevSuggestion,
    AIAcceptSuggestion,
    
    // Blocks
    NextBlock,
    PrevBlock,
    CollapseBlock,
    ExpandBlock,
    PinBlock,
    CopyBlock,
    SelectBlock,
    
    // Tabs
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    
    // Windows
    NewWindow,
    CloseWindow,
    NextWindow,
    PrevWindow,
    
    // Search
    SearchForward,
    SearchBackward,
    SearchNext,
    SearchPrev,
    SearchCancel,
    
    // Scroll
    ScrollUp,
    ScrollDown,
    ScrollToTop,
    ScrollToBottom,
    
    // AI Workflows
    ExecuteWorkflow,
    CancelWorkflow,
    
    // Misc
    ClearScreen,
    Quit,
    ForceQuit,
    ToggleFullscreen,
    ShowHelp,
    ShowHistory,
    ShowCommands,
    
    // Custom actions
    Custom(String),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::MoveUp => write!(f, "Move Up"),
            Action::MoveDown => write!(f, "Move Down"),
            Action::MoveLeft => write!(f, "Move Left"),
            Action::MoveRight => write!(f, "Move Right"),
            Action::PageUp => write!(f, "Page Up"),
            Action::PageDown => write!(f, "Page Down"),
            Action::MoveToStart => write!(f, "Move to Start"),
            Action::MoveToEnd => write!(f, "Move to End"),
            Action::Backspace => write!(f, "Backspace"),
            Action::Delete => write!(f, "Delete"),
            Action::DeleteWord => write!(f, "Delete Word"),
            Action::DeleteToStart => write!(f, "Delete to Start"),
            Action::DeleteToEnd => write!(f, "Delete to End"),
            Action::Insert => write!(f, "Insert"),
            Action::SelectUp => write!(f, "Select Up"),
            Action::SelectDown => write!(f, "Select Down"),
            Action::SelectLeft => write!(f, "Select Left"),
            Action::SelectRight => write!(f, "Select Right"),
            Action::SelectWord => write!(f, "Select Word"),
            Action::SelectAll => write!(f, "Select All"),
            Action::Copy => write!(f, "Copy"),
            Action::Paste => write!(f, "Paste"),
            Action::Execute => write!(f, "Execute"),
            Action::NewLine => write!(f, "New Line"),
            Action::AIPrompt => write!(f, "AI Prompt"),
            Action::AIComplete => write!(f, "AI Complete"),
            Action::AINextSuggestion => write!(f, "AI Next Suggestion"),
            Action::AIPrevSuggestion => write!(f, "AI Previous Suggestion"),
            Action::AIAcceptSuggestion => write!(f, "AI Accept Suggestion"),
            Action::NextBlock => write!(f, "Next Block"),
            Action::PrevBlock => write!(f, "Previous Block"),
            Action::CollapseBlock => write!(f, "Collapse Block"),
            Action::ExpandBlock => write!(f, "Expand Block"),
            Action::PinBlock => write!(f, "Pin Block"),
            Action::CopyBlock => write!(f, "Copy Block"),
            Action::SelectBlock => write!(f, "Select Block"),
            Action::NewTab => write!(f, "New Tab"),
            Action::CloseTab => write!(f, "Close Tab"),
            Action::NextTab => write!(f, "Next Tab"),
            Action::PrevTab => write!(f, "Previous Tab"),
            Action::NewWindow => write!(f, "New Window"),
            Action::CloseWindow => write!(f, "Close Window"),
            Action::NextWindow => write!(f, "Next Window"),
            Action::PrevWindow => write!(f, "Previous Window"),
            Action::SearchForward => write!(f, "Search Forward"),
            Action::SearchBackward => write!(f, "Search Backward"),
            Action::SearchNext => write!(f, "Search Next"),
            Action::SearchPrev => write!(f, "Search Previous"),
            Action::SearchCancel => write!(f, "Search Cancel"),
            Action::ScrollUp => write!(f, "Scroll Up"),
            Action::ScrollDown => write!(f, "Scroll Down"),
            Action::ScrollToTop => write!(f, "Scroll to Top"),
            Action::ScrollToBottom => write!(f, "Scroll to Bottom"),
            Action::ExecuteWorkflow => write!(f, "Execute Workflow"),
            Action::CancelWorkflow => write!(f, "Cancel Workflow"),
            Action::ClearScreen => write!(f, "Clear Screen"),
            Action::Quit => write!(f, "Quit"),
            Action::ForceQuit => write!(f, "Force Quit"),
            Action::ToggleFullscreen => write!(f, "Toggle Fullscreen"),
            Action::ShowHelp => write!(f, "Show Help"),
            Action::ShowHistory => write!(f, "Show History"),
            Action::ShowCommands => write!(f, "Show Commands"),
            Action::Custom(s) => write!(f, "Custom: {}", s),
        }
    }
}

/// Keybindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    /// Mapping from keybindings to actions
    #[serde(default)]
    pub bindings: HashMap<String, Action>,
    
    /// Mode-specific keybindings (insert, normal, etc.)
    #[serde(default)]
    pub modes: HashMap<String, HashMap<String, Action>>,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        
        // Navigation
        bindings.insert("up".to_string(), Action::MoveUp);
        bindings.insert("down".to_string(), Action::MoveDown);
        bindings.insert("left".to_string(), Action::MoveLeft);
        bindings.insert("right".to_string(), Action::MoveRight);
        bindings.insert("ctrl+up".to_string(), Action::ScrollUp);
        bindings.insert("ctrl+down".to_string(), Action::ScrollDown);
        bindings.insert("ctrl+left".to_string(), Action::PrevBlock);
        bindings.insert("ctrl+right".to_string(), Action::NextBlock);
        bindings.insert("home".to_string(), Action::MoveToStart);
        bindings.insert("end".to_string(), Action::MoveToEnd);
        bindings.insert("pageup".to_string(), Action::PageUp);
        bindings.insert("pagedown".to_string(), Action::PageDown);
        
        // Editing
        bindings.insert("backspace".to_string(), Action::Backspace);
        bindings.insert("delete".to_string(), Action::Delete);
        bindings.insert("ctrl+w".to_string(), Action::DeleteWord);
        bindings.insert("ctrl+u".to_string(), Action::DeleteToStart);
        bindings.insert("ctrl+k".to_string(), Action::DeleteToEnd);
        bindings.insert("insert".to_string(), Action::Insert);
        
        // Selection
        bindings.insert("shift+up".to_string(), Action::SelectUp);
        bindings.insert("shift+down".to_string(), Action::SelectDown);
        bindings.insert("shift+left".to_string(), Action::SelectLeft);
        bindings.insert("shift+right".to_string(), Action::SelectRight);
        bindings.insert("ctrl+shift+w".to_string(), Action::SelectWord);
        bindings.insert("ctrl+a".to_string(), Action::SelectAll);
        bindings.insert("ctrl+c".to_string(), Action::Copy);
        bindings.insert("ctrl+v".to_string(), Action::Paste);
        
        // Command execution
        bindings.insert("enter".to_string(), Action::Execute);
        bindings.insert("ctrl+j".to_string(), Action::NewLine);
        
        // AI
        bindings.insert("ctrl+space".to_string(), Action::AIComplete);
        bindings.insert("/".to_string(), Action::AIPrompt);
        bindings.insert("ctrl+.".to_string(), Action::AIAcceptSuggestion);
        
        // Blocks
        bindings.insert("ctrl+[".to_string(), Action::CollapseBlock);
        bindings.insert("ctrl+]".to_string(), Action::ExpandBlock);
        bindings.insert("ctrl+p".to_string(), Action::PinBlock);
        bindings.insert("ctrl+shift+c".to_string(), Action::CopyBlock);
        
        // Tabs
        bindings.insert("ctrl+t".to_string(), Action::NewTab);
        bindings.insert("ctrl+w".to_string(), Action::CloseTab);
        bindings.insert("ctrl+tab".to_string(), Action::NextTab);
        bindings.insert("ctrl+shift+tab".to_string(), Action::PrevTab);
        
        // Search
        bindings.insert("ctrl+f".to_string(), Action::SearchForward);
        bindings.insert("ctrl+r".to_string(), Action::SearchBackward);
        bindings.insert("ctrl+g".to_string(), Action::SearchCancel);
        
        // Misc
        bindings.insert("ctrl+l".to_string(), Action::ClearScreen);
        bindings.insert("ctrl+q".to_string(), Action::Quit);
        bindings.insert("ctrl+c".to_string(), Action::ForceQuit); // Also used for copy
        bindings.insert("f1".to_string(), Action::ShowHelp);
        bindings.insert("f2".to_string(), Action::ShowHistory);
        bindings.insert("f3".to_string(), Action::ShowCommands);
        
        Self {
            bindings,
            modes: HashMap::new(),
        }
    }
}

impl KeybindingsConfig {
    /// Get the action for a key event
    pub fn get_action(&self, code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
        // Check mode-specific bindings first
        // For now, we only have global bindings
        
        // Build a key string from the event
        let mut key_str = String::new();
        
        if modifiers.contains(KeyModifiers::CONTROL) {
            key_str.push_str("ctrl+");
        }
        if modifiers.contains(KeyModifiers::ALT) {
            key_str.push_str("alt+");
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            key_str.push_str("shift+");
        }
        
        // Add the key
        key_str.push_str(&key_code_to_string(code));
        
        // Look up the binding
        if let Some(action) = self.bindings.get(&key_str.to_lowercase()) {
            return Some(action.clone());
        }
        
        // Try without modifiers (for simple keys)
        if !key_str.contains('+') {
            if let Some(action) = self.bindings.get(&key_str) {
                return Some(action.clone());
            }
        }
        
        None
    }

    /// Get the keybinding for an action
    pub fn get_keybinding(&self, action: &Action) -> Option<Vec<Keybinding>> {
        let mut bindings = Vec::new();
        
        for (key_str, bound_action) in &self.bindings {
            if bound_action == action {
                if let Some(binding) = parse_keybinding_string(key_str) {
                    bindings.push(binding);
                }
            }
        }
        
        if bindings.is_empty() {
            None
        } else {
            Some(bindings)
        }
    }

    /// Add or update a keybinding
    pub fn set_binding(&mut self, keybinding: Keybinding, action: Action) {
        let key_str = keybinding.to_string().to_lowercase();
        self.bindings.insert(key_str, action);
    }

    /// Remove a keybinding
    pub fn remove_binding(&mut self, keybinding: &Keybinding) {
        let key_str = keybinding.to_string().to_lowercase();
        self.bindings.remove(&key_str);
    }

    /// Get all keybindings
    pub fn all_bindings(&self) -> Vec<(Keybinding, Action)> {
        let mut result = Vec::new();
        
        for (key_str, action) in &self.bindings {
            if let Some(binding) = parse_keybinding_string(key_str) {
                result.push((binding, action.clone()));
            }
        }
        
        result
    }
}

/// Parse a keybinding string into a Keybinding
pub fn parse_keybinding_string(s: &str) -> Option<Keybinding> {
    let parts: Vec<&str> = s.split('+').collect();
    
    if parts.is_empty() {
        return None;
    }
    
    let mut modifiers = Vec::new();
    let key = parts.last()?.to_string();
    
    for part in &parts[..parts.len() - 1] {
        modifiers.push(part.to_string());
    }
    
    Some(Keybinding { key, modifiers })
}

/// Convert a KeyCode to a string
fn key_code_to_string(code: KeyCode) -> String {
    match code {
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::Home => "home".to_string(),
        KeyCode::End => "end".to_string(),
        KeyCode::PageUp => "pageup".to_string(),
        KeyCode::PageDown => "pagedown".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::BackTab => "backtab".to_string(),
        KeyCode::Delete => "delete".to_string(),
        KeyCode::Insert => "insert".to_string(),
        KeyCode::F(n) => format!("f{}", n),
        KeyCode::Char(c) => {
            if c.is_control() {
                // Handle control characters
                match c as u8 {
                    0 => "null".to_string(),
                    1..=26 => format!("ctrl+{}", (b'a' + c as u8 - 1) as char),
                    _ => c.to_string(),
                }
            } else {
                c.to_string()
            }
        }
        KeyCode::Null => "null".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::CapsLock => "capslock".to_string(),
        KeyCode::ScrollLock => "scrolllock".to_string(),
        KeyCode::NumLock => "numlock".to_string(),
        KeyCode::PrintScreen => "printscreen".to_string(),
        KeyCode::Pause => "pause".to_string(),
        KeyCode::Menu => "menu".to_string(),
        KeyCode::KeypadBegin => "keypadbegin".to_string(),
        KeyCode::Media(_) => "media".to_string(),
        KeyCode::Modifier(_) => "modifier".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybinding_matches() {
        let kb = Keybinding::new("c").with_modifier("ctrl");
        assert!(kb.matches(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!kb.matches(KeyCode::Char('c'), KeyModifiers::NONE));
        assert!(!kb.matches(KeyCode::Char('C'), KeyModifiers::SHIFT));
    }

    #[test]
    fn test_keybinding_display() {
        let kb = Keybinding::new("c").with_modifier("ctrl");
        assert_eq!(kb.to_string(), "Ctrl+C");
        
        let kb = Keybinding::new("a").with_modifier("ctrl").with_modifier("shift");
        assert_eq!(kb.to_string(), "Ctrl+Shift+A");
        
        let kb = Keybinding::new("up");
        assert_eq!(kb.to_string(), "↑");
    }

    #[test]
    fn test_keybindings_get_action() {
        let kb_config = KeybindingsConfig::default();
        
        // Test Ctrl+C
        assert_eq!(
            kb_config.get_action(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Some(Action::ForceQuit)
        );
        
        // Test Up arrow
        assert_eq!(
            kb_config.get_action(KeyCode::Up, KeyModifiers::NONE),
            Some(Action::MoveUp)
        );
        
        // Test Enter
        assert_eq!(
            kb_config.get_action(KeyCode::Enter, KeyModifiers::NONE),
            Some(Action::Execute)
        );
    }

    #[test]
    fn test_parse_keybinding_string() {
        assert_eq!(
            parse_keybinding_string("ctrl+c"),
            Some(Keybinding {
                key: "c".to_string(),
                modifiers: vec!["ctrl".to_string()]
            })
        );
        
        assert_eq!(
            parse_keybinding_string("up"),
            Some(Keybinding {
                key: "up".to_string(),
                modifiers: vec![]
            })
        );
        
        assert_eq!(
            parse_keybinding_string("ctrl+shift+t"),
            Some(Keybinding {
                key: "t".to_string(),
                modifiers: vec!["ctrl".to_string(), "shift".to_string()]
            })
        );
    }

    #[test]
    fn test_keybindings_set_binding() {
        let mut kb_config = KeybindingsConfig::default();
        let kb = Keybinding::new("x").with_modifier("ctrl");
        kb_config.set_binding(kb, Action::Custom("my_action".to_string()));
        
        assert_eq!(
            kb_config.get_action(KeyCode::Char('x'), KeyModifiers::CONTROL),
            Some(Action::Custom("my_action".to_string()))
        );
    }

    #[test]
    fn test_default_keybindings() {
        let kb_config = KeybindingsConfig::default();
        assert!(kb_config.bindings.contains_key("enter"));
        assert!(kb_config.bindings.contains_key("ctrl+c"));
        assert!(kb_config.bindings.contains_key("ctrl+space"));
    }
}
