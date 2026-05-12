//! ANSI escape sequence parser and types

use super::color::Color;

/// Current cursor style
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorStyle {
    Block,
    Beam,
    Underline,
}

impl Default for CursorStyle {
    fn default() -> Self {
        CursorStyle::Block
    }
}

/// Cursor state within the terminal
#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    /// Column position (0-indexed)
    pub column: u16,
    /// Row position (0-indexed)
    pub row: u16,
    /// Visibility flag
    pub visible: bool,
    /// Cursor style
    pub style: CursorStyle,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            column: 0,
            row: 0,
            visible: true,
            style: CursorStyle::default(),
        }
    }
}

/// Erase display mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EraseDisplayMode {
    FromCursorToEnd,
    FromCursorToStart,
    All,
}

/// Erase line mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EraseLineMode {
    FromCursorToEnd,
    FromCursorToStart,
    All,
}

/// Terminal mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalMode {
    IrM,
    Insert,
    SendReceive,
    AutoWrap,
    Origin,
    CursorKeys,
    Keypad,
    Echo,
    LineWrap,
    Mouse,
}

/// SGR (Select Graphic Rendition) attributes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SgrAttribute {
    Reset,
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Hidden,
    CrossedOut,
    NoBold,
    NoDim,
    NoItalic,
    NoUnderline,
    NoBlink,
    NoReverse,
    NoHidden,
    NoCrossedOut,
    ForegroundColor(Color),
    BackgroundColor(Color),
    DefaultForeground,
    DefaultBackground,
    BrightForeground,
    Unknown(u16),
}

impl SgrAttribute {
    pub fn from_code(code: u16) -> Vec<Self> {
        match code {
            0 => vec![Self::Reset],
            1 => vec![Self::Bold],
            2 => vec![Self::Dim],
            3 => vec![Self::Italic],
            4 => vec![Self::Underline],
            5 => vec![Self::Blink],
            7 => vec![Self::Reverse],
            8 => vec![Self::Hidden],
            9 => vec![Self::CrossedOut],
            21 | 22 => vec![Self::NoBold],
            22 => vec![Self::NoDim],
            23 => vec![Self::NoItalic],
            24 => vec![Self::NoUnderline],
            25 => vec![Self::NoBlink],
            27 => vec![Self::NoReverse],
            28 => vec![Self::NoHidden],
            29 => vec![Self::NoCrossedOut],
            30 => vec![Self::ForegroundColor(Color::Index(0))],
            31 => vec![Self::ForegroundColor(Color::Index(1))],
            32 => vec![Self::ForegroundColor(Color::Index(2))],
            33 => vec![Self::ForegroundColor(Color::Index(3))],
            34 => vec![Self::ForegroundColor(Color::Index(4))],
            35 => vec![Self::ForegroundColor(Color::Index(5))],
            36 => vec![Self::ForegroundColor(Color::Index(6))],
            37 => vec![Self::ForegroundColor(Color::Index(7))],
            38 => vec![],
            39 => vec![Self::DefaultForeground],
            40 => vec![Self::BackgroundColor(Color::Index(0))],
            41 => vec![Self::BackgroundColor(Color::Index(1))],
            42 => vec![Self::BackgroundColor(Color::Index(2))],
            43 => vec![Self::BackgroundColor(Color::Index(3))],
            44 => vec![Self::BackgroundColor(Color::Index(4))],
            45 => vec![Self::BackgroundColor(Color::Index(5))],
            46 => vec![Self::BackgroundColor(Color::Index(6))],
            47 => vec![Self::BackgroundColor(Color::Index(7))],
            48 => vec![],
            49 => vec![Self::DefaultBackground],
            90 => vec![Self::ForegroundColor(Color::Index(8))],
            91 => vec![Self::ForegroundColor(Color::Index(9))],
            92 => vec![Self::ForegroundColor(Color::Index(10))],
            93 => vec![Self::ForegroundColor(Color::Index(11))],
            94 => vec![Self::ForegroundColor(Color::Index(12))],
            95 => vec![Self::ForegroundColor(Color::Index(13))],
            96 => vec![Self::ForegroundColor(Color::Index(14))],
            97 => vec![Self::ForegroundColor(Color::Index(15))],
            100 => vec![Self::BackgroundColor(Color::Index(8))],
            101 => vec![Self::BackgroundColor(Color::Index(9))],
            102 => vec![Self::BackgroundColor(Color::Index(10))],
            103 => vec![Self::BackgroundColor(Color::Index(11))],
            104 => vec![Self::BackgroundColor(Color::Index(12))],
            105 => vec![Self::BackgroundColor(Color::Index(13))],
            106 => vec![Self::BackgroundColor(Color::Index(14))],
            107 => vec![Self::BackgroundColor(Color::Index(15))],
            _ => vec![Self::Unknown(code)],
        }
    }
}

