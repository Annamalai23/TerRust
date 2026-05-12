//! PTY (pseudo-terminal) implementation for shell process management
//!
//! Provides cross-platform PTY creation and management for spawning
//! shell processes with proper terminal emulation.

use super::ansi::{AnsiParser, AnsiSequence};
use super::buffer::{Grid, ScrollbackBuffer};
use super::cell::Cell;
use super::color::Color;
use super::config::ShellConfig;
use anyhow::{Context, Result};
use nix::sys::wait::waitpid;
use nix::unistd::{close, execvp, fork, ForkResult, Pid};
use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};

/// Default terminal size for PTY
pub const DEFAULT_COLUMNS: u16 = 80;
pub const DEFAULT_ROWS: u16 = 24;

/// Terminal structure combining PTY, grid, scrollback, and parser
pub struct Terminal {
    /// PTY master file descriptor
    pty_master: RawFd,
    /// Child process PID
    child_pid: Pid,
    /// Terminal grid
    pub grid: Grid,
    /// Scrollback buffer
    pub scrollback: ScrollbackBuffer,
    /// ANSI parser
    pub parser: AnsiParser,
    /// Current cursor state
    pub cursor: super::ansi::Cursor,
    /// Shell configuration
    pub shell_config: ShellConfig,
    /// Terminal columns
    pub columns: u16,
    /// Terminal rows
    pub rows: u16,
    /// Exit code of child process (if exited)
    pub exit_code: Option<i32>,
    /// Whether exit code has been captured
    exit_captured: bool,
}

impl Terminal {
    /// Create a new terminal with a spawned shell process
    pub fn new(shell_config: ShellConfig, columns: u16, rows: u16) -> Result<Self> {
        // Create PTY
        let (pty_master, pty_slave) = Self::create_pty()?;
        
        // Fork and spawn shell in child process
        let child_pid = Self::spawn_shell(pty_slave, &shell_config)?;
        
        // Close slave side in parent
        close(pty_slave).context("Failed to close PTY slave")?;
        
        // Set non-blocking mode on master.
        let flags = unsafe { libc::fcntl(pty_master, libc::F_GETFL) };
        if flags < 0 {
            return Err(anyhow::anyhow!("Failed to get PTY flags: {}", std::io::Error::last_os_error()));
        }
        let result = unsafe { libc::fcntl(pty_master, libc::F_SETFL, flags | libc::O_NONBLOCK) };
        if result < 0 {
            return Err(anyhow::anyhow!("Failed to set PTY non-blocking: {}", std::io::Error::last_os_error()));
        }
        
        // Configure terminal size
        Self::set_pty_size(pty_master, columns, rows)?;
        
        // Create terminal state
        let grid = Grid::new(columns, rows);
        let scrollback = ScrollbackBuffer::new(10000, columns);
        let parser = AnsiParser::new();
        let cursor = super::ansi::Cursor::default();
        
        Ok(Self {
            pty_master,
            child_pid,
            grid,
            scrollback,
            parser,
            cursor,
            shell_config,
            columns,
            rows,
            exit_code: None,
            exit_captured: false,
        })
    }
    
    /// Create a PTY pair (master and slave file descriptors)
    fn create_pty() -> Result<(RawFd, RawFd)> {
        let mut master_fd: RawFd = -1;
        let mut slave_fd: RawFd = -1;
        let result = unsafe {
            libc::openpty(
                &mut master_fd,
                &mut slave_fd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if result < 0 {
            return Err(anyhow::anyhow!("Failed to create PTY: {}", std::io::Error::last_os_error()));
        }
        Ok((master_fd, slave_fd))
    }
    
    /// Spawn shell process in child with PTY as controlling terminal
    fn spawn_shell(pty_slave: RawFd, shell_config: &ShellConfig) -> Result<Pid> {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                // Parent process - return child PID
                Ok(child)
            }
            Ok(ForkResult::Child) => {
                // Child process - set up shell
                Self::setup_child_process(pty_slave, shell_config)?;
                // This should never return
                unreachable!()
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to fork: {}", e))
            }
        }
    }
    
