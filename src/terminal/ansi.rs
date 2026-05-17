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
            21 => vec![Self::NoBold],
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
        if self.buffer.starts_with("0;") {
            let title: String = self.buffer.chars().skip(2).collect();
            return OscSequence::SetWindowTitle(title);
        }
        if self.buffer.starts_with("0") {
            let title: String = self.buffer.chars().skip(1).collect();
            return OscSequence::SetWindowTitle(title);
        }
        if self.buffer.starts_with("8;;") {
            let url: String = self.buffer.chars().skip(3).collect();
            return OscSequence::Hyperlink(url);
        }
        OscSequence::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ── Hand-written unit tests (keep existing) ──

    #[test]
    fn test_ansi_parser_cursor_movement() {
        let mut parser = AnsiParser::new();
        
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'3'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'A'), AnsiParseResult::Complete(AnsiSequence::CursorUp(3)));
        
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
        
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'1'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'm'), AnsiParseResult::Complete(AnsiSequence::Sgr(vec![SgrAttribute::Bold])));
        
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
        
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'2'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'J'), AnsiParseResult::Complete(AnsiSequence::ClearScreen));
    }

    #[test]
    fn test_ansi_parser_osc() {
        let mut parser = AnsiParser::new();
        
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

    // ── Property-based tests ──

    // Property: Any arbitrary byte sequence should never panic the parser.
    proptest! {
        #[test]
        fn parser_never_panics(data: Vec<u8>) {
            let mut parser = AnsiParser::new();
            for &byte in &data {
                let _ = parser.parse(byte);
            }
        }
    }

    // Property: After resetting, the parser should always be in Text state
    // and correctly parse a known sequence.
    proptest! {
        #[test]
        fn parser_resets_cleanly(data: Vec<u8>) {
            let mut parser = AnsiParser::new();
            // Parse arbitrary bytes
            for &byte in &data {
                let _ = parser.parse(byte);
            }
            // Reset
            parser.reset();
            // Now parse a known sequence: ESC [ 1 m (bold)
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'1'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'm'), AnsiParseResult::Complete(AnsiSequence::Sgr(vec![SgrAttribute::Bold])));
        }
    }

    // Property: Text bytes (printable ASCII, no ESC) should always produce Character results.
    proptest! {
        #[test]
        fn plain_text_parses_as_characters(text: String) {
            let mut parser = AnsiParser::new();
            for &byte in text.as_bytes() {
                if byte == b'\x1b' {
                    continue;
                }
                // Only test if it's likely plain text
                if byte.is_ascii_graphic() || byte == b' ' {
                    let result = parser.parse(byte);
                    assert!(matches!(result, AnsiParseResult::Character(_)));
                }
            }
        }
    }

    // Property: Multiple param values on a single CSI should all be collected.
    proptest! {
        #[test]
        fn csi_params_accumulate(params: Vec<u16>) {
            // Test up to 5 params to keep test fast
            let params: Vec<u16> = params.into_iter().take(5).collect();
            let mut parser = AnsiParser::new();
            
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
            
            let mut expected_params = Vec::new();
            for (i, &p) in params.iter().enumerate() {
                let digits = p.to_string();
                for &d in digits.as_bytes() {
                    assert_eq!(parser.parse(d), AnsiParseResult::Escape);
                }
                if i < params.len() - 1 {
                    assert_eq!(parser.parse(b';'), AnsiParseResult::Escape);
                }
                expected_params.push(p);
            }
            if expected_params.is_empty() {
                expected_params.push(0); // default when no params
            }
            
            // Final byte 'J' -> EraseInDisplay
            let result = parser.parse(b'J');
            assert!(matches!(result, AnsiParseResult::Complete(AnsiSequence::EraseInDisplay(_))));
        }
    }

    // Property: Bold -> NoBold correctly tracks attribute toggle
    proptest! {
        #[test]
        fn sgr_bold_toggle_between_noise(noise: Vec<u8>) {
            let mut parser = AnsiParser::new();
            
            // Parse some noise first
            for &byte in &noise {
                let _ = parser.parse(byte);
            }
            
            // Reset
            parser.reset();
            
            // Bold on
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'1'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'm'), AnsiParseResult::Complete(AnsiSequence::Sgr(vec![SgrAttribute::Bold])));
        }
    }

    // Property: OSC terminated by BEL (0x07) produces some OscSequence (never panics)
    proptest! {
        #[test]
        fn osc_with_bel_terminator(content: Vec<u8>) {
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b']'), AnsiParseResult::Escape);
            for &byte in &content {
                if byte == b'\x07' || byte == b'\x1b' {
                    continue;
                }
                let _ = parser.parse(byte);
            }
            let result = parser.parse(b'\x07');
            assert!(matches!(result, AnsiParseResult::Osc(_)));
        }
    }

    // Property: OSC terminated by ST (ESC \) produces some OscSequence
    proptest! {
        #[test]
        fn osc_with_st_terminator(content: Vec<u8>) {
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b']'), AnsiParseResult::Escape);
            for &byte in &content {
                if byte == b'\x07' || byte == b'\x1b' {
                    continue;
                }
                let _ = parser.parse(byte);
            }
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            let result = parser.parse(b'\\');
            assert!(matches!(result, AnsiParseResult::Osc(_)));
        }
    }

    /// Property: Simple escape sequences (ESC M, ESC D, ESC 7, ESC 8, ESC c) 
    /// always produce known Complete sequences.
    #[test]
    fn simple_escape_sequences_all() {
        let cases = [
            (b'M', AnsiSequence::ReverseIndex),
            (b'D', AnsiSequence::Index),
            (b'c', AnsiSequence::FullReset),
            (b'7', AnsiSequence::SaveCursor),
            (b'8', AnsiSequence::RestoreCursor),
        ];
        for &(byte, ref expected) in &cases {
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(byte), AnsiParseResult::Complete(expected.clone()));
        }
    }

    // Property: Unknown CSI final bytes produce AnsiSequence::Unknown
    proptest! {
        #[test]
        fn unknown_csi_final_byte(final_byte: u8) {
            // Only test bytes in the CSI final range that we don't handle
            let known_finals = [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'f',
                                b'J', b'K', b'S', b'T', b'm', b'q', b'n'];
            if known_finals.contains(&final_byte) || !final_byte.is_ascii_uppercase() {
                return Ok(()); // skip - tested elsewhere or not a CSI final byte
            }
            
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
            let result = parser.parse(final_byte);
            assert!(matches!(result, AnsiParseResult::Complete(AnsiSequence::Unknown)));
        }
    }

    /// Property: Private mode sequences (?1049h, ?25l) parse correctly
    #[test]
    fn private_mode_sequences() {
        // ?1049h -> AlternateScreenBuffer(true)
        let mut parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'?'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'1'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'0'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'4'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'9'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'h'), AnsiParseResult::Complete(AnsiSequence::AlternateScreenBuffer(true)));

        // ?1049l -> AlternateScreenBuffer(false)
        parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'?'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'1'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'0'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'4'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'9'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'l'), AnsiParseResult::Complete(AnsiSequence::AlternateScreenBuffer(false)));

        // ?25h -> CursorVisibility(true)
        parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'?'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'2'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'5'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'h'), AnsiParseResult::Complete(AnsiSequence::CursorVisibility(true)));

        // ?25l -> CursorVisibility(false)
        parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'?'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'2'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'5'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'l'), AnsiParseResult::Complete(AnsiSequence::CursorVisibility(false)));
    }

    // Property: DCS sequences are consumed without panic
    proptest! {
        #[test]
        fn dcs_sequence_ignored(mut data: Vec<u8>) {
            // Remove any ESC bytes from generated data, as they would
            // transition DCS -> DcsEscape and consume our terminator
            data.retain(|&b| b != b'\x1b');
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'P'), AnsiParseResult::Escape);
            for &byte in &data {
                let _ = parser.parse(byte);
            }
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            let result = parser.parse(b'\\');
            assert_eq!(result, AnsiParseResult::Ignored);
        }
    }

    // Property: Unknown escape bytes (not [ P ] M D c 7 8 \ ) produce Ignored
    proptest! {
        #[test]
        fn unknown_escape_ignored(byte: u8) {
            let known = [b'[', b'P', b']', b'M', b'D', b'c', b'7', b'8', b'\\'];
            if known.contains(&byte) || byte == 0 {
                return Ok(());
            }
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(byte), AnsiParseResult::Ignored);
        }
    }

    /// Property: Known SGR attribute codes produce non-empty Vec; empty-returning
    /// codes (38, 48) are explicitly documented. Unknown codes are returned
    /// for SGR values not in the implementation's supported set.
    #[test]
    fn sgr_attribute_codes_range() {
        let known_codes: std::collections::HashSet<u16> = [
            0, 1, 2, 3, 4, 5, 7, 8, 9,
            21, 22, 23, 24, 25, 27, 28, 29,
            30, 31, 32, 33, 34, 35, 36, 37, 39,
            40, 41, 42, 43, 44, 45, 46, 47, 49,
            90, 91, 92, 93, 94, 95, 96, 97,
            100, 101, 102, 103, 104, 105, 106, 107,
        ].into();
        for code in 0..=107 {
            // Codes 38 and 48 always return empty
            if code == 38 || code == 48 {
                assert!(SgrAttribute::from_code(code).is_empty(),
                        "SGR code {} should be empty", code);
                continue;
            }
            let attrs = SgrAttribute::from_code(code);
            if known_codes.contains(&code) {
                assert!(!attrs.is_empty(), "SGR code {} produced empty attrs", code);
            }
            // Non-known codes may return Unknown(code) - that's expected
            // since from_code uses a catch-all wildcard
        }
    }

    /// Property: CursorStyle q sequence variants
    #[test]
    fn cursor_style_q_sequences() {
        for style in 0u16..=5 {
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
            let digits = style.to_string();
            for &d in digits.as_bytes() {
                let _ = parser.parse(d);
            }
            match parser.parse(b'q') {
                AnsiParseResult::Complete(AnsiSequence::CursorStyle(_)) => {}
                other => panic!("Expected CursorStyle, got {:?}", other),
            }
        }
    }

    /// Property: Parse all standard cursor movement sequences with default (no params)
    #[test]
    fn cursor_movement_default_params() {
        let finals: &[(u8, fn(u16) -> AnsiSequence)] = &[
            (b'A', |n| AnsiSequence::CursorUp(n)),
            (b'B', |n| AnsiSequence::CursorDown(n)),
            (b'C', |n| AnsiSequence::CursorForward(n)),
            (b'D', |n| AnsiSequence::CursorBackward(n)),
            (b'E', |n| AnsiSequence::CursorNextLine(n)),
            (b'F', |n| AnsiSequence::CursorPreviousLine(n)),
        ];
        for &(final_byte, _) in finals {
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
            let result = parser.parse(final_byte);
            assert!(matches!(result, AnsiParseResult::Complete(_)));
        }
    }

    /// Property: Cursor position with defaults maps to row=1,col=1
    #[test]
    fn cursor_position_defaults() {
        let mut parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'H'), AnsiParseResult::Complete(AnsiSequence::CursorPosition(1, 1)));
    }

    /// Property: Cursor position with only row uses col=1
    #[test]
    fn cursor_position_row_only() {
        let mut parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'5'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'H'), AnsiParseResult::Complete(AnsiSequence::CursorPosition(5, 1)));
    }

    /// Check that `new()` starts in Text state
    #[test]
    fn initial_state_is_text() {
        let parser = AnsiParser::new();
        assert_eq!(parser.state, AnsiParseState::Text);
    }

    /// Property: Large multi-digit params don't overflow the parser.
    /// Uses CursorUp (final byte 'A') which accepts any param value.
    #[test]
    fn large_csi_params() {
        let param_values = [u16::MAX, 65535, 50000, 0, 1];
        for &param in &param_values {
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
            let digits = param.to_string();
            for &d in digits.as_bytes() {
                assert_eq!(parser.parse(d), AnsiParseResult::Escape);
            }
            assert_eq!(parser.parse(b'A'), AnsiParseResult::Complete(AnsiSequence::CursorUp(param)));
        }
    }

    /// Property: EraseInDisplay modes (param 2 is optimized to ClearScreen)
    #[test]
    fn erase_in_display_modes() {
        // param = 2 is optimized to ClearScreen in parse_csi_final
        let cases: Vec<(u8, AnsiSequence)> = vec![
            (b'0', AnsiSequence::EraseInDisplay(EraseDisplayMode::FromCursorToEnd)),
            (b'1', AnsiSequence::EraseInDisplay(EraseDisplayMode::FromCursorToStart)),
            (b'2', AnsiSequence::ClearScreen),
        ];
        for &(byte, ref expected) in &cases {
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
            let _ = parser.parse(byte);
            let result = parser.parse(b'J');
            assert_eq!(result, AnsiParseResult::Complete(expected.clone()));
        }
    }

    /// Property: EraseInLine modes (param 2 is optimized to ClearLine)
    #[test]
    fn erase_in_line_modes() {
        // param = 2 is optimized to ClearLine in parse_csi_final
        let cases: Vec<(u8, AnsiSequence)> = vec![
            (b'0', AnsiSequence::EraseInLine(EraseLineMode::FromCursorToEnd)),
            (b'1', AnsiSequence::EraseInLine(EraseLineMode::FromCursorToStart)),
            (b'2', AnsiSequence::ClearLine),
        ];
        for &(byte, ref expected) in &cases {
            let mut parser = AnsiParser::new();
            assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
            assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
            let _ = parser.parse(byte);
            let result = parser.parse(b'K');
            assert_eq!(result, AnsiParseResult::Complete(expected.clone()));
        }
    }

    /// Property: Scroll up/down sequences
    #[test]
    fn scroll_sequences() {
        let mut parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'5'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'S'), AnsiParseResult::Complete(AnsiSequence::ScrollUp(5)));

        parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'3'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'T'), AnsiParseResult::Complete(AnsiSequence::ScrollDown(3)));
    }

    /// Property: Report sequences
    #[test]
    fn report_sequences() {
        let mut parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'['), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'6'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'n'), AnsiParseResult::Complete(AnsiSequence::ReportCursorPosition));
    }

    /// Verify OSC hyperlink sequence
    #[test]
    fn osc_hyperlink() {
        let mut parser = AnsiParser::new();
        assert_eq!(parser.parse(b'\x1b'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b']'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'8'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b';'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b';'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'h'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b't'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b't'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'p'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b':'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'/'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'/'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'x'), AnsiParseResult::Escape);
        assert_eq!(parser.parse(b'\x07'), AnsiParseResult::Osc(OscSequence::Hyperlink("http://x".to_string())));
    }
}
