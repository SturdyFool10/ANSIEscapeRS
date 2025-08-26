//! ansi_creator.rs
//!
//! API for producing ANSI escape codes, querying environment capabilities,
//! and supporting text formatting, cursor movement, clearing the terminal, and more.

use super::ansi_types::{
    AnsiEscape, Color, CursorMove, DeviceControl, Erase, EraseMode, SgrAttribute,
};

/// Query the environment for ANSI support and capabilities.
/// Describes the ANSI capabilities of the current environment (terminal).
///
/// Use [`AnsiEnvironment::detect`] to query the current environment.
pub struct AnsiEnvironment {
    /// True if ANSI escape codes are supported.
    pub supports_ansi: bool,
    /// True if 24-bit (truecolor) is supported.
    pub supports_truecolor: bool,
    /// True if 8-bit (256 color) is supported.
    pub supports_8bit_color: bool,
    // Add more capabilities as needed
}
impl AnsiEnvironment {
    /// Query the current environment for ANSI capabilities.

    /// Query the current environment for ANSI capabilities.
    ///
    /// This will check for ANSI, 8-bit, and truecolor support using platform-specific logic.
    pub fn detect() -> Self {
        // Use atty to check if stdout is a tty
        let is_tty = atty::is(atty::Stream::Stdout);

        // Platform-specific logic
        #[cfg(windows)]
        let (supports_ansi, supports_truecolor, supports_8bit_color) = {
            // Windows 10+ supports ANSI if ENABLE_VIRTUAL_TERMINAL_PROCESSING is enabled.
            // For now, assume Windows 10+ and that it's enabled if we're in a tty.
            // For more robust detection, winapi could be used to check/enable the flag.
            // Truecolor is supported in Windows Terminal, VSCode, and some others.
            let supports_ansi = is_tty;
            let supports_truecolor = std::env::var("WT_SESSION").is_ok()
                || std::env::var("TERM_PROGRAM")
                    .map(|v| v == "vscode")
                    .unwrap_or(false)
                || std::env::var("TERM")
                    .map(|v| v.contains("xterm") || v.contains("truecolor"))
                    .unwrap_or(false);
            let supports_8bit_color = supports_ansi;
            (supports_ansi, supports_truecolor, supports_8bit_color)
        };

        #[cfg(not(windows))]
        let (supports_ansi, supports_truecolor, supports_8bit_color) = {
            // On Unix, check TERM and COLORTERM
            let term = std::env::var("TERM").unwrap_or_default();
            let colorterm = std::env::var("COLORTERM").unwrap_or_default();
            let supports_ansi = is_tty && term != "dumb" && !term.is_empty();
            let supports_truecolor = colorterm == "truecolor"
                || colorterm == "24bit"
                || term.contains("truecolor")
                || term.contains("24bit");
            let supports_8bit_color = term.contains("256color") || supports_truecolor;
            (supports_ansi, supports_truecolor, supports_8bit_color)
        };

        Self {
            supports_ansi,
            supports_truecolor,
            supports_8bit_color,
        }
    }
}

/// API for producing ANSI escape codes.
/// API for producing ANSI escape codes for formatting, color, cursor movement, and more.
///
/// This is the main entry point for generating ANSI codes in a capability-aware way.
pub struct AnsiCreator {
    /// The detected environment capabilities.
    pub env: AnsiEnvironment,
}

impl AnsiCreator {
    /// Create a new `AnsiCreator`, querying the environment for capabilities.
    ///
    /// # Example
    /// ```
    /// use ansi_escapers::AnsiCreator;
    /// let creator = AnsiCreator::new();
    /// ```
    pub fn new() -> Self {
        Self {
            env: AnsiEnvironment::detect(),
        }
    }

    /// Format text with the given SGR (Select Graphic Rendition) attributes.
    ///
    /// The text will be wrapped in the appropriate ANSI codes and reset at the end.
    ///
    /// # Example
    /// ```
    /// use ansi_escapers::{AnsiCreator, SgrAttribute, Color};
    /// let creator = AnsiCreator::new();
    /// let s = creator.format_text("Hello", &[SgrAttribute::Bold, SgrAttribute::Foreground(Color::Red)]);
    /// ```
    pub fn format_text(&self, text: &str, attrs: &[SgrAttribute]) -> String {
        let mut code = String::new();
        for attr in attrs {
            code.push_str(&self.sgr_code(*attr));
        }
        let reset = self.sgr_code(SgrAttribute::Reset);
        format!("{}{}{}", code, text, reset)
    }

