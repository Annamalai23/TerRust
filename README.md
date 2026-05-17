# TerRust

A next-generation terminal environment built in Rust, combining traditional terminal workflows with AI assistance, block-based UI, and scrollback interaction.

## Features

### Core Terminal
- **PTY-based shell** — Native shell execution using POSIX pseudo-terminals via `libc::openpty` / `fork()` / `execvp()`
- **ANSI escape sequence parser** — Full state-machine parser for cursor movement, SGR attributes (bold, italic, underline, blink, 16/256/truecolor), OSC sequences, alternate screen buffer, scroll regions, and terminal modes
- **Scrollback buffer** — Configurable line limit with O(1) amortized insertion, bidirectional navigation
- **Scrollback interaction** — Keyboard (PgUp/PgDn/Shift+↑↓) and mouse wheel scrolling, scrollbar indicator, text selection with Clipboard copy

### Block-Based UI
- **Structured output** — Commands and output organized as discrete, collapsible blocks
- **Block types** — Command, Output, Error, AI, System with distinct styling per theme
- **Block metadata** — Tracks exit code, duration, working directory, shell, and timestamp per block
- **Block management** — Pin (preserve across eviction), collapse, search across all blocks, auto-eviction with pinned-block protection
- **Search** — Incremental case-insensitive search across all blocks with match highlighting (active match in yellow, others in gray)

### AI Integration
- **Multiple providers** — Claude CLI, Local/Ollama, OpenAI (extensible via `AIProvider` trait)
- **AI chat mode** — Press `/` to enter AI prompt mode, response displayed as `BlockType::AI` block
- **Streaming display** — Provider responses split into word-group chunks rendered incrementally through event channel
- **Command assistance** — AI suggestions for partial input

### Configuration
- **Themes** — Tokyo Night (default), Catppuccin Mocha, Dracula with full ANSI palette and block-specific colors
- **Keybindings** — 50+ predefined actions with customizable bindings, display formatting with Unicode symbols
- **TOML-based** — Human-readable config files at platform-specific paths
- **Theme CLI** — `terrust theme list`, `theme preview <name>`, `theme show <name>`, `theme edit <name> <key>=<value>`

### Plugin System
- **Dynamic loading** — Plugin discovery from TOML metadata in plugin directories
- **git_helper plugin** — Git-aware prompt (branch name, working tree status)

## Installation

### Prerequisites
- Rust 1.70+
- Cargo

### Build from Source

```bash
git clone https://github.com/Annamalai23/TerRust.git
cd TerRust
cargo build --release
./target/release/terrust
```

### Development

```bash
cargo run -- --verbose

# With specific shell
cargo run -- --shell /bin/zsh

# Theme management
cargo run -- theme list
cargo run -- theme preview tokyo-night
```

## Usage

### Command-Line Options

```
terrust [OPTIONS] [COMMAND]

Commands:
  theme   Theme management (list, preview, show, edit)

Options:
  -c, --config <FILE>       Path to configuration file
  --no-ai                   Disable AI features
  -s, --shell <SHELL>       Shell to use
  --fullscreen              Start in fullscreen mode
  -v, --verbose             Enable verbose logging
  --list-plugins            List available plugins
  --plugin-dir <DIR>        Plugin directory
  -h, --help                Print help
  -V, --version             Print version
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Execute command |
| `Ctrl+C` | Force quit |
| `Ctrl+Q` | Quit |
| `Esc` | Cancel / exit mode |
| `↑` / `↓` | Command history |
| `/` | Enter AI mode |
| `PageUp` | Scroll up one page |
| `PageDown` | Scroll down one page |
| `Shift+↑` | Scroll up one line |
| `Shift+↓` | Scroll down one line |
| `Ctrl+F` | Enter search mode |
| `Enter` (search) | Next match |
| `Shift+Enter` (search) | Previous match |
| `Esc` (search) | Exit search |
| `Ctrl+Shift+C` | Copy selection to clipboard |
| `Mouse Wheel` | Scroll up/down (3 lines per tick) |

### Theme CLI

```bash
# List available themes
terrust theme list

# Preview a theme (renders ANSI color swatch)
terrust theme preview tokyo-night

# Show full theme config as TOML
terrust theme show dracula

# Edit a theme value and save to user themes directory
terrust theme edit my-theme general.background "#1a1b26"
```

### Configuration File

```toml
[general]
theme = "tokyo_night"
scrollback_limit = 10000
opacity = 1.0

