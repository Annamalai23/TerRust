//! Terminal emulator core functionality
//!
//! Provides PTY-based shell process management, terminal grid/buffer,
//! and ANSI escape sequence parsing for TerRust.

mod ansi;
mod buffer;
mod cell;
mod color;
mod config;
mod pty;

pub use ansi::{AnsiParser, AnsiParseResult, AnsiSequence, AnsiParseState};
pub use buffer::{Grid, ScrollbackBuffer};
pub use cell::{Cell, Attributes};
pub use color::Color;
pub use config::ShellConfig;
pub use pty::Terminal;

pub use ansi::{CursorStyle, Cursor, EraseDisplayMode, EraseLineMode, OscSequence, SgrAttribute, TerminalMode};
pub use color::ColorName;