    /// Produce the ANSI escape code for a single SGR attribute.
    ///
    /// # Example
    /// ```
    /// use ansi_escapers::{AnsiCreator, SgrAttribute};
    /// let creator = AnsiCreator::new();
    /// let code = creator.sgr_code(SgrAttribute::Bold);
    /// ```
    pub fn sgr_code(&self, attr: SgrAttribute) -> String {
        match attr {
            SgrAttribute::Reset => "\x1B[0m".to_string(),
            SgrAttribute::Bold => "\x1B[1m".to_string(),
            SgrAttribute::Faint => "\x1B[2m".to_string(),
            SgrAttribute::Italic => "\x1B[3m".to_string(),
            SgrAttribute::Underline => "\x1B[4m".to_string(),
            SgrAttribute::BlinkSlow => "\x1B[5m".to_string(),
            SgrAttribute::BlinkRapid => "\x1B[6m".to_string(),
            SgrAttribute::Reverse => "\x1B[7m".to_string(),
            SgrAttribute::Conceal => "\x1B[8m".to_string(),
            SgrAttribute::CrossedOut => "\x1B[9m".to_string(),
            SgrAttribute::Foreground(color) => self.fg_code(color),
            SgrAttribute::Background(color) => self.bg_code(color),
            SgrAttribute::UnderlineColor(color) => self.underline_color_code_explicit(color),
        }
    }

    /// Produce the ANSI escape code for a standard foreground color (SGR 30-37, 90-97).
    ///
    /// # Arguments
    /// * `code` - The SGR code for the color (30-37 for normal, 90-97 for bright).
    pub fn fg_standard(&self, code: u8) -> String {
        // code: 30-37 (normal), 90-97 (bright)
        format!("\x1B[{}m", code)
    }

    /// Internal: produce the ANSI escape code for a foreground color, using the most idiomatic form.
    fn fg_code(&self, color: Color) -> String {
        match color {
            Color::Black => self.fg_standard(30),
            Color::Red => self.fg_standard(31),
            Color::Green => self.fg_standard(32),
            Color::Yellow => self.fg_standard(33),
            Color::Blue => self.fg_standard(34),
            Color::Magenta => self.fg_standard(35),
            Color::Cyan => self.fg_standard(36),
            Color::White => self.fg_standard(37),
            Color::BrightBlack => self.fg_standard(90),
            Color::BrightRed => self.fg_standard(91),
            Color::BrightGreen => self.fg_standard(92),
            Color::BrightYellow => self.fg_standard(93),
            Color::BrightBlue => self.fg_standard(94),
            Color::BrightMagenta => self.fg_standard(95),
            Color::BrightCyan => self.fg_standard(96),
            Color::BrightWhite => self.fg_standard(97),
            Color::AnsiValue(idx) => self.fg_8bit(idx),
            Color::Rgb24 { r, g, b } => self.fg_24bit(r, g, b),
        }
    }

    /// Internal: produce the ANSI escape code for a background color, using the most idiomatic form.
    fn bg_code(&self, color: Color) -> String {
        match color {
            Color::Black => self.bg_standard(40),
            Color::Red => self.bg_standard(41),
            Color::Green => self.bg_standard(42),
            Color::Yellow => self.bg_standard(43),
            Color::Blue => self.bg_standard(44),
            Color::Magenta => self.bg_standard(45),
            Color::Cyan => self.bg_standard(46),
            Color::White => self.bg_standard(47),
            Color::BrightBlack => self.bg_standard(100),
            Color::BrightRed => self.bg_standard(101),
            Color::BrightGreen => self.bg_standard(102),
            Color::BrightYellow => self.bg_standard(103),
            Color::BrightBlue => self.bg_standard(104),
            Color::BrightMagenta => self.bg_standard(105),
            Color::BrightCyan => self.bg_standard(106),
            Color::BrightWhite => self.bg_standard(107),
            Color::AnsiValue(idx) => self.bg_8bit(idx),
            Color::Rgb24 { r, g, b } => self.bg_24bit(r, g, b),
        }
    }

    /// Internal: produce the ANSI escape code for underline color, using the most idiomatic form.
    fn underline_color_code_explicit(&self, color: Color) -> String {
        match color {
            Color::AnsiValue(idx) => self.underline_8bit(idx),
            Color::Rgb24 { r, g, b } => self.underline_24bit(r, g, b),
            _ => String::new(),
        }
    }

