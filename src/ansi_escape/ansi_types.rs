//! ansi_types.rs
//!
//! Enums representing the full capability of ANSI escape codes,
//! designed to make invalid states unrepresentable.
/// Select Graphic Rendition (SGR) attributes for text formatting.
/// Used to control style, color, and effects in ANSI escape codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SgrAttribute {
    /// Reset all attributes.
    Reset,
    /// Bold text.
    Bold,
    /// Faint text.
    Faint,
    /// Italic text.
    Italic,
    /// Underlined text.
    Underline,
    /// Slow blinking text.
    BlinkSlow,
    /// Rapid blinking text.
    BlinkRapid,
    /// Reverse video (swap foreground/background).
    Reverse,
    /// Concealed (hidden) text.
    Conceal,
    /// Crossed out (strikethrough) text.
    CrossedOut,
    /// Set foreground color.
    Foreground(Color),
    /// Set background color.
    Background(Color),
    /// Set underline color.
    UnderlineColor(Color),
}

/// Color specification for ANSI codes, supporting standard, 8-bit, and 24-bit colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Standard black.
    Black,
    /// Standard red.
    Red,
    /// Standard green.
    Green,
    /// Standard yellow.
    Yellow,
    /// Standard blue.
    Blue,
    /// Standard magenta.
    Magenta,
    /// Standard cyan.
    Cyan,
    /// Standard white.
    White,
    /// Bright black (gray).
    BrightBlack,
    /// Bright red.
    BrightRed,
    /// Bright green.
    BrightGreen,
    /// Bright yellow.
    BrightYellow,
    /// Bright blue.
    BrightBlue,
    /// Bright magenta.
    BrightMagenta,
    /// Bright cyan.
    BrightCyan,
    /// Bright white.
    BrightWhite,
    /// 8-bit color (0-255).
    AnsiValue(u8),
    /// 24-bit RGB color.
    Rgb24 { r: u8, g: u8, b: u8 },
}

/// Cursor movement commands for ANSI escape codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMove {
    /// Move cursor up by `u16` rows.
    Up(u16),
    /// Move cursor down by `u16` rows.
    Down(u16),
    /// Move cursor forward (right) by `u16` columns.
    Forward(u16),
    /// Move cursor backward (left) by `u16` columns.
    Backward(u16),
    /// Move cursor to beginning of next `u16` lines.
    NextLine(u16),
    /// Move cursor to beginning of previous `u16` lines.
    PreviousLine(u16),
    /// Move cursor to absolute horizontal position (column).
    HorizontalAbsolute(u16),
    /// Move cursor to specific row and column.
    Position { row: u16, col: u16 },
}

/// Erase display or line commands for clearing parts of the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Erase {
    /// Erase part or all of the display.
    Display(EraseMode),
    /// Erase part or all of the current line.
    Line(EraseMode),
}

/// Mode for erase operations (display or line).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraseMode {
    /// Erase from cursor to end of screen/line.
    ToEnd,
    /// Erase from cursor to beginning of screen/line.
    ToStart,
    /// Erase entire screen/line.
    All,
}

/// Device control commands for cursor and terminal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceControl {
    /// Save the current cursor position.
    SaveCursor,
    /// Restore the saved cursor position.
    RestoreCursor,
    /// Hide the cursor.
    HideCursor,
    /// Show the cursor.
    ShowCursor,
}

/// The top-level enum representing any ANSI escape code supported by this library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnsiEscape {
    /// Select Graphic Rendition (SGR) attribute.
    Sgr(SgrAttribute),
    /// Cursor movement command.
    Cursor(CursorMove),
    /// Erase display or line command.
    Erase(Erase),
    /// Device control command.
    Device(DeviceControl),
    // Extend with more ANSI capabilities as needed
}
