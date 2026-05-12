//! Application state and main event loop for TerRust
//!
//! This module contains the core application logic including:
//! - App struct with all application state
//! - Event loop for handling keyboard, resize, and paste events
//! - Terminal management
//! - Integration with PTY, config, and UI modules

use crate::config::Config;
use crate::terminal::Terminal;
use crate::ui::{BlockManager, InputBar};
#[cfg(feature = "ai")]
use crate::ai::AI;
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal as RatatuiTerminal;
use std::io::stdout;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

/// Application state structure
pub struct App {
    /// Configuration
    config: Config,
    /// Terminal emulation
    terminal: Option<Terminal>,
    /// UI block manager
    block_manager: BlockManager,
    /// Input bar
    input_bar: InputBar,
    /// Output block currently mirroring terminal output
    current_output_block: Option<Uuid>,
    /// Command block for the current executing command
    current_command_block: Option<Uuid>,
    /// Start time of current command (for duration tracking)
    command_start_time: Option<std::time::Instant>,
    /// Whether AI features are enabled
    ai_enabled: bool,
    /// AI client for AI assistance
    #[cfg(feature = "ai")]
    ai_client: Option<AI>,
    /// Whether AI mode is active (waiting for AI prompt)
    ai_mode: bool,
    /// Whether to run in fullscreen mode
    fullscreen: bool,
    /// Plugin directory
    plugin_dir: Option<PathBuf>,
    /// Event sender for background tasks
    event_sender: Option<Sender<AppEvent>>,
    /// Event receiver
    event_receiver: Receiver<AppEvent>,
    /// Whether the application is running
    running: bool,
    /// Terminal scroll offset (lines scrolled up from bottom)
    scroll_offset: usize,
    /// Whether terminal scroll mode is active
    scroll_mode: bool,
}

/// Application events
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Key pressed
    Key(KeyCode, KeyModifiers),
    /// Terminal resized
    Resize(u16, u16),
    /// Paste event with text
    Paste(String),
    /// Quit application
    Quit,
    /// Execute command
    Execute(String),
    /// Terminal output received
    TerminalOutput(Vec<u8>),
    /// Child process exited
    ChildExited(i32),
}