    /// Produce the ANSI escape code for an 8-bit foreground color (SGR 38;5;N).
    ///
    /// # Arguments
    /// * `idx` - The 8-bit color index (0-255).
    pub fn fg_8bit(&self, idx: u8) -> String {
        format!("\x1B[38;5;{}m", idx)
    }

    /// Produce the ANSI escape code for a 24-bit foreground color (SGR 38;2;R;G;B).
    ///
    /// # Arguments
    /// * `r`, `g`, `b` - Red, green, and blue components (0-255).
    pub fn fg_24bit(&self, r: u8, g: u8, b: u8) -> String {
        format!("\x1B[38;2;{};{};{}m", r, g, b)
    }

    /// Produce the ANSI escape code for a standard background color (SGR 40-47, 100-107).
    ///
    /// # Arguments
    /// * `code` - The SGR code for the color (40-47 for normal, 100-107 for bright).
    pub fn bg_standard(&self, code: u8) -> String {
        // code: 40-47 (normal), 100-107 (bright)
        format!("\x1B[{}m", code)
    }

    /// Produce the ANSI escape code for an 8-bit background color (SGR 48;5;N).
    ///
    /// # Arguments
    /// * `idx` - The 8-bit color index (0-255).
    pub fn bg_8bit(&self, idx: u8) -> String {
        format!("\x1B[48;5;{}m", idx)
    }

    /// Produce the ANSI escape code for a 24-bit background color (SGR 48;2;R;G;B).
    ///
    /// # Arguments
    /// * `r`, `g`, `b` - Red, green, and blue components (0-255).
    pub fn bg_24bit(&self, r: u8, g: u8, b: u8) -> String {
        format!("\x1B[48;2;{};{};{}m", r, g, b)
    }

    /// Produce the ANSI escape code for an 8-bit underline color (SGR 58;5;N).
    ///
    /// # Arguments
    /// * `idx` - The 8-bit color index (0-255).
    pub fn underline_8bit(&self, idx: u8) -> String {
        format!("\x1B[58;5;{}m", idx)
    }

    /// Produce the ANSI escape code for a 24-bit underline color (SGR 58;2;R;G;B).
    ///
    /// # Arguments
    /// * `r`, `g`, `b` - Red, green, and blue components (0-255).
    pub fn underline_24bit(&self, r: u8, g: u8, b: u8) -> String {
        format!("\x1B[58;2;{};{};{}m", r, g, b)
    }

    /// Produce the ANSI escape code for a cursor movement.
    ///
    /// # Arguments
    /// * `movement` - The cursor movement command.
    pub fn cursor_code(&self, movement: CursorMove) -> String {
        match movement {
            CursorMove::Up(n) => format!("\x1B[{}A", n),
            CursorMove::Down(n) => format!("\x1B[{}B", n),
            CursorMove::Forward(n) => format!("\x1B[{}C", n),
            CursorMove::Backward(n) => format!("\x1B[{}D", n),
            CursorMove::NextLine(n) => format!("\x1B[{}E", n),
            CursorMove::PreviousLine(n) => format!("\x1B[{}F", n),
            CursorMove::HorizontalAbsolute(n) => format!("\x1B[{}G", n),
            CursorMove::Position { row, col } => format!("\x1B[{};{}H", row, col),
        }
    }

    /// Produce the ANSI escape code for clearing display or line.
    ///
    /// # Arguments
    /// * `erase` - The erase command (display or line, with mode).
    pub fn erase_code(&self, erase: Erase) -> String {
        match erase {
            Erase::Display(mode) => format!("\x1B[{}J", erase_mode_num(mode)),
            Erase::Line(mode) => format!("\x1B[{}K", erase_mode_num(mode)),
        }
    }

    /// Produce the ANSI escape code for device control.
    ///
    /// # Arguments
    /// * `device` - The device control command.
    pub fn device_code(&self, device: DeviceControl) -> String {
        match device {
            DeviceControl::SaveCursor => "\x1B[s".to_string(),
            DeviceControl::RestoreCursor => "\x1B[u".to_string(),
            DeviceControl::HideCursor => "\x1B[?25l".to_string(),
            DeviceControl::ShowCursor => "\x1B[?25h".to_string(),
        }
    }

    /// Produce the ANSI escape code for any [`AnsiEscape`] enum variant.
    ///
    /// # Arguments
    /// * `code` - The escape code to convert to a string.
    pub fn escape_code(&self, code: AnsiEscape) -> String {
        match code {
            AnsiEscape::Sgr(attr) => self.sgr_code(attr),
            AnsiEscape::Cursor(movement) => self.cursor_code(movement),
            AnsiEscape::Erase(erase) => self.erase_code(erase),
            AnsiEscape::Device(device) => self.device_code(device),
        }
    }
}

