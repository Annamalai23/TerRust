//! PTY (pseudo-terminal) implementation for shell process management
//!
//! Provides cross-platform PTY creation and management for spawning
//! shell processes with proper terminal emulation.

use super::ansi::{AnsiParser, AnsiSequence, SgrAttribute};
use super::buffer::{Grid, ScrollbackBuffer};
use super::cell::{Attributes, Cell};
use super::color::Color;
use super::config::ShellConfig;
use anyhow::{Context, Result};
use nix::sys::wait::waitpid;
use nix::unistd::{close, execvp, fork, ForkResult, Pid};
use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::time::Duration;
use tracing::{debug, info, trace};

/// Close all inherited file descriptors above 2 (stdin/stdout/stderr).
/// Prevents leaking sockets, pipes, or other FDs into the child shell.
fn close_inherited_fds() {
    // Try reading /dev/fd for accurate FD enumeration
    if let Ok(entries) = std::fs::read_dir("/dev/fd") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if let Some(name_str) = name.to_str() {
                if let Ok(fd) = name_str.parse::<RawFd>() {
                    if fd > 2 {
                        close(fd).ok();
                    }
                }
            }
        }
    } else {
        let max_fd = unsafe {
            let mut rl = std::mem::zeroed::<libc::rlimit>();
            if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rl) == 0 {
                rl.rlim_cur as RawFd
            } else {
                4096
            }
        };
        for fd in 3..max_fd {
            close(fd).ok();
        }
    }
}