impl App {
    /// Create a new App instance
    pub fn new(
        config: Config,
        ai_enabled: bool,
        fullscreen: bool,
        plugin_dir: Option<PathBuf>,
    ) -> Result<Self> {
        info!(
            "Initializing TerRust application (ai_enabled={}, fullscreen={})",
            ai_enabled, fullscreen
        );

        // Initialize UI components
        let block_manager = BlockManager::new(1000, config.general.scrollback_limit);
        let input_bar = InputBar::new("$ ");

        // Set up event channel
        let (sender, receiver) = channel();

        // Initialize AI client if enabled
        #[cfg(feature = "ai")]
        let ai_client = if ai_enabled && config.ai.enabled {
            match crate::ai::ProviderFactory::create(&config.ai) {
                Ok(provider) => Some(crate::ai::AI::new(provider)),
                Err(e) => {
                    tracing::warn!("Failed to initialize AI client: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            config,
            terminal: None,
            block_manager,
            input_bar,
            current_output_block: None,
            current_command_block: None,
            command_start_time: None,
            ai_enabled,
            #[cfg(feature = "ai")]
            ai_client,
            #[cfg(feature = "ai")]
            ai_mode: false,
            #[cfg(not(feature = "ai"))]
            ai_mode: false,
            fullscreen,
            plugin_dir,
            event_sender: Some(sender),
            event_receiver: receiver,
            running: false,
            scroll_offset: 0,
            scroll_mode: false,
        })
    }

    /// Run the main application event loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting main event loop");

        // Initialize terminal
        self.initialize_terminal()?;

        // Initialize ratatui terminal
        let mut stdout = stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(stdout);
        let mut ratatui_terminal = RatatuiTerminal::new(backend)?;
        ratatui_terminal.hide_cursor()?;

        self.running = true;

        // Start PTY reader thread
        self.start_pty_reader()?;

        // Main event loop
        while self.running {
            while event::poll(Duration::from_millis(0))? {
                match event::read()? {
                    Event::Key(key) => self.handle_event(
                        AppEvent::Key(key.code, key.modifiers),
                        &mut ratatui_terminal,
                    )?,
                    Event::Resize(cols, rows) => {
                        self.handle_event(AppEvent::Resize(cols, rows), &mut ratatui_terminal)?
                    }
                    Event::Paste(text) => {
                        self.handle_event(AppEvent::Paste(text), &mut ratatui_terminal)?
                    }
                    _ => {}
                }
            }

            // Check for internal events with timeout
            if let Ok(event) = self.event_receiver.recv_timeout(Duration::from_millis(16)) {
                self.handle_event(event, &mut ratatui_terminal)?;
            }

            // Check for terminal output
            self.process_terminal_output(&mut ratatui_terminal)?;

            // Render UI
            self.render(&mut ratatui_terminal)?;
        }

        // Cleanup
        self.cleanup_terminal(&mut ratatui_terminal)?;

        Ok(())
    }

    /// Initialize the PTY terminal
    fn initialize_terminal(&mut self) -> Result<()> {
        use crate::terminal::ShellConfig;

        let columns = 80;
        let rows = 24;

        let shell_config = ShellConfig {
            shell: self.config.terminal.shell.clone(),
            args: self.config.terminal.shell_args.clone(),
        };

        let term =
            Terminal::new(shell_config, columns, rows).context("Failed to create terminal")?;

        self.terminal = Some(term);
        info!(
            "Terminal initialized with shell: {}",
            self.config.terminal.shell
        );

        Ok(())
    }

    /// Start PTY reader thread
    fn start_pty_reader(&mut self) -> Result<()> {
        Ok(())
    }

/// Process terminal output
    fn process_terminal_output(
        &mut self,
        terminal: &mut RatatuiTerminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let content = if let Some(ref mut term) = self.terminal {
            let data = term.read_nonblocking()?;
            let content = if !data.is_empty() {
                term.process_data(&data)?;
                Some(term.grid.cells.clone())
            } else {
                None
            };
            
            // Check for child exit and capture exit code
            let exit_code = term.get_exit_code();
            let child_alive = term.is_child_alive();
            if !child_alive {
                // Child exited - capture exit code and duration
                self.capture_command_completion(exit_code.unwrap_or(-1));
                self.running = false;
            }
            content
        } else {
            None
        };

        if let Some(content) = content {
            // Check for prompt return (command completion)
            self.check_command_completion(&content);
            self.sync_terminal_output_block(content);
        }

        Ok(())
    }
    
    /// Capture command completion (called when child exits or prompt returns)
    fn capture_command_completion(&mut self, exit_code: i32) {
        if let Some(start_time) = self.command_start_time {
            let duration = start_time.elapsed();
            
            // Update command block with exit code and duration
            if let Some(cmd_block_id) = self.current_command_block {
                if let Some(block) = self.block_manager.get_block_mut(cmd_block_id) {
                    block.exit_code = Some(exit_code);
                    block.set_duration(duration);
                }
            }
            
            // Reset start time
            self.command_start_time = None;
        }
    }
    
    /// Check if shell prompt has returned (indicates command completion)
    fn check_command_completion(&mut self, content: &[Vec<crate::terminal::Cell>]) {
        // Look for common shell prompts: $, #, > at end of last line
        if content.is_empty() {
            return;
        }
        
        for row in content.iter().rev().take(1) {
            for cell in row.iter().rev().take(3) {
                if cell.character == '$' || cell.character == '#' || cell.character == '>' {
                    // Likely a prompt - command completed
                    if self.command_start_time.is_some() && self.current_output_block.is_some() {
                        // Only capture if we have a tracked command
                        self.capture_command_completion(0);
                    }
                    return;
                }
            }
        }
    }

    /// Keep the latest terminal screen visible in a discrete output block.
    fn sync_terminal_output_block(&mut self, mut content: Vec<Vec<crate::terminal::Cell>>) {
        content.retain(|row| row.iter().any(|cell| cell.character != ' '));

        if content.is_empty() {
            return;
        }

        let block_id = match self.current_output_block {
            Some(id) if self.block_manager.get_block(id).is_some() => id,
            _ => {
                let id = self.block_manager.add_output_block();
                if let Some(block) = self.block_manager.get_block_mut(id) {
                    block.title = Some("Terminal Output".to_string());
                }
                self.current_output_block = Some(id);
                id
            }
        };

        if let Some(block) = self.block_manager.get_block_mut(block_id) {
            block.content = content;
            block.height = block.content.len() as u16;
        }
    }

    /// Handle an application event
    fn handle_event(
        &mut self,
        event: AppEvent,
        terminal: &mut RatatuiTerminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        match event {
            AppEvent::Key(key, mods) => {
                self.handle_key(key, mods, terminal)?;
            }
            AppEvent::Resize(cols, rows) => {
                self.handle_resize(cols, rows)?;
            }
            AppEvent::Paste(text) => {
                self.handle_paste(&text)?;
            }
            AppEvent::Quit => {
                self.running = false;
            }
            AppEvent::Execute(cmd) => {
                self.handle_execute(&cmd)?;
            }
            AppEvent::TerminalOutput(data) => {
                self.handle_terminal_output(data)?;
            }
            AppEvent::ChildExited(code) => {
                self.handle_child_exited(code)?;
            }
        }
        Ok(())
    }

    /// Handle a key press
    fn handle_key(
        &mut self,
        key: KeyCode,
        mods: KeyModifiers,
        _terminal: &mut RatatuiTerminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        trace!("Key pressed: {:?} with modifiers {:?}", key, mods);

        // Handle input bar keys first
        let handled = self.input_bar.handle_key(key, mods);
        if handled {
            // Check if we should execute
            if key == KeyCode::Enter {
                let command = self.input_bar.get_content().clone();
                if !command.is_empty() {
                    // Get current working directory and shell for metadata
                    let cwd = std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let shell = self.config.terminal.shell.clone();
                    
                    // Add command block with metadata
                    let cmd_block = self.block_manager.add_command_block(&command);
                    if let Some(block) = self.block_manager.get_block_mut(cmd_block) {
                        block.start_time = Some(chrono::Local::now());
                        block.command = Some(command.clone());
                        block.cwd = Some(cwd.clone());
                        block.shell = Some(shell.clone());
                    }
                    self.current_command_block = Some(cmd_block);
                    
                    // Add output block
                    let output_id = self.block_manager.add_output_block();
                    if let Some(block) = self.block_manager.get_block_mut(output_id) {
                        block.title = Some("Output".to_string());
                    }
                    self.current_output_block = Some(output_id);
                    
                    // Track start time for duration
                    self.command_start_time = Some(std::time::Instant::now());
                    
                    // Push to history and clear input
                    self.input_bar.history_push(command.clone());
                    self.input_bar.clear();
                    
                    // Write to terminal
                    if let Some(ref term) = self.terminal {
                        term.write(format!("{}\n", command).as_bytes())?;
                    }
                }
            }
            return Ok(());
        }

        // Handle global keys
        match (key, mods) {
            // AI mode: / to enter, Escape to cancel
            (KeyCode::Char('/'), _) if self.ai_enabled => {
                self.ai_mode = true;
                self.input_bar.set_prompt("AI: ");
                info!("Entered AI mode");
            }
            (KeyCode::Esc, _) => {
                if self.ai_mode {
                    self.ai_mode = false;
                    self.input_bar.set_prompt("$ ");
                    self.input_bar.cancel();
                    info!("Exited AI mode");
                } else {
                    self.input_bar.cancel();
                }
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.running = false;
            }
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.running = false;
            }
            // Scroll handling - PageUp/PageDown to scroll through terminal history
            (KeyCode::PageUp, _) => {
                self.scroll_mode = true;
                let term_rows = self.terminal.as_ref().map(|t| t.grid.rows as usize).unwrap_or(24);
                self.scroll_offset = self.scroll_offset.saturating_add(term_rows.saturating_sub(2));
                trace!("Scroll up: offset={}", self.scroll_offset);
            }
            (KeyCode::PageDown, _) => {
                self.scroll_offset = self.scroll_offset.saturating_sub(24);
                if self.scroll_offset == 0 {
                    self.scroll_mode = false;
                }
                trace!("Scroll down: offset={}", self.scroll_offset);
            }
            (KeyCode::Up, KeyModifiers::SHIFT) => {
                self.scroll_mode = true;
                self.scroll_offset = self.scroll_offset.saturating_add(1);
                trace!("Scroll up 1: offset={}", self.scroll_offset);
            }
            (KeyCode::Down, KeyModifiers::SHIFT) => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                if self.scroll_offset == 0 {
                    self.scroll_mode = false;
                }
                trace!("Scroll down 1: offset={}", self.scroll_offset);
            }
            // Exit scroll mode with Escape
            (KeyCode::Esc, _) if self.scroll_mode => {
                self.scroll_offset = 0;
                self.scroll_mode = false;
                trace!("Exited scroll mode");
            }
            _ => {
                trace!("Unhandled key: {:?}", key);
            }
        }