[terminal]
shell = "/bin/zsh"
cursor_style = "block"
cursor_blink = true
mouse_support = true
true_color = true

[ai]
enabled = true
default_provider = "claude"
timeout = 30

[ai.providers.claude]
api_key = "claude"
endpoint = ""
model = ""
```

## Architecture

### Module Structure

```
src/
├── main.rs             # Entry point, CLI argument parsing (7 tests)
├── lib.rs              # Library crate (public API exports)
├── cli.rs              # Theme subcommand (list, preview, show, edit)
├── app.rs              # Application state & event loop
├── config/
│   ├── mod.rs          # Config hierarchy (6 tests)
│   ├── theme.rs        # Theme system with 3 presets (4 tests)
│   └── keybindings.rs  # Keybinding system (6 tests)
├── terminal/
│   ├── mod.rs          # Module exports
│   ├── pty.rs          # PTY + grid + ANSI parser + MockTerminal (36 tests)
│   ├── ansi.rs         # State-machine ANSI parser (35 tests + 10 proptest)
│   ├── buffer.rs       # Grid & ScrollbackBuffer
│   ├── cell.rs         # Cell & Attributes
│   ├── color.rs        # Color (Named, Index, RGB)
│   └── config.rs       # ShellConfig
├── ui/
│   ├── mod.rs          # UI exports
│   ├── blocks.rs       # Block system (12 tests)
│   ├── input.rs        # Input bar (10 tests)
│   ├── components.rs   # Reusable components (14 tests)
│   ├── render.rs       # Rendering helpers (3 tests)
│   └── search.rs       # Search engine (11 tests)
├── ai/
│   ├── mod.rs          # AIClient, CompletionProvider, streaming (8 tests)
│   └── provider.rs     # Provider trait, Claude/Local/OpenAI (7 tests)
├── history/mod.rs      # Command history (3 tests)
├── plugins/mod.rs      # Plugin manager (4 tests)
└── utils.rs            # Platform detection, string/path/validation (22 tests)
```

### Testing

| Suite | Tests |
|-------|-------|
| Unit tests (lib) | 173 |
| Unit tests (main) | 7 |
| Integration: AI | 8 |
| Integration: Config | 11 |
| Integration: E2E | 4 |
| Integration: Plugin | 8 |
| Integration: Scrollback | 9 |
| Integration: Terminal | 15 |
| Integration: UI | 25 |
| **Total** | **260** |
| Property-based (ANSI parser) | 10 |
| Benchmarks | 18 |

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Run with specific features
cargo test --features "ai,clipboard"
```

## Project Status

| Area | Status |
|------|--------|
| Terminal core (ANSI, buffer, grid) | ✅ Complete |
| PTY integration | ✅ Complete |
| SGR attribute tracking | ✅ Complete |
| Alternate screen buffer | ✅ Complete |
| Config validation | ✅ Complete |
| Panic safety (terminal restore) | ✅ Complete |
| Block-based UI | ✅ Complete |
| Command lifecycle tracking | ✅ Complete |
| Scrollback scrolling (keyboard + mouse) | ✅ Complete |
| Scrollback search (incremental) | ✅ Complete |
| Selection + Clipboard copy | ✅ Complete |
| Scrollbar indicator | ✅ Complete |
| AI integration (providers, streaming) | ✅ Complete |
| Theme CLI (list, preview, show, edit) | ✅ Complete |
| Plugin system + git_helper | ✅ Complete |
| Benchmarks (ANSI parser, grid) | ✅ Complete |
| Property-based tests (ANSI parser) | ✅ Complete |
| E2E integration tests | ✅ Complete |
| **Overall** | **~100%** |

## Technology Stack

| Component | Library |
|-----------|---------|
| Terminal I/O | crossterm |
| UI Framework | ratatui |
| Async Runtime | tokio |
| Syntax Highlighting | syntect |
| Configuration | TOML via serde |
| Plugin System | libloading |
| Error Handling | anyhow + thiserror |
| Logging | tracing |
| Clipboard | arboard |
| PTY | libc + nix |
| UUID | uuid |

## Contributing

```bash
cargo check      # Must pass with 0 warnings
cargo test       # Must pass all 260+ tests
cargo bench      # Verify benchmarks compile
```

## License

MIT OR Apache-2.0