/// OSC (Operating System Command) sequences
#[derive(Debug, Clone, PartialEq)]
pub enum OscSequence {
    SetWindowTitle(String),
    SetIconName(String),
    SetWindowTitleAndIconName(String),
    ResetWindowTitle,
    ResetIconName,
    Hyperlink(String),
    Unknown,
}

/// ANSI escape sequences
#[derive(Debug, Clone, PartialEq)]
pub enum AnsiSequence {
    // Cursor movement
    CursorUp(u16),
    CursorDown(u16),
    CursorForward(u16),
    CursorBackward(u16),
    CursorNextLine(u16),
    CursorPreviousLine(u16),
    CursorHorizontalAbsolute(u16),
    CursorPosition(u16, u16),
    
    // Cursor style
    CursorStyle(CursorStyle),
    CursorVisibility(bool),
    
    // Save/Restore cursor
    SaveCursor,
    RestoreCursor,
    
    // Display
    EraseInDisplay(EraseDisplayMode),
    EraseInLine(EraseLineMode),
    ScrollUp(u16),
    ScrollDown(u16),
    ClearScreen,
    ClearLine,
    
    // SGR
    Sgr(Vec<SgrAttribute>),
    
    // Window
    SetWindowTitle(String),
    
    // Modes
    SetMode(TerminalMode),
    ResetMode(TerminalMode),
    
    // Reports
    ReportCursorPosition,
    ReportTerminalType,
    
    // Screen
    AlternateScreenBuffer(bool),
    
    // Misc
    Index,
    ReverseIndex,
    FullReset,
    Reset,
    Unknown,
}

/// Parse state for ANSI sequences
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnsiParseState {
    Text,
    Escape,
    Csi,
    Params,
    Intermediate,
    Private,
    Osc,
    Dcs,
    OscEscape,
    DcsEscape,
}

/// Result of parsing a byte
#[derive(Debug, Clone, PartialEq)]
pub enum AnsiParseResult {
    Character(char),
    Escape,
    Complete(AnsiSequence),
    Osc(OscSequence),
    Ignored,
}

/// ANSI escape sequence parser
pub struct AnsiParser {
    state: AnsiParseState,
    params: Vec<u16>,
    current_param: u16,
    intermediate_chars: Vec<char>,
    buffer: String,
}

impl AnsiParser {
    pub fn new() -> Self {
        Self {
            state: AnsiParseState::Text,
            params: Vec::new(),
            current_param: 0,
            intermediate_chars: Vec::new(),
            buffer: String::new(),
        }
    }

    pub fn parse(&mut self, byte: u8) -> AnsiParseResult {
        match self.state {
            AnsiParseState::Text => self.parse_text(byte),
            AnsiParseState::Escape => self.parse_escape(byte),
            AnsiParseState::Csi => self.parse_csi(byte),
            AnsiParseState::Params => self.parse_csi(byte),
            AnsiParseState::Intermediate => self.parse_intermediate(byte),
            AnsiParseState::Private => self.parse_private(byte),
            AnsiParseState::Osc => self.parse_osc(byte),
            AnsiParseState::Dcs => self.parse_dcs(byte),
            AnsiParseState::OscEscape => self.parse_osc_escape(byte),
            AnsiParseState::DcsEscape => self.parse_dcs_escape(byte),
        }
    }