        // Handle AI mode submission
        if self.ai_mode && key == KeyCode::Enter {
            let prompt = self.input_bar.get_content().clone();
            if !prompt.is_empty() && self.ai_client.is_some() {
                // Call AI synchronously (blocking but that's OK for now)
                let ai = self.ai_client.as_ref().unwrap();
                match ai.ask(&prompt) {
                    Ok(response) => {
                        info!("AI response: {} chars", response.text.len());
                        // Convert response to cells for the block
                        let content: Vec<Vec<crate::terminal::Cell>> = response.text
                            .lines()
                            .map(|line| {
                                line.chars()
                                    .map(|c| crate::terminal::Cell {
                                        character: c,
                                        foreground: None,
                                        background: None,
                                        attributes: crate::terminal::Attributes::default(),
                                    })
                                    .collect()
                            })
                            .collect();
                        
                        let ai_block = self.block_manager.add_ai_block();
                        if let Some(block) = self.block_manager.get_block_mut(ai_block) {
                            block.title = Some("AI Response".to_string());
                            block.content = content;
                            block.height = block.content.len() as u16;
                        }
                    }
                    Err(e) => {
                        error!("AI error: {}", e);
                    }
                }
            }
            self.ai_mode = false;
            self.input_bar.set_prompt("$ ");
            self.input_bar.clear();
        }