    /// Set up child process to run the shell
    fn setup_child_process(pty_slave: RawFd, shell_config: &ShellConfig) -> Result<()> {
        unsafe {
            nix::unistd::setsid().context("Failed to setsid")?;

            // Make slave the controlling terminal
            if libc::ioctl(pty_slave, libc::TIOCSCTTY as libc::c_ulong, 0) < 0 {
                return Err(anyhow::anyhow!("Failed to set controlling terminal: {}", std::io::Error::last_os_error()));
            }

            if libc::dup2(pty_slave, 0) < 0 || libc::dup2(pty_slave, 1) < 0 || libc::dup2(pty_slave, 2) < 0 {
                return Err(anyhow::anyhow!("Failed to attach PTY to stdio: {}", std::io::Error::last_os_error()));
            }
            
            // Close the original slave fd since we've duplicated it
            close(pty_slave).ok();
            
            // Close all other file descriptors we might have inherited
            // This is a bit aggressive but ensures clean state
            for fd in 3..=100 {
                close(fd).ok();
            }
            
            // Convert shell and args to CStrings
            let shell = CString::new(shell_config.shell.clone())
                .context("Invalid shell path")?;
            
            let mut args: Vec<CString> = vec![shell.clone()];
            for arg in &shell_config.args {
                args.push(CString::new(arg.clone())
                    .context(format!("Invalid arg: {}", arg))?);
            }
            // Set environment
            // Set TERM environment variable
            std::env::set_var("TERM", "xterm-256color");
            
            // Execute the shell
            execvp(&shell, &args)
                .context(format!("Failed to exec shell: {:?}", shell_config.shell))?;
        }
        
        // If we get here, exec failed
        std::process::exit(1);
    }
    
    /// Set the terminal size
    pub fn set_size(&mut self, columns: u16, rows: u16) -> Result<()> {
        Self::set_pty_size(self.pty_master, columns, rows)?;
        self.columns = columns;
        self.rows = rows;
        self.grid.resize(columns, rows);
        self.scrollback.resize(columns);
        Ok(())
    }
    
    /// Set PTY window size using ioctl
    fn set_pty_size(fd: RawFd, columns: u16, rows: u16) -> Result<()> {
        // Create winsize structure
        let mut winsize: libc::winsize = unsafe { std::mem::zeroed() };
        winsize.ws_col = columns;
        winsize.ws_row = rows;
        winsize.ws_xpixel = 0;
        winsize.ws_ypixel = 0;
        
        // Use TIOCSWINSZ ioctl to set window size
        unsafe {
            let result = libc::ioctl(fd, libc::TIOCSWINSZ, &winsize);
            if result < 0 {
                let err = std::io::Error::last_os_error();
                return Err(anyhow::anyhow!("Failed to set PTY size: {}", err));
            }
        }
        
        Ok(())
    }
    