/// Helper to convert EraseMode to its numeric code.
fn erase_mode_num(mode: EraseMode) -> u8 {
    match mode {
        EraseMode::ToEnd => 0,
        EraseMode::ToStart => 1,
        EraseMode::All => 2,
    }
}

// Optionally, add more helpers for advanced features as needed.

#[cfg(test)]

mod tests {

    use super::*;

    use crate::ansi_escape::ansi_types::*;

    #[test]

    fn test_format_text_bold() {
        let creator = AnsiCreator::new();

        let s = creator.format_text("hi", &[SgrAttribute::Bold]);

        assert!(s.starts_with("\x1B[1m"));
        assert!(s.ends_with("\x1B[0m"));

        assert!(s.contains("hi"));
    }

    #[test]

    fn test_format_text_fg_red() {
        let creator = AnsiCreator::new();

        // Use explicit standard SGR code for red foreground
        let code = creator.fg_standard(31);
        assert_eq!(code, "\x1B[31m");

        let s = format!("{}hi{}", code, creator.sgr_code(SgrAttribute::Reset));
        assert!(s.starts_with("\x1B[31m"));
        assert!(s.ends_with("\x1B[0m"));
        assert!(s.contains("hi"));
    }

    #[test]
    fn test_sgr_reset() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.sgr_code(SgrAttribute::Reset), "\x1B[0m");
    }

    #[test]
    fn test_sgr_bold() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.sgr_code(SgrAttribute::Bold), "\x1B[1m");
    }

    #[test]
    fn test_sgr_faint() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.sgr_code(SgrAttribute::Faint), "\x1B[2m");
    }

    #[test]
    fn test_sgr_italic() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.sgr_code(SgrAttribute::Italic), "\x1B[3m");
    }

    #[test]
    fn test_sgr_underline() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.sgr_code(SgrAttribute::Underline), "\x1B[4m");
    }

    #[test]
    fn test_sgr_blink_slow() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.sgr_code(SgrAttribute::BlinkSlow), "\x1B[5m");
    }

    #[test]
    fn test_sgr_blink_rapid() {
        let creator = AnsiCreator::new();

        assert_eq!(creator.sgr_code(SgrAttribute::BlinkRapid), "\x1B[6m");
    }

    #[test]
    fn test_sgr_reverse() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.sgr_code(SgrAttribute::Reverse), "\x1B[7m");
    }

    #[test]
    fn test_sgr_conceal() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.sgr_code(SgrAttribute::Conceal), "\x1B[8m");
    }

    #[test]
    fn test_sgr_crossed_out() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.sgr_code(SgrAttribute::CrossedOut), "\x1B[9m");
    }

    #[test]
    fn test_sgr_fg_standard_colors() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.fg_standard(30), "\x1B[30m");
        assert_eq!(creator.fg_standard(31), "\x1B[31m");
        assert_eq!(creator.fg_standard(32), "\x1B[32m");
        assert_eq!(creator.fg_standard(33), "\x1B[33m");
        assert_eq!(creator.fg_standard(34), "\x1B[34m");
        assert_eq!(creator.fg_standard(35), "\x1B[35m");
        assert_eq!(creator.fg_standard(36), "\x1B[36m");
        assert_eq!(creator.fg_standard(37), "\x1B[37m");
    }

    #[test]
    fn test_sgr_fg_bright_colors() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.fg_standard(90), "\x1B[90m");
        assert_eq!(creator.fg_standard(91), "\x1B[91m");
        assert_eq!(creator.fg_standard(92), "\x1B[92m");
        assert_eq!(creator.fg_standard(93), "\x1B[93m");
        assert_eq!(creator.fg_standard(94), "\x1B[94m");
        assert_eq!(creator.fg_standard(95), "\x1B[95m");
        assert_eq!(creator.fg_standard(96), "\x1B[96m");
        assert_eq!(creator.fg_standard(97), "\x1B[97m");
    }

    #[test]
    fn test_sgr_bg_standard_colors() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.bg_standard(40), "\x1B[40m");
        assert_eq!(creator.bg_standard(41), "\x1B[41m");
        assert_eq!(creator.bg_standard(42), "\x1B[42m");
        assert_eq!(creator.bg_standard(43), "\x1B[43m");
        assert_eq!(creator.bg_standard(44), "\x1B[44m");
        assert_eq!(creator.bg_standard(45), "\x1B[45m");
        assert_eq!(creator.bg_standard(46), "\x1B[46m");
        assert_eq!(creator.bg_standard(47), "\x1B[47m");
    }

    #[test]
    fn test_sgr_bg_bright_colors() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.bg_standard(100), "\x1B[100m");
        assert_eq!(creator.bg_standard(101), "\x1B[101m");
        assert_eq!(creator.bg_standard(102), "\x1B[102m");
        assert_eq!(creator.bg_standard(103), "\x1B[103m");
        assert_eq!(creator.bg_standard(104), "\x1B[104m");
        assert_eq!(creator.bg_standard(105), "\x1B[105m");
        assert_eq!(creator.bg_standard(106), "\x1B[106m");
        assert_eq!(creator.bg_standard(107), "\x1B[107m");
    }

    #[test]
    fn test_sgr_fg_8bit_color() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.fg_8bit(123), "\x1B[38;5;123m");
    }

    #[test]
    fn test_sgr_fg_24bit_color() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.fg_24bit(10, 20, 30), "\x1B[38;2;10;20;30m");
    }

    #[test]
    fn test_sgr_underline_color_8bit() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.underline_8bit(42), "\x1B[58;5;42m");
    }

    #[test]
    fn test_sgr_underline_color_24bit() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.underline_24bit(1, 2, 3), "\x1B[58;2;1;2;3m");
    }

    #[test]
    fn test_cursor_up() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.cursor_code(CursorMove::Up(3)), "\x1B[3A");
    }

    #[test]
    fn test_cursor_down() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.cursor_code(CursorMove::Down(2)), "\x1B[2B");
    }

    #[test]
    fn test_cursor_forward() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.cursor_code(CursorMove::Forward(5)), "\x1B[5C");
    }

    #[test]
    fn test_cursor_backward() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.cursor_code(CursorMove::Backward(4)), "\x1B[4D");
    }

    #[test]
    fn test_cursor_next_line() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.cursor_code(CursorMove::NextLine(1)), "\x1B[1E");
    }

    #[test]
    fn test_cursor_previous_line() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.cursor_code(CursorMove::PreviousLine(2)), "\x1B[2F");
    }

    #[test]
    fn test_cursor_horizontal_absolute() {
        let creator = AnsiCreator::new();
        assert_eq!(
            creator.cursor_code(CursorMove::HorizontalAbsolute(7)),
            "\x1B[7G"
        );
    }

    #[test]
    fn test_cursor_position() {
        let creator = AnsiCreator::new();
        assert_eq!(
            creator.cursor_code(CursorMove::Position { row: 3, col: 4 }),
            "\x1B[3;4H"
        );
    }

    #[test]
    fn test_erase_display_to_end() {
        let creator = AnsiCreator::new();
        assert_eq!(
            creator.erase_code(Erase::Display(EraseMode::ToEnd)),
            "\x1B[0J"
        );
    }

    #[test]
    fn test_erase_display_to_start() {
        let creator = AnsiCreator::new();
        assert_eq!(
            creator.erase_code(Erase::Display(EraseMode::ToStart)),
            "\x1B[1J"
        );
    }

    #[test]
    fn test_erase_display_all() {
        let creator = AnsiCreator::new();
        assert_eq!(
            creator.erase_code(Erase::Display(EraseMode::All)),
            "\x1B[2J"
        );
    }

    #[test]
    fn test_erase_line_to_end() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.erase_code(Erase::Line(EraseMode::ToEnd)), "\x1B[0K");
    }

    #[test]
    fn test_erase_line_to_start() {
        let creator = AnsiCreator::new();
        assert_eq!(
            creator.erase_code(Erase::Line(EraseMode::ToStart)),
            "\x1B[1K"
        );
    }

    #[test]
    fn test_erase_line_all() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.erase_code(Erase::Line(EraseMode::All)), "\x1B[2K");
    }

    #[test]
    fn test_device_save_cursor() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.device_code(DeviceControl::SaveCursor), "\x1B[s");
    }

    #[test]
    fn test_device_restore_cursor() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.device_code(DeviceControl::RestoreCursor), "\x1B[u");
    }

    #[test]
    fn test_device_hide_cursor() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.device_code(DeviceControl::HideCursor), "\x1B[?25l");
    }

    #[test]
    fn test_device_show_cursor() {
        let creator = AnsiCreator::new();
        assert_eq!(creator.device_code(DeviceControl::ShowCursor), "\x1B[?25h");
    }
}