        Ok(())
    }

    /// Handle resize event
    fn handle_resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        info!("Terminal resized to {}x{}", cols, rows);
        if let Some(ref mut term) = self.terminal {
            term.set_size(cols, rows)?;
        }
        Ok(())
    }

    /// Handle paste event
    fn handle_paste(&mut self, text: &str) -> Result<()> {
        debug!("Paste: {}", text);
        self.input_bar.insert_str(text);
        Ok(())
    }

    /// Handle execute command
    fn handle_execute(&mut self, cmd: &str) -> Result<()> {
        info!("Executing command: {}", cmd);
        // Would send command to terminal here
        Ok(())
    }

    /// Handle terminal output data
    fn handle_terminal_output(&mut self, data: Vec<u8>) -> Result<()> {
        trace!("Received {} bytes from terminal", data.len());
        if let Some(ref mut term) = self.terminal {
            term.process_data(&data)?;
        }
        Ok(())
    }

    /// Handle child process exit
    fn handle_child_exited(&mut self, code: i32) -> Result<()> {
        warn!("Child process exited with code: {}", code);
        // Would trigger restart or notification
        Ok(())
    }

    /// Render the UI
    fn render(
        &mut self,
        terminal: &mut RatatuiTerminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        terminal.draw(|f| {
            use crate::ui::render::{blocks_to_lines, grid_to_lines};
            use ratatui::layout::{Constraint, Direction, Layout};
            use ratatui::style::Style;
            use ratatui::text::{Line, Span};
            use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

            // Get theme colors
            let bg = self.config.theme.bg();
            let fg = self.config.theme.fg();

            // Clear background
            f.render_widget(
                Paragraph::new("".to_string())
                    .block(Block::default().style(Style::default().bg(bg))),
                f.size(),
            );

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(f.size());

            let mut output_lines = if self.scroll_offset > 0 {
                // Scroll mode: show scrollback + current grid
                let mut lines = Vec::new();
                if let Some(term) = self.terminal.as_ref() {
                    // Add scrollback lines (newest first, up to scroll_offset)
                    for (idx, line) in term.scrollback.iter_rev().enumerate() {
                        if idx >= self.scroll_offset {
                            break;
                        }
                        lines.push(crate::ui::render::cells_to_line(line, fg));
                    }
                    // Add current grid
                    lines.extend(grid_to_lines(&term.grid, &self.config.theme));
                }
                // Reverse to show oldest at top
                lines.reverse();
                lines
            } else if self.block_manager.is_empty() {
                self.terminal
                    .as_ref()
                    .map(|term| grid_to_lines(&term.grid, &self.config.theme))
                    .unwrap_or_default()
            } else {
                blocks_to_lines(&self.block_manager, &self.config.theme, chunks[0].width)
            };

            let visible_height = chunks[0].height.saturating_sub(2) as usize;
            if visible_height > 0 && output_lines.len() > visible_height {
                output_lines = output_lines.split_off(output_lines.len() - visible_height);
            }

            let title = if self.scroll_mode && self.scroll_offset > 0 {
                format!("TerRust [Scroll: {} lines]", self.scroll_offset)
            } else {
                "TerRust".to_string()
            };

            let output_paragraph = Paragraph::new(output_lines)
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().fg(fg).bg(bg)),
                )
                .style(Style::default().fg(fg).bg(bg));

            f.render_widget(output_paragraph, chunks[0]);

            // Draw input bar
            let input_content = self.input_bar.get_content();
            let input_span = Span::styled(
                input_content.to_string(),
                ratatui::style::Style::default().fg(fg),
            );

            let input_paragraph = Paragraph::new(Line::from(vec![input_span])).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(fg)),
            );

            let input_area = chunks[1];

            f.render_widget(input_paragraph.clone(), input_area);

            // Draw cursor if input is active
            if self.input_bar.is_active() {
                let cursor_x = input_content.len() as u16;
                f.set_cursor(input_area.x + 1 + cursor_x, input_area.y + 1);
            }
        })?;

        Ok(())
    }

    /// Cleanup terminal on exit
    fn cleanup_terminal(
        &mut self,
        terminal: &mut RatatuiTerminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        info!("Cleaning up terminal");

        // Cleanup PTY
        self.terminal = None;

        // Restore terminal state
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    /// Get reference to config (for read-only access)
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable reference to config
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Get reference to block manager
    pub fn block_manager(&self) -> &BlockManager {
        &self.block_manager
    }

    /// Get mutable reference to block manager
    pub fn block_manager_mut(&mut self) -> &mut BlockManager {
        &mut self.block_manager
    }

    /// Check if AI is enabled
    pub fn is_ai_enabled(&self) -> bool {
        self.ai_enabled
    }
}

impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            terminal: None, // Can't clone terminal with file descriptors
            block_manager: self.block_manager.clone(),
            input_bar: self.input_bar.clone(),
            current_output_block: self.current_output_block,
            current_command_block: self.current_command_block,
            command_start_time: None, // Can't clone Instant
            ai_enabled: self.ai_enabled,
            #[cfg(feature = "ai")]
            ai_client: None, // Can't clone AI
            #[cfg(feature = "ai")]
            ai_mode: self.ai_mode,
            #[cfg(not(feature = "ai"))]
            ai_mode: self.ai_mode,
            fullscreen: self.fullscreen,
            plugin_dir: self.plugin_dir.clone(),
            event_sender: None,          // Can't clone sender
            event_receiver: channel().1, // Create new receiver
            running: self.running,
            scroll_offset: self.scroll_offset,
            scroll_mode: self.scroll_mode,
        }
    }
}

/// Module-level logger

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let config = Config::default();
        let app = App::new(config, true, false, None).unwrap();
        assert!(app.is_ai_enabled());
        assert!(!app.fullscreen);
    }

    #[test]
    fn test_app_config_access() {
        let config = Config::default();
        let app = App::new(config, true, false, None).unwrap();
        assert_eq!(app.config().general.theme, "tokyo-night");
    }

    #[test]
    fn test_event_types() {
        let event = AppEvent::Key(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(matches!(event, AppEvent::Key(_, _)));

        let event = AppEvent::Resize(80, 24);
        assert!(matches!(event, AppEvent::Resize(_, _)));

        let event = AppEvent::Quit;
        assert!(matches!(event, AppEvent::Quit));
    }
}