    /// Read data from PTY with timeout
    pub fn read_with_timeout(&self, timeout: Duration) -> Result<Vec<u8>> {
        let mut buf = [0u8; 4096];

        let mut read_fds = unsafe { std::mem::zeroed::<libc::fd_set>() };
        unsafe { libc::FD_ZERO(&mut read_fds) };
        unsafe { libc::FD_SET(self.pty_master, &mut read_fds) };

        let mut timeval = libc::timeval {
            tv_sec: timeout.as_secs() as libc::time_t,
            tv_usec: timeout.subsec_micros() as libc::suseconds_t,
        };

        let ready = unsafe {
            libc::select(
                self.pty_master + 1,
                &mut read_fds,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut timeval,
            )
        };

        if ready == 0 {
            return Ok(Vec::new());
        }
        if ready < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                return Ok(Vec::new());
            }
            return Err(anyhow::anyhow!("Select error: {}", err));
        }

        let count = unsafe { libc::read(self.pty_master, buf.as_mut_ptr().cast(), buf.len()) };
        if count > 0 {
            Ok(buf[..count as usize].to_vec())
        } else if count == 0 {
            Ok(Vec::new())
        } else {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::WouldBlock || err.kind() == std::io::ErrorKind::Interrupted {
                Ok(Vec::new())
            } else {
                Err(anyhow::anyhow!("Read error: {}", err))
            }
        }
    }
    
    /// Read data from PTY non-blocking
    pub fn read_nonblocking(&self) -> Result<Vec<u8>> {
        self.read_with_timeout(Duration::from_millis(0))
    }
    
    /// Write data to PTY
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        let mut written = 0;
        
        while written < data.len() {
            let count = unsafe {
                libc::write(
                    self.pty_master,
                    data[written..].as_ptr().cast(),
                    data.len() - written,
                )
            };
            if count > 0 {
                written += count as usize;
            } else if count < 0 {
                let e = std::io::Error::last_os_error();
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    // Wait a bit and retry
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                } else if e.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                } else {
                    return Err(anyhow::anyhow!("Write error: {}", e));
                }
            } else {
                break;
            }
        }
        
        Ok(written)
    }
    
    /// Process incoming data from PTY through ANSI parser
    pub fn process_data(&mut self, data: &[u8]) -> Result<()> {
        trace!("Processing {} bytes of data", data.len());
        
        for &byte in data {
            let result = self.parser.parse(byte);
            
            match result {
                super::ansi::AnsiParseResult::Character(c) => {
                    self.handle_character(c);
                }
                super::ansi::AnsiParseResult::Complete(seq) => {
                    self.handle_sequence(seq)?;
                }
                super::ansi::AnsiParseResult::Osc(osc) => {
                    self.handle_osc(osc)?;
                }
                super::ansi::AnsiParseResult::Escape | super::ansi::AnsiParseResult::Ignored => {
                    // Need more data
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle a regular character
    fn handle_character(&mut self, c: char) {
        trace!("Handling character: {:?} at ({}, {})", c, self.cursor.column, self.cursor.row);
        if c == '\n' {
            self.cursor.column = 0;
            self.cursor.row += 1;
            if self.cursor.row >= self.rows {
                self.grid.scroll_up();
                self.cursor.row = self.rows - 1;
            }
            return;
        }
        if c == '\r' {
            self.cursor.column = 0;
            return;
        }
        
        let cell = Cell {
            character: c,
            foreground: None,
            background: None,
            attributes: super::cell::Attributes::default(),
        };
        
        // Place character at cursor position
        self.grid.set(self.cursor.column, self.cursor.row, cell);
        
        // Advance cursor
        self.cursor.column += 1;
        
        // Handle line wrapping
        if self.cursor.column >= self.columns {
            self.cursor.column = 0;
            self.cursor.row += 1;
            
            if self.cursor.row >= self.rows {
                // Save current bottom line to scrollback
                let last_row = self.rows - 1;
                let mut line = Vec::with_capacity(self.columns as usize);
                for col in 0..self.columns {
                    if let Some(cell) = self.grid.get(col, last_row) {
                        line.push(cell.clone());
                    } else {
                        line.push(Cell::default());
                    }
                }
                self.scrollback.push_line(line);
                
                // Scroll up
                self.grid.scroll_up();
                self.cursor.row = self.rows - 1;
            }
        }
    }
    
    /// Handle an ANSI sequence
    fn handle_sequence(&mut self, seq: AnsiSequence) -> Result<()> {
        trace!("Handling ANSI sequence: {:?}", seq);
        
        use super::ansi::*;
        
        match seq {
            AnsiSequence::CursorUp(n) => {
                self.cursor.row = self.cursor.row.saturating_sub(n);
            }
            AnsiSequence::CursorDown(n) => {
                self.cursor.row = std::cmp::min(self.cursor.row.saturating_add(n), self.rows - 1);
            }
            AnsiSequence::CursorForward(n) => {
                self.cursor.column = std::cmp::min(self.cursor.column.saturating_add(n), self.columns - 1);
            }
            AnsiSequence::CursorBackward(n) => {
                self.cursor.column = self.cursor.column.saturating_sub(n);
            }
            AnsiSequence::CursorNextLine(n) => {
                self.cursor.row = std::cmp::min(self.cursor.row.saturating_add(n), self.rows - 1);
                self.cursor.column = 0;
            }
            AnsiSequence::CursorPreviousLine(n) => {
                self.cursor.row = self.cursor.row.saturating_sub(n);
                self.cursor.column = 0;
            }
            AnsiSequence::CursorHorizontalAbsolute(col) => {
                self.cursor.column = std::cmp::min(col.saturating_sub(1), self.columns - 1);
            }
            AnsiSequence::CursorPosition(row, col) => {
                self.cursor.row = std::cmp::min(row.saturating_sub(1), self.rows - 1);
                self.cursor.column = std::cmp::min(col.saturating_sub(1), self.columns - 1);
            }
            AnsiSequence::SaveCursor => {
                // Save cursor state - simplified for now
            }
            AnsiSequence::RestoreCursor => {
                // Restore cursor state - simplified for now
            }
            AnsiSequence::CursorStyle(style) => {
                self.cursor.style = style;
            }
            AnsiSequence::CursorVisibility(visible) => {
                self.cursor.visible = visible;
            }
            AnsiSequence::EraseInDisplay(mode) => {
                self.handle_erase_display(mode);
            }
            AnsiSequence::EraseInLine(mode) => {
                self.handle_erase_line(mode);
            }
            AnsiSequence::ScrollUp(n) => {
                self.handle_scroll_up(n);
            }
            AnsiSequence::ScrollDown(n) => {
                self.handle_scroll_down(n);
            }
            AnsiSequence::ClearScreen => {
                self.grid.clear();
                self.scrollback = ScrollbackBuffer::new(10000, self.columns);
                self.cursor = super::ansi::Cursor::default();
            }
            AnsiSequence::ClearLine => {
                self.grid.clear_line(self.cursor.row);
            }
            AnsiSequence::Sgr(attrs) => {
                self.handle_sgr(attrs);
            }
            AnsiSequence::SetWindowTitle(title) => {
                info!("Window title set to: {}", title);
            }
            AnsiSequence::SetMode(mode) => {
                trace!("Terminal mode {} set", mode as i32);
            }
            AnsiSequence::ResetMode(mode) => {
                trace!("Terminal mode {} reset", mode as i32);
            }
            AnsiSequence::AlternateScreenBuffer(enable) => {
                trace!("Alternate screen buffer: {}", enable);
                // Simplified - just clear screen for now
                if enable {
                    self.grid.clear();
                }
            }
            AnsiSequence::ReportCursorPosition => {
                // Report cursor position - simplified
            }
            AnsiSequence::ReportTerminalType => {
                // Report terminal type - simplified
            }
            AnsiSequence::Index => {
                // Line feed - scroll if at bottom
                if self.cursor.row >= self.rows - 1 {
                    // Save current bottom line to scrollback
                    let last_row = self.rows - 1;
                    let mut line = Vec::with_capacity(self.columns as usize);
                    for col in 0..self.columns {
                        if let Some(cell) = self.grid.get(col, last_row) {
                            line.push(cell.clone());
                        } else {
                            line.push(Cell::default());
                        }
                    }
                    self.scrollback.push_line(line);
                    
                    self.grid.scroll_up();
                } else {
                    self.cursor.row += 1;
                }
                self.cursor.column = 0;
            }
            AnsiSequence::ReverseIndex => {
                // Reverse line feed
                if self.cursor.row > 0 {
                    self.cursor.row -= 1;
                }
                self.cursor.column = 0;
            }
            AnsiSequence::FullReset => {
                self.grid.clear();
                self.scrollback = ScrollbackBuffer::new(10000, self.columns);
                self.cursor = super::ansi::Cursor::default();
                self.parser.reset();
            }
            AnsiSequence::Reset => {
                self.cursor = super::ansi::Cursor::default();
            }
            AnsiSequence::Unknown => {}
        }
        
        Ok(())
    }
    
    /// Handle erase in display
    fn handle_erase_display(&mut self, mode: super::ansi::EraseDisplayMode) {
        match mode {
            super::ansi::EraseDisplayMode::FromCursorToEnd => {
                // Clear from cursor to end of screen
                for row in self.cursor.row..self.rows {
                    self.grid.clear_row(row);
                }
            }
            super::ansi::EraseDisplayMode::FromCursorToStart => {
                // Clear from cursor to start of screen
                for row in 0..=self.cursor.row {
                    self.grid.clear_row(row);
                }
            }
            super::ansi::EraseDisplayMode::All => {
                self.grid.clear();
            }
        }
    }
    
    /// Handle erase in line
    fn handle_erase_line(&mut self, mode: super::ansi::EraseLineMode) {
        match mode {
            super::ansi::EraseLineMode::FromCursorToEnd => {
                self.grid.clear_to_end_of_line(self.cursor.column, self.cursor.row);
            }
            super::ansi::EraseLineMode::FromCursorToStart => {
                self.grid.clear_to_start_of_line(self.cursor.column, self.cursor.row);
            }
            super::ansi::EraseLineMode::All => {
                self.grid.clear_line(self.cursor.row);
            }
        }
    }
    
    /// Handle scroll up
    fn handle_scroll_up(&mut self, n: u16) {
        for _ in 0..n {
            // Save top line to scrollback
            let mut line = Vec::with_capacity(self.columns as usize);
            for col in 0..self.columns {
                if let Some(cell) = self.grid.get(col, 0) {
                    line.push(cell.clone());
                } else {
                    line.push(Cell::default());
                }
            }
            self.scrollback.push_line(line);
            
            self.grid.scroll_up();
        }
    }
    
    /// Handle scroll down
    fn handle_scroll_down(&mut self, n: u16) {
        for _ in 0..n {
            // Remove last line from scrollback and add to top
            if self.scrollback.len() > 0 {
                if let Some(line) = self.scrollback.get_line(0) {
                    // Shift grid down
                    for row in (1..self.rows).rev() {
                        for col in 0..self.columns {
                            if let Some(cell) = self.grid.get(col, row - 1) {
                                self.grid.set(col, row, cell.clone());
                            }
                        }
                    }
                    
                    // Place scrollback line at top
                    for col in 0..self.columns {
                        if usize::from(col) < line.len() {
                            self.grid.set(col, 0, line[col as usize].clone());
                        } else {
                            self.grid.set(col, 0, Cell::default());
                        }
                    }
                    
                    // Remove from scrollback
                    // Note: ScrollbackBuffer doesn't support removing from front yet
                    // This is a simplification
                }
            }
        }
    }
    
    /// Handle SGR (Select Graphic Rendition) attributes
    fn handle_sgr(&mut self, attrs: Vec<super::ansi::SgrAttribute>) {
        // For now, just apply to next character
        // Full implementation would track current attributes
        for attr in attrs {
            match attr {
                super::ansi::SgrAttribute::Reset => {
                    // Reset all attributes
                }
                super::ansi::SgrAttribute::Bold => {
                    // Set bold
                }
                super::ansi::SgrAttribute::ForegroundColor(color) => {
                    // Set foreground color
                }
                super::ansi::SgrAttribute::BackgroundColor(color) => {
                    // Set background color
                }
                _ => {
                    // Other attributes
                }
            }
        }
    }
    
    /// Handle OSC (Operating System Command) sequence
    fn handle_osc(&mut self, osc: super::ansi::OscSequence) -> Result<()> {
        match osc {
            super::ansi::OscSequence::SetWindowTitle(title) => {
                info!("Window title: {}", title);
            }
            super::ansi::OscSequence::Hyperlink(url) => {
                debug!("Hyperlink: {}", url);
            }
            super::ansi::OscSequence::SetIconName(name) => {
                debug!("Icon name: {}", name);
            }
            super::ansi::OscSequence::SetWindowTitleAndIconName(title) => {
                debug!("Window title and icon name: {}", title);
            }
            super::ansi::OscSequence::ResetWindowTitle | super::ansi::OscSequence::ResetIconName => {}
            super::ansi::OscSequence::Unknown => {
                // Ignore unknown OSC sequences
            }
        }
        
        Ok(())
    }
    
    /// Check if the child process is still alive
    pub fn is_child_alive(&mut self) -> bool {
        match waitpid(self.child_pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
            Ok(nix::sys::wait::WaitStatus::Exited(_, code)) => {
                self.exit_code = Some(code);
                false
            }
            Ok(nix::sys::wait::WaitStatus::Signaled(_, _, _)) => false,
            Ok(nix::sys::wait::WaitStatus::Stopped(_, _)) => true,
            Ok(nix::sys::wait::WaitStatus::Continued(_)) => true,
            Ok(nix::sys::wait::WaitStatus::StillAlive) => true,
            Err(_) => true, // Error means we couldn't check, assume alive
        }
    }
    
    /// Get the exit code of the child process
    pub fn get_exit_code(&mut self) -> Option<i32> {
        if self.exit_captured {
            return self.exit_code;
        }
        
        // Try to get the exit code
        match waitpid(self.child_pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
            Ok(nix::sys::wait::WaitStatus::Exited(_, code)) => {
                self.exit_code = Some(code);
                self.exit_captured = true;
                self.exit_code
            }
            Ok(nix::sys::wait::WaitStatus::Signaled(_, sig, _)) => {
                // Process was killed by a signal (128 + signal number)
                self.exit_code = Some(128 + sig as i32);
                self.exit_captured = true;
                self.exit_code
            }
            _ => None,
        }
    }
    
    /// Get the child process PID
    pub fn child_pid(&self) -> Pid {
        self.child_pid
    }
    
    /// Get the PTY master file descriptor
    pub fn pty_master(&self) -> RawFd {
        self.pty_master
    }
    
    /// Send SIGHUP to child process (typically for terminal close)
    pub fn send_sighup(&self) -> Result<()> {
        use nix::sys::signal::{kill, Signal};
        
        kill(self.child_pid, Signal::SIGHUP)
            .context("Failed to send SIGHUP to child process")?;
        
        Ok(())
    }
    
    /// Send SIGTERM to child process
    pub fn send_sigterm(&self) -> Result<()> {
        use nix::sys::signal::{kill, Signal};
        
        kill(self.child_pid, Signal::SIGTERM)
            .context("Failed to send SIGTERM to child process")?;
        
        Ok(())
    }
    
    /// Wait for child process to exit
    pub fn wait_for_child(&self) -> Result<nix::sys::wait::WaitStatus> {
        waitpid(self.child_pid, None)
            .context("Failed to wait for child process")
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        trace!("Dropping Terminal - cleaning up resources");
        
        // Try to close PTY master
        close(self.pty_master).ok();
        
        // Try to wait for child process
        let _ = self.wait_for_child();
    }
}

/// Simplified terminal for testing without actual PTY
#[cfg(test)]
pub struct MockTerminal {
    pub grid: Grid,
    pub scrollback: ScrollbackBuffer,
    pub parser: AnsiParser,
    pub cursor: super::ansi::Cursor,
    pub columns: u16,
    pub rows: u16,
}

#[cfg(test)]
impl MockTerminal {
    pub fn new(columns: u16, rows: u16) -> Self {
        Self {
            grid: Grid::new(columns, rows),
            scrollback: ScrollbackBuffer::new(10000, columns),
            parser: AnsiParser::new(),
            cursor: super::ansi::Cursor::default(),
            columns,
            rows,
        }
    }
    
    pub fn process_data(&mut self, data: &[u8]) -> Result<()> {
        use super::ansi::*;
        
        for &byte in data {
            match self.parser.parse(byte) {
                AnsiParseResult::Character(c) => {
                    self.handle_character(c);
                }
                AnsiParseResult::Complete(seq) => {
                    self.handle_sequence(seq)?;
                }
                AnsiParseResult::Osc(osc) => {
                    self.handle_osc(osc)?;
                }
                AnsiParseResult::Escape | AnsiParseResult::Ignored => {}
            }
        }
        
        Ok(())
    }
    
    fn handle_character(&mut self, c: char) {
        if c == '\n' {
            self.cursor.column = 0;
            self.cursor.row += 1;
            if self.cursor.row >= self.rows {
                self.grid.scroll_up();
                self.cursor.row = self.rows - 1;
            }
            return;
        }
        if c == '\r' {
            self.cursor.column = 0;
            return;
        }
        let cell = Cell {
            character: c,
            foreground: None,
            background: None,
            attributes: super::cell::Attributes::default(),
        };
        
        self.grid.set(self.cursor.column, self.cursor.row, cell);
        self.cursor.column += 1;
        
        if self.cursor.column >= self.columns {
            self.cursor.column = 0;
            self.cursor.row += 1;
            
            if self.cursor.row >= self.rows {
                let last_row = self.rows - 1;
                let mut line = Vec::with_capacity(self.columns as usize);
                for col in 0..self.columns {
                    if let Some(cell) = self.grid.get(col, last_row) {
                        line.push(cell.clone());
                    } else {
                        line.push(Cell::default());
                    }
                }
                self.scrollback.push_line(line);
                self.grid.scroll_up();
                self.cursor.row = self.rows - 1;
            }
        }
    }
    
    fn handle_sequence(&mut self, seq: AnsiSequence) -> Result<()> {
        use super::ansi::*;
        
        match seq {
            AnsiSequence::CursorPosition(row, col) => {
                self.cursor.row = std::cmp::min(row.saturating_sub(1), self.rows - 1);
                self.cursor.column = std::cmp::min(col.saturating_sub(1), self.columns - 1);
            }
            AnsiSequence::ClearScreen => {
                self.grid.clear();
            }
            AnsiSequence::ClearLine => {
                self.grid.clear_line(self.cursor.row);
            }
            AnsiSequence::Sgr(attrs) => {
                for attr in attrs {
                    match attr {
                        SgrAttribute::Reset => {}
                        SgrAttribute::ForegroundColor(color) => {
                            // Would set current fg color
                        }
                        SgrAttribute::BackgroundColor(color) => {
                            // Would set current bg color
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn handle_osc(&mut self, _osc: super::ansi::OscSequence) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ansi::*;
    
    #[test]
    fn test_mock_terminal_basic_text() -> Result<()> {
        let mut term = MockTerminal::new(10, 5);
        
        // Process simple text
        let text = b"Hello, World!";
        term.process_data(text)?;
        
        // Check that characters were placed correctly
        assert_eq!(term.grid.get(0, 0).unwrap().character, 'H');
        assert_eq!(term.grid.get(1, 0).unwrap().character, 'e');
        assert_eq!(term.grid.get(2, 0).unwrap().character, 'l');
        
        Ok(())
    }
    
    #[test]
    fn test_mock_terminal_line_wrapping() -> Result<()> {
        let mut term = MockTerminal::new(5, 3);
        
        // Write more than fits in one line
        let text = b"Hello World";
        term.process_data(text)?;
        
        // Should wrap to next line
        assert_eq!(term.cursor.row, 2);
        assert_eq!(term.cursor.column, 1);
        
        // Three rows are enough for this input, so no scrollback should be created.
        assert!(term.scrollback.is_empty());
        
        Ok(())
    }
    
    #[test]
    fn test_mock_terminal_clear_screen() -> Result<()> {
        let mut term = MockTerminal::new(10, 5);
        
        // Write some text
        term.process_data(b"Hello")?;
        assert!(!term.grid.get(0, 0).unwrap().character.is_whitespace());
        
        // Clear screen
        term.process_data(b"\x1B[2J")?;
        
        // Grid should be cleared
        assert!(term.grid.get(0, 0).unwrap().character.is_whitespace());
        
        Ok(())
    }
    
    #[test]
    fn test_mock_terminal_cursor_position() -> Result<()> {
        let mut term = MockTerminal::new(10, 5);
        
        // Move cursor to position (2, 3)
        term.process_data(b"\x1B[3;2H")?;
        
        assert_eq!(term.cursor.row, 2);
        assert_eq!(term.cursor.column, 1); // 0-indexed
        
        Ok(())
    }
    
    #[test]
    fn test_mock_terminal_newline() -> Result<()> {
        let mut term = MockTerminal::new(10, 5);
        
        // Write text and newline
        term.process_data(b"Hello\nWorld")?;
        
        assert_eq!(term.cursor.row, 1);
        assert_eq!(term.cursor.column, 5);
        
        Ok(())
    }
    
    #[test]
    fn test_ansi_parser_integration() {
        let mut parser = AnsiParser::new();
        
        // Test cursor movement
        let result = parser.parse(b'\x1B');
        assert!(matches!(result, AnsiParseResult::Escape));
        
        let result = parser.parse(b'[');
        assert!(matches!(result, AnsiParseResult::Escape));
        
        let result = parser.parse(b'2');
        assert!(matches!(result, AnsiParseResult::Escape));
        
        let result = parser.parse(b'J');
        assert!(matches!(result, AnsiParseResult::Complete(AnsiSequence::ClearScreen)));
    }
}