    pub fn reset(&mut self) {
        self.state = AnsiParseState::Text;
        self.params.clear();
        self.current_param = 0;
        self.intermediate_chars.clear();
        self.buffer.clear();
    }

    fn parse_text(&mut self, byte: u8) -> AnsiParseResult {
        match byte {
            b'\x1b' => {
                self.state = AnsiParseState::Escape;
                AnsiParseResult::Escape
            }
            _ => AnsiParseResult::Character(byte as char),
        }
    }

    fn parse_escape(&mut self, byte: u8) -> AnsiParseResult {
        match byte {
            b'[' => {
                self.state = AnsiParseState::Csi;
                self.params.clear();
                self.current_param = 0;
                AnsiParseResult::Escape
            }
            b'\\' => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Character('\\')
            }
            b'M' => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Complete(AnsiSequence::ReverseIndex)
            }
            b'D' => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Complete(AnsiSequence::Index)
            }
            b'c' => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Complete(AnsiSequence::FullReset)
            }
            b'7' => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Complete(AnsiSequence::SaveCursor)
            }
            b'8' => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Complete(AnsiSequence::RestoreCursor)
            }
            b'P' => {
                self.state = AnsiParseState::Dcs;
                AnsiParseResult::Escape
            }
            b']' => {
                self.state = AnsiParseState::Osc;
                self.buffer.clear();
                AnsiParseResult::Escape
            }
            _ => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Ignored
            }
        }
    }

    fn parse_csi(&mut self, byte: u8) -> AnsiParseResult {
        match byte {
            b'0'..=b'9' => {
                self.current_param = self.current_param * 10 + (byte - b'0') as u16;
                AnsiParseResult::Escape
            }
            b';' => {
                self.params.push(self.current_param);
                self.current_param = 0;
                AnsiParseResult::Escape
            }
            b'?' => {
                self.params.push(self.current_param);
                self.current_param = 0;
                self.state = AnsiParseState::Private;
                AnsiParseResult::Escape
            }
            b' '..=b'/' => {
                self.intermediate_chars.push(byte as char);
                self.state = AnsiParseState::Intermediate;
                AnsiParseResult::Escape
            }
            b'@'..=b'~' => {
                if !self.params.is_empty() || self.current_param != 0 {
                    self.params.push(self.current_param);
                }
                let sequence = self.parse_csi_final(byte);
                self.state = AnsiParseState::Text;
                AnsiParseResult::Complete(sequence)
            }
            _ => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Ignored
            }
        }
    }

    fn parse_intermediate(&mut self, byte: u8) -> AnsiParseResult {
        match byte {
            b' '..=b'/' => {
                self.intermediate_chars.push(byte as char);
                AnsiParseResult::Escape
            }
            b'@'..=b'~' => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Complete(AnsiSequence::Unknown)
            }
            _ => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Ignored
            }
        }
    }

    fn parse_private(&mut self, byte: u8) -> AnsiParseResult {
        match byte {
            b'0'..=b'9' => {
                self.current_param = self.current_param * 10 + (byte - b'0') as u16;
                AnsiParseResult::Escape
            }
            b';' => {
                self.params.push(self.current_param);
                self.current_param = 0;
                AnsiParseResult::Escape
            }
            b' '..=b'/' => {
                self.intermediate_chars.push(byte as char);
                self.state = AnsiParseState::Intermediate;
                AnsiParseResult::Escape
            }
            b'h' => {
                // DECSM - Set Mode
                if self.current_param != 0 || self.params.is_empty() {
                    self.params.push(self.current_param);
                }
                let private_code = self.params.last().copied().unwrap_or(0);
                if private_code == 25 {
                    self.state = AnsiParseState::Text;
                    return AnsiParseResult::Complete(AnsiSequence::CursorVisibility(true));
                }
                if private_code == 1049 {
                    self.state = AnsiParseState::Text;
                    return AnsiParseResult::Complete(AnsiSequence::AlternateScreenBuffer(true));
                }
                let mode = match private_code {
                    1 => TerminalMode::CursorKeys,
                    6 => TerminalMode::Origin,
                    7 => TerminalMode::AutoWrap,
                    12 => TerminalMode::CursorKeys,
                    25 => TerminalMode::CursorKeys,
                    _ => TerminalMode::CursorKeys,
                };
                self.state = AnsiParseState::Text;
                AnsiParseResult::Complete(AnsiSequence::SetMode(mode))
            }
            b'l' => {
                // DECRM - Reset Mode
                if self.current_param != 0 || self.params.is_empty() {
                    self.params.push(self.current_param);
                }
                let private_code = self.params.last().copied().unwrap_or(0);
                if private_code == 25 {
                    self.state = AnsiParseState::Text;
                    return AnsiParseResult::Complete(AnsiSequence::CursorVisibility(false));
                }
                if private_code == 1049 {
                    self.state = AnsiParseState::Text;
                    return AnsiParseResult::Complete(AnsiSequence::AlternateScreenBuffer(false));
                }
                let mode = match private_code {
                    1 => TerminalMode::CursorKeys,
                    6 => TerminalMode::Origin,
                    7 => TerminalMode::AutoWrap,
                    _ => TerminalMode::CursorKeys,
                };
                self.state = AnsiParseState::Text;
                AnsiParseResult::Complete(AnsiSequence::ResetMode(mode))
            }
            b'@'..=b'~' => {
                if !self.params.is_empty() || self.current_param != 0 {
                    self.params.push(self.current_param);
                }
                let sequence = self.parse_csi_final(byte);
                self.state = AnsiParseState::Text;
                AnsiParseResult::Complete(sequence)
            }
            _ => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Ignored
            }
        }
    }

    fn parse_osc(&mut self, byte: u8) -> AnsiParseResult {
        match byte {
            b'\x1b' => {
                self.state = AnsiParseState::OscEscape;
                AnsiParseResult::Escape
            }
            b'\x07' => {
                let sequence = self.parse_osc_sequence();
                self.state = AnsiParseState::Text;
                AnsiParseResult::Osc(sequence)
            }
            _ => {
                self.buffer.push(byte as char);
                AnsiParseResult::Escape
            }
        }
    }

    fn parse_dcs(&mut self, byte: u8) -> AnsiParseResult {
        match byte {
            b'\x1b' => {
                self.state = AnsiParseState::DcsEscape;
                AnsiParseResult::Escape
            }
            _ => AnsiParseResult::Escape,
        }
    }

    fn parse_osc_escape(&mut self, byte: u8) -> AnsiParseResult {
        match byte {
            b'\\' => {
                self.state = AnsiParseState::Text;
                let sequence = self.parse_osc_sequence();
                AnsiParseResult::Osc(sequence)
            }
            _ => {
                self.buffer.push(byte as char);
                AnsiParseResult::Escape
            }
        }
    }

    fn parse_dcs_escape(&mut self, byte: u8) -> AnsiParseResult {
        match byte {
            b'\\' => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Ignored
            }
            _ => {
                self.state = AnsiParseState::Text;
                AnsiParseResult::Ignored
            }
        }
    }

    fn parse_csi_final(&self, final_byte: u8) -> AnsiSequence {
        let default_params = [0u16];
        let params: &[u16] = if !self.params.is_empty() { &self.params } else { &default_params };

        match final_byte {
            b'A' => AnsiSequence::CursorUp(params[0]),
            b'B' => AnsiSequence::CursorDown(params[0]),
            b'C' => AnsiSequence::CursorForward(params[0]),
            b'D' => AnsiSequence::CursorBackward(params[0]),
            b'E' => AnsiSequence::CursorNextLine(params[0]),
            b'F' => AnsiSequence::CursorPreviousLine(params[0]),
            b'G' => AnsiSequence::CursorHorizontalAbsolute(if params[0] == 0 { 1 } else { params[0] }),
            b'H' | b'f' => {
                let row = if params[0] == 0 { 1 } else { params[0] };
                let col = if params.get(1).copied().unwrap_or(0) == 0 { 1 } else { params.get(1).copied().unwrap_or(1) };
                AnsiSequence::CursorPosition(row, col)
            }
            b'J' => {
                if params[0] == 2 {
                    return AnsiSequence::ClearScreen;
                }
                let mode = match params[0] {
                    0 => EraseDisplayMode::FromCursorToEnd,
                    1 => EraseDisplayMode::FromCursorToStart,
                    2 => EraseDisplayMode::All,
                    _ => EraseDisplayMode::FromCursorToEnd,
                };
                AnsiSequence::EraseInDisplay(mode)
            }
            b'K' => {
                if params[0] == 2 {
                    return AnsiSequence::ClearLine;
                }
                let mode = match params[0] {
                    0 => EraseLineMode::FromCursorToEnd,
                    1 => EraseLineMode::FromCursorToStart,
                    2 => EraseLineMode::All,
                    _ => EraseLineMode::FromCursorToEnd,
                };
                AnsiSequence::EraseInLine(mode)
            }
            b'S' => AnsiSequence::ScrollUp(params[0]),
            b'T' => AnsiSequence::ScrollDown(params[0]),
            b'm' => {
                let attrs: Vec<SgrAttribute> = params.iter()
                    .flat_map(|&p| SgrAttribute::from_code(p))
                    .collect();
                AnsiSequence::Sgr(attrs)
            }
            b' ' => {
                // Space - DECSET/DECRST private mode
                AnsiSequence::Unknown
            }
            b'q' => {
                let style = match params.get(0).copied().unwrap_or(0) {
                    0..=1 => CursorStyle::Block,
                    2 => CursorStyle::Beam,
                    3 => CursorStyle::Underline,
                    _ => CursorStyle::Block,
                };
                AnsiSequence::CursorStyle(style)
            }
            b'n' => AnsiSequence::ReportCursorPosition,
            _ => AnsiSequence::Unknown,
        }
    }

    fn parse_osc_sequence(&self) -> OscSequence {
        if self.buffer.starts_with("0;") || self.buffer.starts_with("0") {
            let title = self.buffer[2..].to_string();
            return OscSequence::SetWindowTitle(title);
        }
        if self.buffer.starts_with("8;;") {
            // Hyperlink
            return OscSequence::Hyperlink(self.buffer[3..].to_string());
        }
        OscSequence::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_parser_cursor_movement() {
        let mut parser = AnsiParser::new();
        
        // Test cursor up
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'3'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'A'), AnsiParseResult::Complete(AnsiSequence::CursorUp(3)));
        
        // Test cursor position
        parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'5'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b';'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'1'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'0'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'H'), AnsiParseResult::Complete(AnsiSequence::CursorPosition(5, 10)));
    }

    #[test]
    fn test_ansi_parser_sgr() {
        let mut parser = AnsiParser::new();
        
        // Test bold
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'1'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'm'), AnsiParseResult::Complete(AnsiSequence::Sgr(vec![SgrAttribute::Bold])));
        
        // Test red foreground
        parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'3'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'1'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'm'), AnsiParseResult::Complete(AnsiSequence::Sgr(vec![SgrAttribute::ForegroundColor(Color::Index(1))])));
    }

    #[test]
    fn test_ansi_parser_clear() {
        let mut parser = AnsiParser::new();
        
        // Test clear screen
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'2'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'J'), AnsiParseResult::Complete(AnsiSequence::ClearScreen));
    }

    #[test]
    fn test_ansi_parser_osc() {
        let mut parser = AnsiParser::new();
        
        // Test OSC window title
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b']'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'0'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b';'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b't'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'e'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b's'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b't'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'\x07'), AnsiParseResult::Osc(OscSequence::SetWindowTitle("test".to_string())));
    }

    #[test]
    fn test_cursor_default() {
        let cursor = Cursor::default();
        assert_eq!(cursor.column, 0);
        assert_eq!(cursor.row, 0);
        assert!(cursor.visible);
    }

    #[test]
    fn test_sgr_attribute_from_code() {
        assert_eq!(SgrAttribute::from_code(1), vec![SgrAttribute::Bold]);
        assert_eq!(SgrAttribute::from_code(31), vec![SgrAttribute::ForegroundColor(Color::Index(1))]);
        assert_eq!(SgrAttribute::from_code(0), vec![SgrAttribute::Reset]);
    }
}