/// Default terminal size for PTY
#[allow(dead_code)]
pub const DEFAULT_COLUMNS: u16 = 80;
#[allow(dead_code)]
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
    /// Current foreground color from SGR tracking
    current_fg: Option<Color>,
    /// Current background color from SGR tracking
    current_bg: Option<Color>,
    /// Current text attributes from SGR tracking
    current_attrs: Attributes,
    /// Saved grid state for alternate screen buffer
    saved_grid: Option<Grid>,
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
            current_fg: None,
            current_bg: None,
            current_attrs: Attributes::default(),
            saved_grid: None,
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
            
            // Close all inherited file descriptors to prevent leaking
            // sockets, pipes, or other FDs into the shell process.
            close_inherited_fds();
            
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
            foreground: self.current_fg,
            background: self.current_bg,
            attributes: self.current_attrs,
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
                if enable {
                    // Save current grid and clear
                    self.saved_grid = Some(std::mem::replace(
                        &mut self.grid,
                        Grid::new(self.columns, self.rows),
                    ));
                } else {
                    // Restore saved grid
                    if let Some(saved) = self.saved_grid.take() {
                        self.grid = saved;
                    }
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
    
    /// Handle scroll down (reverse scroll).
    /// Pops the oldest line from scrollback and inserts it at the top of the grid,
    /// shifting all grid content down.
    fn handle_scroll_down(&mut self, n: u16) {
        for _ in 0..n {
            if self.scrollback.len() > 0 {
                let line = self.scrollback.get_line(0).cloned();
                if let Some(scrollback_line) = line {
                    // Shift grid content down by one row
                    for row in (1..self.rows as usize).rev() {
                        for col in 0..self.columns as usize {
                            let src = self.grid.cells[row - 1][col].clone();
                            self.grid.cells[row][col] = src;
                        }
                    }
                    
                    // Write scrollback line into the top grid row
                    for col in 0..self.columns as usize {
                        self.grid.cells[0][col] = scrollback_line
                            .get(col)
                            .cloned()
                            .unwrap_or_default();
                    }
                    
                    // Remove the consumed line from scrollback front
                    self.scrollback.pop_front();
                }
            }
        }
    }
    
    /// Handle SGR (Select Graphic Rendition) attributes.
    /// Tracks current fg/bg color and text attributes, applying them
    /// to all subsequently placed characters.
    fn handle_sgr(&mut self, attrs: Vec<SgrAttribute>) {
        for attr in attrs {
            match attr {
                SgrAttribute::Reset => {
                    self.current_fg = None;
                    self.current_bg = None;
                    self.current_attrs = Attributes::default();
                }
                SgrAttribute::Bold => self.current_attrs.bold = true,
                SgrAttribute::Dim => self.current_attrs.dim = true,
                SgrAttribute::Italic => self.current_attrs.italic = true,
                SgrAttribute::Underline => self.current_attrs.underline = true,
                SgrAttribute::Blink => self.current_attrs.blink = true,
                SgrAttribute::Reverse => self.current_attrs.reverse = true,
                SgrAttribute::Hidden => self.current_attrs.hidden = true,
                SgrAttribute::CrossedOut => self.current_attrs.crossed_out = true,
                SgrAttribute::NoBold => self.current_attrs.bold = false,
                SgrAttribute::NoDim => self.current_attrs.dim = false,
                SgrAttribute::NoItalic => self.current_attrs.italic = false,
                SgrAttribute::NoUnderline => self.current_attrs.underline = false,
                SgrAttribute::NoBlink => self.current_attrs.blink = false,
                SgrAttribute::NoReverse => self.current_attrs.reverse = false,
                SgrAttribute::NoHidden => self.current_attrs.hidden = false,
                SgrAttribute::NoCrossedOut => self.current_attrs.crossed_out = false,
                SgrAttribute::ForegroundColor(color) => self.current_fg = Some(color),
                SgrAttribute::BackgroundColor(color) => self.current_bg = Some(color),
                SgrAttribute::DefaultForeground => self.current_fg = None,
                SgrAttribute::DefaultBackground => self.current_bg = None,
                SgrAttribute::BrightForeground => {
                    // BrightForeground typically handled by extended colors;
                    // keep current fg if set, otherwise fall through
                }
                SgrAttribute::Unknown(_) => {}
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
    pub current_fg: Option<Color>,
    pub current_bg: Option<Color>,
    pub current_attrs: Attributes,
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
            current_fg: None,
            current_bg: None,
            current_attrs: Attributes::default(),
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
            foreground: self.current_fg,
            background: self.current_bg,
            attributes: self.current_attrs,
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
                        SgrAttribute::Reset => {
                            self.current_fg = None;
                            self.current_bg = None;
                            self.current_attrs = Attributes::default();
                        }
                        SgrAttribute::Bold => self.current_attrs.bold = true,
                        SgrAttribute::Dim => self.current_attrs.dim = true,
                        SgrAttribute::Italic => self.current_attrs.italic = true,
                        SgrAttribute::Underline => self.current_attrs.underline = true,
                        SgrAttribute::Blink => self.current_attrs.blink = true,
                        SgrAttribute::Reverse => self.current_attrs.reverse = true,
                        SgrAttribute::Hidden => self.current_attrs.hidden = true,
                        SgrAttribute::CrossedOut => self.current_attrs.crossed_out = true,
                        SgrAttribute::NoBold => self.current_attrs.bold = false,
                        SgrAttribute::NoDim => self.current_attrs.dim = false,
                        SgrAttribute::NoItalic => self.current_attrs.italic = false,
                        SgrAttribute::NoUnderline => self.current_attrs.underline = false,
                        SgrAttribute::NoBlink => self.current_attrs.blink = false,
                        SgrAttribute::NoReverse => self.current_attrs.reverse = false,
                        SgrAttribute::NoHidden => self.current_attrs.hidden = false,
                        SgrAttribute::NoCrossedOut => self.current_attrs.crossed_out = false,
                        SgrAttribute::ForegroundColor(color) => self.current_fg = Some(color),
                        SgrAttribute::BackgroundColor(color) => self.current_bg = Some(color),
                        SgrAttribute::DefaultForeground => self.current_fg = None,
                        SgrAttribute::DefaultBackground => self.current_bg = None,
                        _ => {}
                    }
                }
            }
            AnsiSequence::CursorVisibility(visible) => {
                self.cursor.visible = visible;
            }
            AnsiSequence::EraseInLine(mode) => {
                match mode {
                    EraseLineMode::FromCursorToEnd => {
                        self.grid.clear_to_end_of_line(self.cursor.column, self.cursor.row);
                    }
                    EraseLineMode::FromCursorToStart => {
                        self.grid.clear_to_start_of_line(self.cursor.column, self.cursor.row);
                    }
                    EraseLineMode::All => {
                        self.grid.clear_line(self.cursor.row);
                    }
                }
            }
            AnsiSequence::EraseInDisplay(mode) => {
                match mode {
                    EraseDisplayMode::FromCursorToEnd => {
                        for r in self.cursor.row..self.rows {
                            self.grid.clear_line(r);
                        }
                    }
                    EraseDisplayMode::FromCursorToStart => {
                        for r in 0..=self.cursor.row {
                            self.grid.clear_line(r);
                        }
                    }
                    EraseDisplayMode::All => {
                        self.grid.clear();
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

    // ── Command lifecycle integration tests ──

    /// Simulates a shell session: prompt → command echo → output → next prompt
    #[test]
    fn test_command_lifecycle_simple() -> Result<()> {
        let mut term = MockTerminal::new(80, 10);

        // Stage 1: Shell prompt appears (e.g. "$ " in green)
        term.process_data(b"\x1b[32m$ \x1b[0m")?;
        assert_eq!(term.grid.get(0, 0).unwrap().character, '$');
        assert!(term.grid.get(0, 0).unwrap().attributes.dim == false);
        assert_eq!(term.grid.get(0, 0).unwrap().foreground, Some(Color::Index(2)));

        // Stage 2: User types "ls -la" (echoed back by terminal)
        // Prompt "$ " occupies columns 0-1, so "ls -la" starts at column 2.
        // String "ls -la" = 'l','s',' ','-','l','a' (6 chars starting at col 2)
        term.process_data(b"ls -la")?;
        assert_eq!(term.grid.get(0, 0).unwrap().character, '$');
        assert_eq!(term.grid.get(2, 0).unwrap().character, 'l');
        assert_eq!(term.grid.get(3, 0).unwrap().character, 's');
        assert_eq!(term.grid.get(4, 0).unwrap().character, ' ');
        assert_eq!(term.grid.get(5, 0).unwrap().character, '-');
        assert_eq!(term.grid.get(6, 0).unwrap().character, 'l');
        assert_eq!(term.grid.get(7, 0).unwrap().character, 'a');

        // Stage 3: User presses Enter, output appears
        term.process_data(b"\r\n")?;
        assert_eq!(term.cursor.column, 0);
        assert_eq!(term.cursor.row, 1);

        term.process_data(b"total 42\r\n")?;
        term.process_data(b"drwxr-xr-x  2 user staff  64 May 10 12:00 .\r\n")?;
        term.process_data(b"-rw-r--r--  1 user staff 128 May 10 12:00 file.txt\r\n")?;

        assert_eq!(term.grid.get(0, 1).unwrap().character, 't');
        assert_eq!(term.grid.get(0, 2).unwrap().character, 'd');
        assert_eq!(term.grid.get(0, 3).unwrap().character, '-');

        // Stage 4: New prompt appears
        term.process_data(b"\x1b[32m$ \x1b[0m")?;
        assert_eq!(term.grid.get(0, 4).unwrap().character, '$');
        assert_eq!(term.grid.get(0, 4).unwrap().foreground, Some(Color::Index(2)));

        Ok(())
    }

    /// Tests command output that triggers scrollback overflow via line wrapping
    #[test]
    fn test_command_lifecycle_scrollback() -> Result<()> {
        let mut term = MockTerminal::new(10, 3);

        // MockTerminal creates scrollback only when characters wrap past the
        // column limit AND the row overflows. Write enough lines to wrap and
        // overflow the small terminal, verifying scrollback accumulates.
        for _ in 0..5 {
            // 20 chars on a 10-col terminal causes wrap + eventual scrollback
            term.process_data(b"1234567890ABCDEFGHIJ\r\n")?;
        }

        // After 5 long lines on a 3-row terminal, scrollback should have content
        assert!(!term.scrollback.is_empty(), "Scrollback should have content after overflow");

        // The grid state after scrolling should still be coherent.
        // Row 0 should have content from after scroll occurred.
        let cell = term.grid.get(0, 0);
        assert!(cell.is_some());
        // Cursor should be on a valid row within the terminal
        assert!(term.cursor.row < term.rows);

        Ok(())
    }

    /// Tests that SGR attributes persist correctly across a command lifecycle
    #[test]
    fn test_command_lifecycle_sgr_persistence() -> Result<()> {
        let mut term = MockTerminal::new(40, 8);

        // Prompt with bold green
        term.process_data(b"\x1b[1;32muser@host:~$ \x1b[0m")?;
        assert_eq!(term.grid.get(0, 0).unwrap().character, 'u');
        assert!(term.grid.get(0, 0).unwrap().attributes.bold);
        assert_eq!(term.grid.get(0, 0).unwrap().foreground, Some(Color::Index(2)));

        // After reset, new text should have no attributes
        term.process_data(b"echo test\r\n")?;
        term.process_data(b"test\r\n")?;
        assert!(!term.grid.get(0, 2).unwrap().attributes.bold);
        assert_eq!(term.grid.get(0, 2).unwrap().foreground, None);

        Ok(())
    }

    /// Tests cursor visibility toggling during command lifecycle
    #[test]
    fn test_command_lifecycle_cursor_visibility() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);

        // Initially visible
        assert!(term.cursor.visible);

        // Hide cursor (common during TUI programs)
        term.process_data(b"\x1b[?25l")?;
        assert!(!term.cursor.visible);

        // Show cursor again
        term.process_data(b"\x1b[?25h")?;
        assert!(term.cursor.visible);

        Ok(())
    }

    /// Tests that output after cursor position changes is correct
    #[test]
    fn test_command_lifecycle_cursor_repositioning() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);

        // Write some text, then move cursor and overwrite
        term.process_data(b"Hello World")?;
        assert_eq!(term.cursor.column, 11);
        assert_eq!(term.cursor.row, 0);

        // Move cursor to beginning of line
        term.process_data(b"\r")?;
        assert_eq!(term.cursor.column, 0);

        // Overwrite with different text
        term.process_data(b"Hi")?;
        assert_eq!(term.grid.get(0, 0).unwrap().character, 'H');
        assert_eq!(term.grid.get(1, 0).unwrap().character, 'i');
        // The rest of the original text should remain
        assert_eq!(term.grid.get(2, 0).unwrap().character, 'l');
        assert_eq!(term.grid.get(3, 0).unwrap().character, 'l');

        Ok(())
    }

    /// Simulates a full multi-line command output within a single grid frame
    #[test]
    fn test_command_lifecycle_multi_line_output() -> Result<()> {
        let mut term = MockTerminal::new(30, 10);

        // Clear and show prompt
        term.process_data(b"\x1b[2J\x1b[H\x1b[32m$ \x1b[0m")?;
        assert_eq!(term.cursor.row, 0);
        assert_eq!(term.cursor.column, 2);

        term.process_data(b"cat file.txt\r\n")?;

        // Simulate multi-line file output
        let lines = [
            "line 1: alpha",
            "line 2: beta",
            "line 3: gamma",
            "line 4: delta",
            "",
        ];
        for line in &lines {
            term.process_data(format!("{}\r\n", line).as_bytes())?;
        }

        // Verify each line was placed correctly
        assert_eq!(term.grid.get(0, 1).unwrap().character, 'l');
        assert_eq!(term.grid.get(0, 2).unwrap().character, 'l');
        assert_eq!(term.grid.get(0, 3).unwrap().character, 'l');
        assert_eq!(term.grid.get(0, 4).unwrap().character, 'l');
        // Line 5 should be empty (the "" line)
        assert_eq!(term.grid.get(0, 5).unwrap().character, ' ');

        Ok(())
    }

    /// Tests that erase sequences mid-command behave correctly
    #[test]
    fn test_command_lifecycle_erase_sequences() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);

        // Write a line, then clear it
        term.process_data(b"old content\x1b[K")?;
        assert_eq!(term.grid.get(0, 0).unwrap().character, 'o');
        // Cells after cursor should be cleared (cursor at column 11, clear to EOL)
        assert_eq!(term.grid.get(11, 0).unwrap().character, ' ');
        assert_eq!(term.grid.get(15, 0).unwrap().character, ' ');

        // Write new content on the cleared line
        term.process_data(b"\rnew")?;
        assert_eq!(term.grid.get(0, 0).unwrap().character, 'n');
        assert_eq!(term.grid.get(1, 0).unwrap().character, 'e');

        Ok(())
    }

    /// Tests that escape sequences within output text don't corrupt subsequent output
    #[test]
    fn test_command_lifecycle_ansi_in_output() -> Result<()> {
        let mut term = MockTerminal::new(40, 8);

        term.process_data(b"\x1b[32m$ \x1b[0m")?;
        term.process_data(b"echo 'hello'\r\n")?;

        // Output with colorized text
        term.process_data(b"\x1b[31mhello\x1b[0m\r\n")?;

        // The red 'hello' should be on the line after command echo
        assert_eq!(term.grid.get(0, 1).unwrap().character, 'h');
        assert_eq!(term.grid.get(0, 1).unwrap().foreground, Some(Color::Index(1)));

        // Subsequent text should be default color
        term.process_data(b"\x1b[32m$ \x1b[0m")?;
        assert_eq!(term.grid.get(0, 2).unwrap().character, '$');
        assert_eq!(term.grid.get(0, 2).unwrap().foreground, Some(Color::Index(2)));

        Ok(())
    }

    /// Tests that rapid successive prompts work correctly
    #[test]
    fn test_command_lifecycle_rapid_prompts() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);

        // Multiple rapid commands in sequence
        for _ in 0..5 {
            term.process_data(b"\x1b[32m$ \x1b[0m")?;
            term.process_data(b"cmd\r\n")?;
        }

        // After 5 iterations on a 5-row terminal, the last \n triggers
        // scroll_up(), pushing row 0 off and adding a blank row at the end.
        // Each row has "$ cmd" (prompt + cmd), so check "cmd" at column 2.
        assert_eq!(term.grid.get(0, 0).unwrap().character, '$');
        assert_eq!(term.grid.get(2, 0).unwrap().character, 'c');
        assert_eq!(term.grid.get(2, 1).unwrap().character, 'c');
        assert_eq!(term.grid.get(2, 2).unwrap().character, 'c');
        assert_eq!(term.grid.get(2, 3).unwrap().character, 'c');
        assert!(term.grid.get(0, 4).unwrap().character.is_whitespace());

        Ok(())
    }

    /// Tests that clearing screen mid-session works correctly
    #[test]
    fn test_command_lifecycle_clear_screen() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);

        term.process_data(b"line 1\r\nline 2\r\nline 3")?;
        assert_eq!(term.grid.get(0, 2).unwrap().character, 'l');

        // Clear screen
        term.process_data(b"\x1b[2J\x1b[H")?;

        // All cells should be empty
        assert!(term.grid.get(0, 0).unwrap().character.is_whitespace());
        assert!(term.grid.get(0, 1).unwrap().character.is_whitespace());
        assert!(term.grid.get(0, 2).unwrap().character.is_whitespace());

        // Cursor should be at home position
        assert_eq!(term.cursor.row, 0);
        assert_eq!(term.cursor.column, 0);

        // New content after clear should work
        term.process_data(b"fresh start")?;
        assert_eq!(term.grid.get(0, 0).unwrap().character, 'f');

        Ok(())
    }

    /// Edge case: output exactly filling the terminal width
    #[test]
    fn test_command_lifecycle_exact_width_line() -> Result<()> {
        let mut term = MockTerminal::new(10, 3);

        // Write exactly 10 chars (fills the line)
        term.process_data(b"1234567890")?;
        assert_eq!(term.cursor.column, 0);
        assert_eq!(term.cursor.row, 1);

        // The last char should wrap to the beginning of the next row
        assert_eq!(term.grid.get(9, 0).unwrap().character, '0');

        Ok(())
    }

    // ── SGR-specific unit tests ──

    /// Tests that bold attribute is correctly set and reset
    #[test]
    fn test_sgr_bold() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);
        term.process_data(b"\x1b[1mBold\x1b[0mNormal")?;
        assert!(term.grid.get(0, 0).unwrap().attributes.bold);
        assert!(!term.grid.get(4, 0).unwrap().attributes.bold);
        Ok(())
    }

    /// Tests that italic attribute is correctly set and reset
    #[test]
    fn test_sgr_italic() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);
        term.process_data(b"\x1b[3mItalic\x1b[23mNoItalic")?;
        assert!(term.grid.get(0, 0).unwrap().attributes.italic);
        assert!(!term.grid.get(6, 0).unwrap().attributes.italic);
        Ok(())
    }

    /// Tests that underline attribute is correctly set and reset
    #[test]
    fn test_sgr_underline() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);
        term.process_data(b"\x1b[4mUnderline\x1b[24mNoUnder")?;
        assert!(term.grid.get(0, 0).unwrap().attributes.underline);
        assert!(!term.grid.get(9, 0).unwrap().attributes.underline);
        Ok(())
    }

    /// Tests that blink attribute is correctly set and reset
    #[test]
    fn test_sgr_blink() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);
        term.process_data(b"\x1b[5mBlink\x1b[25mNoBlink")?;
        assert!(term.grid.get(0, 0).unwrap().attributes.blink);
        assert!(!term.grid.get(5, 0).unwrap().attributes.blink);
        Ok(())
    }

    /// Tests that reverse attribute is correctly set and reset
    #[test]
    fn test_sgr_reverse() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);
        term.process_data(b"\x1b[7mReverse\x1b[27mNoRev")?;
        assert!(term.grid.get(0, 0).unwrap().attributes.reverse);
        assert!(!term.grid.get(7, 0).unwrap().attributes.reverse);
        Ok(())
    }

    /// Tests that hidden attribute is correctly set and reset
    #[test]
    fn test_sgr_hidden() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);
        term.process_data(b"\x1b[8mHidden\x1b[28mVisible")?;
        assert!(term.grid.get(0, 0).unwrap().attributes.hidden);
        assert!(!term.grid.get(6, 0).unwrap().attributes.hidden);
        Ok(())
    }

    /// Tests that crossed-out attribute is correctly set and reset
    #[test]
    fn test_sgr_crossed_out() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);
        term.process_data(b"\x1b[9mCrossed\x1b[29mNormal")?;
        assert!(term.grid.get(0, 0).unwrap().attributes.crossed_out);
        assert!(!term.grid.get(7, 0).unwrap().attributes.crossed_out);
        Ok(())
    }

    /// Tests that reset clears all attributes and colors
    #[test]
    fn test_sgr_reset_clears_all() -> Result<()> {
        let mut term = MockTerminal::new(30, 5);
        term.process_data(b"\x1b[1;3;4;31;42mStyled\x1b[0mPlain")?;
        // Before reset
        let styled = term.grid.get(0, 0).unwrap();
        assert!(styled.attributes.bold);
        assert!(styled.attributes.italic);
        assert!(styled.attributes.underline);
        assert_eq!(styled.foreground, Some(Color::Index(1)));
        assert_eq!(styled.background, Some(Color::Index(2)));
        // After reset - all should be default
        let plain = term.grid.get(6, 0).unwrap();
        assert!(!plain.attributes.bold);
        assert!(!plain.attributes.italic);
        assert!(!plain.attributes.underline);
        assert_eq!(plain.foreground, None);
        assert_eq!(plain.background, None);
        Ok(())
    }

    /// Tests that foreground and background colors work independently
    #[test]
    fn test_sgr_foreground_background_independence() -> Result<()> {
        let mut term = MockTerminal::new(30, 5);
        // Only foreground on row 0
        term.process_data(b"\x1b[31mRedFG\x1b[0m\r\n")?;
        assert_eq!(term.grid.get(0, 0).unwrap().foreground, Some(Color::Index(1)));
        assert_eq!(term.grid.get(0, 0).unwrap().background, None);

        // Only background on row 1
        term.process_data(b"\x1b[42mGreenBG\x1b[0m\r\n")?;
        assert_eq!(term.grid.get(0, 1).unwrap().foreground, None);
        assert_eq!(term.grid.get(0, 1).unwrap().background, Some(Color::Index(2)));

        // Both on row 2
        term.process_data(b"\x1b[31;42mBoth\x1b[0m\r\n")?;
        let both = term.grid.get(0, 2).unwrap();
        assert_eq!(both.foreground, Some(Color::Index(1)));
        assert_eq!(both.background, Some(Color::Index(2)));
        Ok(())
    }

    /// Tests that dim attribute works
    #[test]
    fn test_sgr_dim() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);
        term.process_data(b"\x1b[2mDim\x1b[22mNoDim")?;
        assert!(term.grid.get(0, 0).unwrap().attributes.dim);
        assert!(!term.grid.get(3, 0).unwrap().attributes.dim);
        Ok(())
    }

    /// Tests default foreground/background restore
    #[test]
    fn test_sgr_default_colors() -> Result<()> {
        let mut term = MockTerminal::new(20, 5);
        // Set fg/bg, then restore to default
        term.process_data(b"\x1b[31;42mStyled\x1b[39;49mDefault")?;
        let styled = term.grid.get(0, 0).unwrap();
        assert_eq!(styled.foreground, Some(Color::Index(1)));
        assert_eq!(styled.background, Some(Color::Index(2)));
        let defaulted = term.grid.get(6, 0).unwrap();
        assert_eq!(defaulted.foreground, None);
        assert_eq!(defaulted.background, None);
        Ok(())
    }

    /// Tests multiple SGR codes in a single sequence
    #[test]
    fn test_sgr_multi_code_sequence() -> Result<()> {
        let mut term = MockTerminal::new(30, 5);
        // Bold + Italic + Underline + Red fg in one sequence
        term.process_data(b"\x1b[1;3;4;31mMulti")?;
        let cell = term.grid.get(0, 0).unwrap();
        assert!(cell.attributes.bold);
        assert!(cell.attributes.italic);
        assert!(cell.attributes.underline);
        assert_eq!(cell.foreground, Some(Color::Index(1)));
        Ok(())
    }
}
