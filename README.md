# TerRust

A next-generation terminal environment built in Rust, combining traditional terminal workflows with AI assistance and visual enhancements.

## Overview

TerRust is a modern terminal emulator that brings a block-based UI to your command-line workflows. It combines the familiar experience of a traditional terminal with AI-powered assistance and structured output visualization.

## Features

### Core Terminal
- **PTY-based shell** - Native terminal behavior using POSIX pseudo-terminals
- **ANSI escape sequence parsing** - Full support for cursor movement, colors (16/256/truecolor), text attributes, and terminal modes
- **Scrollback buffer** - Configurable history with efficient memory management
- **Mouse support** - Optional mouse event handling

### AI Integration
- **Multiple AI providers** - Support for Claude, OpenAI, and custom providers
- **AI chat mode** - Press `/` to enter AI prompt mode
- **Command assistance** - AI can help generate and explain commands
- **Context-aware responses** - AI has access to recent command history

### Block-Based UI
- **Structured output** - Commands and output organized as collapsible blocks
- **Multiple block types**: Command, Output, Error, AI, System
- **Block metadata** - Track exit codes, duration, working directory, shell
- **Block management** - Pin, collapse, search, and navigate between blocks

### Configuration
- **Themes** - Tokyo Night (default), Catppuccin Mocha, Dracula
- **Keybindings** - Customizable key bindings with 50+ actions
- **TOML-based config** - Human-readable configuration files
- **Platform detection** - Automatic config directory resolution

### Additional Features
- **Command history** - Persistent history with search
- **Plugin system** - Dynamic plugin loading (scaffold)
- **Clipboard integration** - Copy/paste support
- **Rich text rendering** - Syntax highlighting via syntect

## Installation

### Prerequisites
- Rust 1.70+ (for stable async/await and async traits)
- Cargo

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Annamalai23/TerRust.git
cd TerRust

# Build in release mode
cargo build --release

# Run
./target/release/terrust
```

### Development Build

```bash
# Run in debug mode with verbose logging
cargo run -- --verbose
```

## Usage

### Command-Line Options

```
terrust [OPTIONS]

Options:
  -c, --config <FILE>      Path to configuration file
  --no-ai                  Disable AI features
  -s, --shell <SHELL>      Shell to use (default: from config or $SHELL)
  --fullscreen             Start in fullscreen mode
  -v, --verbose            Enable verbose logging
  --list-plugins           List available plugins
  --plugin-dir <DIR>       Plugin directory
  -h, --help               Print help
  -V, --version            Print version
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Execute command |
| `Ctrl+C` | Force quit |
| `Ctrl+Q` | Quit |
| `/` | Enter AI mode |
| `Esc` | Exit AI mode / Cancel input |
| `PageUp` | Scroll up through history |
| `PageDown` | Scroll down |
| `↑/↓` | Navigate command history |

### Configuration File

Create `~/.config/terrust/config.toml`:

```toml
[general]
theme = "tokyo_night"
scrollback_limit = 10000

[terminal]
shell = "/bin/zsh"
cursor_style = "block"

[ai]
enabled = true
provider = "claude"
```

## Architecture

### Module Structure

```
src/
├── main.rs              # Entry point, CLI parsing
├── app.rs               # Application state & event loop
├── config/              # Configuration system
│   ├── mod.rs           # Config structures
│   ├── theme.rs         # Theme system
│   └── keybindings.rs   # Keybinding system
├── terminal/            # Terminal core
│   ├── mod.rs           # Module exports
│   ├── pty.rs           # PTY management
│   ├── ansi.rs          # ANSI parser
│   ├── buffer.rs        # Grid & scrollback
│   ├── cell.rs          # Cell representation
│   └── color.rs         # Color system
├── ui/                  # User interface
│   ├── mod.rs           # UI exports
│   ├── blocks.rs        # Block system
│   ├── input.rs        # Input bar
│   ├── render.rs        # Rendering helpers
│   └── components.rs    # UI components
├── ai/                  # AI integration
├── history/             # Command history
├── plugins/             # Plugin system
└── utils/               # Utilities
```

### Key Components

**Terminal** (`src/terminal/pty.rs`)
- PTY master/slave creation using `libc::openpty`
- Shell process spawning via `fork()`/`execvp()`
- Non-blocking I/O with `select()`
- ANSI sequence handling with state machine parser

**BlockManager** (`src/ui/blocks.rs`)
- UUID-based block identification
- Content stored as 2D cell grid
- Pin/collapse/search operations
- Max blocks limit with auto-eviction

**App** (`src/app.rs`)
- Async event loop using crossterm
- PTY output polling and processing
- Block-based command lifecycle tracking
- ratatui-based rendering

## Building

### Feature Flags

```toml
[features]
default = ["all"]
all = ["ai", "plugins", "clipboard"]
ai = ["reqwest", "tokio-stream", "serde_json"]
plugins = ["libloading"]
clipboard = ["arboard"]
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --lib terminal
cargo test --lib ui
```

### Benchmarks

```bash
cargo bench
```

## Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Terminal I/O | crossterm | Cross-platform, active maintenance |
| UI Framework | ratatui | Modern successor to tui-rs |
| Async Runtime | tokio | Industry standard |
| Syntax Highlighting | syntect | Mature, feature-rich |
| Serialization | serde | Standard Rust ecosystem |
| Configuration | TOML | Human-readable |

## Project Status

| Area | Status |
|------|--------|
| Terminal core | ✅ Complete |
| PTY integration | ✅ Complete |
| Block-based UI | ✅ Complete |
| AI integration | ✅ Framework ready |
| Plugin system | ⚠️ Scaffold |
| Testing | ✅ 102 tests passing |

## Contributing

Contributions welcome! Please ensure:
- `cargo check` passes
- `cargo test` passes
- No new warnings introduced

## License

MIT OR Apache-2.0