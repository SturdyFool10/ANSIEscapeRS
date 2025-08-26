//! ansi_interpreter.rs
//!
//! Efficient ANSI escape code parser skeleton with state machine and entry points.
//! This module will parse a string containing ANSI escape codes and produce
//! enums/objects describing the codes for downstream consumption.

use super::ansi_types::{
    AnsiEscape, Color, CursorMove, DeviceControl, Erase, EraseMode, SgrAttribute,
};

/// Represents a span of text affected by an ANSI code.
#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents a span of text affected by an ANSI escape code.
/// Used to annotate which range of the cleaned text is affected by a particular code.
pub struct AnsiSpan {
    /// Byte offset in the cleaned text where the span starts.
    pub start: usize,
    /// Byte offset (exclusive) where the span ends.
    pub end: usize,
    /// The set of SGR attributes affecting this span.
    pub codes: Vec<SgrAttribute>,
}

/// Represents a point event (e.g., cursor move) at a position in the text.
#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents a point event (e.g., cursor move) at a position in the text.
pub struct AnsiPoint {
    /// Byte offset in the cleaned text where the event occurs.
    pub pos: usize,
    /// The ANSI escape code at this position.
    pub code: AnsiEscape,
}

/// The full parse result: spans, points, and the cleaned text.
#[derive(Debug, Clone, PartialEq, Eq)]
/// The full parse result: spans, points, and the cleaned text.
/// Returned by the parser to describe the annotated output.
pub struct AnsiParseResult {
    /// The text with escape codes removed.
    pub text: String,
    /// Codes affecting ranges of the text.
    pub spans: Vec<AnsiSpan>,
    /// Codes at specific positions in the text.
    pub points: Vec<AnsiPoint>,
}

/// Skeleton for the ANSI escape code parser.
/// Skeleton for the ANSI escape code parser.
/// Parses a string containing ANSI escape codes and produces annotated results.
pub struct AnsiParser<'a> {
    input: &'a str,
    pos: usize,
    output_pos: usize, // Position in the cleaned text
                       // Additional state fields as needed
}

impl<'a> AnsiParser<'a> {
    /// Create a new parser for the given input.
    ///
    /// # Arguments
    /// * `input` - The string to parse for ANSI escape codes.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            output_pos: 0,
        }
    }

    /// Main entry point: parses the input and returns an annotated parse result.
    ///
    /// Returns an [`AnsiParseResult`] containing the cleaned text, spans, and points.
    pub fn parse_annotated(&mut self) -> AnsiParseResult {
        let mut cleaned = String::with_capacity(self.input.len());
        let mut spans = Vec::new();
        let mut points = Vec::new();
        use std::collections::BTreeSet;
        let mut active_sgrs = BTreeSet::new(); // BTreeSet for deterministic order
        let mut current_span_start: Option<usize> = None;
        let mut last_emitted_sgrs = BTreeSet::new();

        while self.pos < self.input.len() {
            if let Some((escapes, consumed)) = self.parse_next_escapes() {
                for escape in escapes {
                    // Only add non-SGR codes to points
                    if !matches!(escape, AnsiEscape::Sgr(_)) {
                        points.push(AnsiPoint {
                            pos: self.output_pos,
                            code: escape.clone(),
                        });
                    }

                    if let AnsiEscape::Sgr(sgr) = &escape {
                        match sgr {
                            SgrAttribute::Reset => {
                                // If there was an active span, close it
                                if let Some(start) = current_span_start.take() {
                                    if !last_emitted_sgrs.is_empty() {
                                        spans.push(AnsiSpan {
                                            start,
                                            end: self.output_pos,
                                            codes: last_emitted_sgrs.iter().cloned().collect(),
                                        });
                                    }
                                }
                                active_sgrs.clear();
                            }
                            _ => {
                                // If this SGR is already active, replace it (remove old, insert new)
                                // Remove any previous instance of the same SGR "type"
                                // For Foreground/Background/UnderlineColor, remove any previous of that type
                                match sgr {
                                    SgrAttribute::Foreground(_) => {
                                        active_sgrs
                                            .retain(|a| !matches!(a, SgrAttribute::Foreground(_)));
                                    }
                                    SgrAttribute::Background(_) => {
                                        active_sgrs
                                            .retain(|a| !matches!(a, SgrAttribute::Background(_)));
                                    }
                                    SgrAttribute::UnderlineColor(_) => {
                                        active_sgrs.retain(|a| {
                                            !matches!(a, SgrAttribute::UnderlineColor(_))
                                        });
                                    }
                                    _ => {
                                        active_sgrs.retain(|a| {
                                            std::mem::discriminant(a) != std::mem::discriminant(sgr)
                                        });
                                    }
                                }
                                active_sgrs.insert(sgr.clone());
                            }
                        }
                        // If the set of active SGRs changed, close the previous span and start a new one
                        if active_sgrs != last_emitted_sgrs {
                            if let Some(start) = current_span_start.take() {
                                if !last_emitted_sgrs.is_empty() {
                                    spans.push(AnsiSpan {
                                        start,
                                        end: self.output_pos,
                                        codes: last_emitted_sgrs.iter().cloned().collect(),
                                    });
                                }
                            }
                            if !active_sgrs.is_empty() {
                                current_span_start = Some(self.output_pos);
                            }
                            last_emitted_sgrs = active_sgrs.clone();
                        }
                    }
                }
                self.pos += consumed;
            } else {
                // Copy non-escape character to cleaned text
                if let Some(ch) = self.input[self.pos..].chars().next() {
                    cleaned.push(ch);
                    self.pos += ch.len_utf8();
                    self.output_pos += ch.len_utf8();
                } else {
                    // Should not happen, but break to avoid infinite loop
                    break;
                }
            }
        }
        // If a span is still open at the end, close it
        if let Some(start) = current_span_start.take() {
            if !last_emitted_sgrs.is_empty() {
                spans.push(AnsiSpan {
                    start,
                    end: self.output_pos,
                    codes: last_emitted_sgrs.iter().cloned().collect(),
                });
            }
        }
        // Filter out spans with matching start and end positions
        let spans = spans
            .into_iter()
            .filter(|span| span.start != span.end)
            .collect();

        AnsiParseResult {
            text: cleaned,
            spans,
            points,
        }
    }

    /// Parse the next ANSI escape code(s) from the current position, if any.
    /// Returns (Vec<AnsiEscape>, bytes_consumed) or None if not an escape sequence.
    fn parse_next_escapes(&self) -> Option<(Vec<AnsiEscape>, usize)> {
        let bytes = self.input.as_bytes();
        if self.pos + 2 > bytes.len() {
            return None;
        }
        // Check for ESC [
        if bytes[self.pos] == 0x1B && bytes[self.pos + 1] == b'[' {
            // Find the end of the CSI sequence (final byte is 0x40-0x7E)
            let mut end = self.pos + 2;
            while end < bytes.len() {
                let b = bytes[end];
                if (0x40..=0x7E).contains(&b) {
                    break;
                }
                end += 1;
            }
            if end >= bytes.len() {
                // Malformed sequence: skip the entire sequence from ESC to end of input
                let consumed = bytes.len() - self.pos;
                return Some((vec![], consumed));
            }
            let final_byte = bytes[end];
            // params should be everything between '[' and the final byte
            let params = &self.input[self.pos + 2..end];
            let consumed = end + 1 - self.pos;
            let mut escapes = Vec::new();
            // SGR (m)
            if final_byte == b'm' {
                let sgrs = parse_sgr(params);
                for sgr in sgrs {
                    escapes.push(AnsiEscape::Sgr(sgr));
                }
            } else if let Some(cursor) = parse_cursor(params, final_byte) {
                escapes.push(AnsiEscape::Cursor(cursor));
            } else if let Some(erase) = parse_erase(params, final_byte) {
                escapes.push(AnsiEscape::Erase(erase));
            } else if let Some(device) = parse_device(params, final_byte) {
                escapes.push(AnsiEscape::Device(device));
            }
            // Always skip the escape sequence in the cleaned text, even if unknown
            return Some((escapes, consumed));
        }
        None
    }
}

/// Parse SGR parameters (e.g., "1;31").
fn parse_sgr(params: &str) -> Vec<SgrAttribute> {
    let mut result = Vec::new();
    let mut iter = params.split(';').filter(|s| !s.is_empty());
    while let Some(param) = iter.next() {
        match param {
            "0" => result.push(SgrAttribute::Reset),
            "1" => result.push(SgrAttribute::Bold),
            "2" => result.push(SgrAttribute::Faint),
            "3" => result.push(SgrAttribute::Italic),
            "4" => result.push(SgrAttribute::Underline),
            "5" => result.push(SgrAttribute::BlinkSlow),
            "6" => result.push(SgrAttribute::BlinkRapid),
            "7" => result.push(SgrAttribute::Reverse),
            "8" => result.push(SgrAttribute::Conceal),
            "9" => result.push(SgrAttribute::CrossedOut),
            "30" => result.push(SgrAttribute::Foreground(Color::Black)),
            "31" => result.push(SgrAttribute::Foreground(Color::Red)),
            "32" => result.push(SgrAttribute::Foreground(Color::Green)),
            "33" => result.push(SgrAttribute::Foreground(Color::Yellow)),
            "34" => result.push(SgrAttribute::Foreground(Color::Blue)),
            "35" => result.push(SgrAttribute::Foreground(Color::Magenta)),
            "36" => result.push(SgrAttribute::Foreground(Color::Cyan)),
            "37" => result.push(SgrAttribute::Foreground(Color::White)),
            "90" => result.push(SgrAttribute::Foreground(Color::BrightBlack)),
            "91" => result.push(SgrAttribute::Foreground(Color::BrightRed)),
            "92" => result.push(SgrAttribute::Foreground(Color::BrightGreen)),
            "93" => result.push(SgrAttribute::Foreground(Color::BrightYellow)),
            "94" => result.push(SgrAttribute::Foreground(Color::BrightBlue)),
            "95" => result.push(SgrAttribute::Foreground(Color::BrightMagenta)),
            "96" => result.push(SgrAttribute::Foreground(Color::BrightCyan)),
            "97" => result.push(SgrAttribute::Foreground(Color::BrightWhite)),
            "40" => result.push(SgrAttribute::Background(Color::Black)),
            "41" => result.push(SgrAttribute::Background(Color::Red)),
            "42" => result.push(SgrAttribute::Background(Color::Green)),
            "43" => result.push(SgrAttribute::Background(Color::Yellow)),
            "44" => result.push(SgrAttribute::Background(Color::Blue)),
            "45" => result.push(SgrAttribute::Background(Color::Magenta)),
            "46" => result.push(SgrAttribute::Background(Color::Cyan)),
            "47" => result.push(SgrAttribute::Background(Color::White)),
            "100" => result.push(SgrAttribute::Background(Color::BrightBlack)),
            "101" => result.push(SgrAttribute::Background(Color::BrightRed)),
            "102" => result.push(SgrAttribute::Background(Color::BrightGreen)),
            "103" => result.push(SgrAttribute::Background(Color::BrightYellow)),
            "104" => result.push(SgrAttribute::Background(Color::BrightBlue)),
            "105" => result.push(SgrAttribute::Background(Color::BrightMagenta)),
            "106" => result.push(SgrAttribute::Background(Color::BrightCyan)),
            "107" => result.push(SgrAttribute::Background(Color::BrightWhite)),
            "38" | "48" | "58" => {
                // 38: fg, 48: bg, 58: underline color
                let color_type = param;
                if let Some(next) = iter.next() {
                    if next == "5" {
                        // 8-bit color: 38;5;<n> or 48;5;<n> or 58;5;<n>
                        if let Some(val) = iter.next() {
                            if let Ok(idx) = val.parse::<u8>() {
                                let color = Color::AnsiValue(idx);
                                match color_type {
                                    "38" => result.push(SgrAttribute::Foreground(color)),
                                    "48" => result.push(SgrAttribute::Background(color)),
                                    "58" => result.push(SgrAttribute::UnderlineColor(color)),
                                    _ => {}
                                }
                            }
                        }
                    } else if next == "2" {
                        // 24-bit color: 38;2;<r>;<g>;<b> or 48;2;<r>;<g>;<b> or 58;2;<r>;<g>;<b>
                        let r = iter.next().and_then(|v| v.parse::<u8>().ok());
                        let g = iter.next().and_then(|v| v.parse::<u8>().ok());
                        let b = iter.next().and_then(|v| v.parse::<u8>().ok());
                        if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                            let color = Color::Rgb24 { r, g, b };
                            match color_type {
                                "38" => result.push(SgrAttribute::Foreground(color)),
                                "48" => result.push(SgrAttribute::Background(color)),
                                "58" => result.push(SgrAttribute::UnderlineColor(color)),
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    result
}

/// Parse cursor movement codes.
fn parse_cursor(params: &str, final_byte: u8) -> Option<CursorMove> {
    let n = params.parse::<u16>().unwrap_or(1);
    match final_byte {
        b'A' => Some(CursorMove::Up(n)),
        b'B' => Some(CursorMove::Down(n)),
        b'C' => Some(CursorMove::Forward(n)),
        b'D' => Some(CursorMove::Backward(n)),
        b'E' => Some(CursorMove::NextLine(n)),
        b'F' => Some(CursorMove::PreviousLine(n)),
        b'G' => Some(CursorMove::HorizontalAbsolute(n)),
        b'H' | b'f' => {
            let mut split = params.split(';');
            let row = split
                .next()
                .and_then(|v| v.parse::<u16>().ok())
                .unwrap_or(1);
            let col = split
                .next()
                .and_then(|v| v.parse::<u16>().ok())
                .unwrap_or(1);
            Some(CursorMove::Position { row, col })
        }
        _ => None,
    }
}

/// Parse erase codes.
fn parse_erase(params: &str, final_byte: u8) -> Option<Erase> {
    let mode = match params {
        "0" | "" => EraseMode::ToEnd,
        "1" => EraseMode::ToStart,
        "2" => EraseMode::All,
        _ => return None,
    };
    match final_byte {
        b'J' => Some(Erase::Display(mode)),
        b'K' => Some(Erase::Line(mode)),
        _ => None,
    }
}

/// Parse device control codes (save/restore cursor, hide/show cursor).
fn parse_device(params: &str, final_byte: u8) -> Option<DeviceControl> {
    match (params, final_byte) {
        ("", b's') => Some(DeviceControl::SaveCursor),
        ("", b'u') => Some(DeviceControl::RestoreCursor),
        ("?25l", b'l') => Some(DeviceControl::HideCursor),
        ("?25h", b'h') => Some(DeviceControl::ShowCursor),
        ("?25", b'l') => Some(DeviceControl::HideCursor),
        ("?25", b'h') => Some(DeviceControl::ShowCursor),
        _ => None,
    }
}

/// Convenience function for one-shot annotated parsing.
/// Convenience function to parse a string for ANSI escape codes and return an annotated result.
///
/// # Arguments
/// * `input` - The string to parse.
///
/// # Returns
/// An [`AnsiParseResult`] with the cleaned text and all detected ANSI codes.
pub fn parse_ansi_annotated(input: &str) -> AnsiParseResult {
    AnsiParser::new(input).parse_annotated()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi_escape::ansi_types::*;

    #[test]
    fn test_parser_sgr_and_cursor() {
        let input = "A\x1B[31mB\x1B[0mC\x1B[2J";
        let result = parse_ansi_annotated(input);
        assert_eq!(result.text, "ABC");
        // SGR and erase/cursor codes should be detected as points (span logic not yet implemented)
        assert!(
            result
                .points
                .iter()
                .any(|p| matches!(p.code, AnsiEscape::Sgr(_)))
        );
        assert!(
            result
                .points
                .iter()
                .any(|p| matches!(p.code, AnsiEscape::Erase(_)))
        );
    }

    #[test]
    fn test_parser_basic_colors() {
        let input = "X\x1B[31mY\x1B[0mZ";
        let result = parse_ansi_annotated(input);
        assert_eq!(result.text, "XYZ");
        let sgr_points: Vec<_> = result
            .points
            .iter()
            .filter_map(|p| {
                if let AnsiEscape::Sgr(attr) = p.code {
                    Some(attr)
                } else {
                    None
                }
            })
            .collect();
        assert!(sgr_points.contains(&SgrAttribute::Foreground(Color::Red)));
        assert!(sgr_points.contains(&SgrAttribute::Reset));
    }

    #[test]
    fn test_parser_8bit_color() {
        let input = "A\x1B[38;5;123mB\x1B[0m";
        let result = parse_ansi_annotated(input);
        assert_eq!(result.text, "AB");
        let sgr_points: Vec<_> = result
            .points
            .iter()
            .filter_map(|p| {
                if let AnsiEscape::Sgr(attr) = p.code {
                    Some(attr)
                } else {
                    None
                }
            })
            .collect();
        assert!(sgr_points.contains(&SgrAttribute::Foreground(Color::AnsiValue(123))));
        assert!(sgr_points.contains(&SgrAttribute::Reset));
    }

    #[test]
    fn test_parser_24bit_color_fg_bg_underline() {
        let input = "A\x1B[38;2;10;20;30mB\x1B[48;2;40;50;60mC\x1B[58;2;70;80;90mD\x1B[0m";
        let result = parse_ansi_annotated(input);
        assert_eq!(result.text, "ABCD");
        let mut fg = false;
        let mut bg = false;
        let mut ul = false;
        for p in &result.points {
            if let AnsiEscape::Sgr(attr) = p.code {
                match attr {
                    SgrAttribute::Foreground(Color::Rgb24 {
                        r: 10,
                        g: 20,
                        b: 30,
                    }) => fg = true,
                    SgrAttribute::Background(Color::Rgb24 {
                        r: 40,
                        g: 50,
                        b: 60,
                    }) => bg = true,
                    SgrAttribute::UnderlineColor(Color::Rgb24 {
                        r: 70,
                        g: 80,
                        b: 90,
                    }) => ul = true,
                    _ => {}
                }
            }
        }
        assert!(fg, "Did not find 24-bit foreground color");
        assert!(bg, "Did not find 24-bit background color");
        assert!(ul, "Did not find 24-bit underline color");
    }

    #[test]
    fn test_parser_cursor_movement() {
        let input = "A\x1B[2BC";
        let result = parse_ansi_annotated(input);
        assert_eq!(result.text, "AC");
        let found = result
            .points
            .iter()
            .any(|p| matches!(p.code, AnsiEscape::Cursor(CursorMove::Down(2))));
        assert!(found, "Did not find CursorMove::Down(2)");
    }

    #[test]
    fn test_parser_erase_display_and_line() {
        let input = "A\x1B[2JB\x1B[1KC";
        let result = parse_ansi_annotated(input);
        assert_eq!(result.text, "ABC");
        let found_display = result
            .points
            .iter()
            .any(|p| matches!(p.code, AnsiEscape::Erase(Erase::Display(EraseMode::All))));
        let found_line = result
            .points
            .iter()
            .any(|p| matches!(p.code, AnsiEscape::Erase(Erase::Line(EraseMode::ToStart))));
        assert!(found_display, "Did not find Erase::Display(EraseMode::All)");
        assert!(found_line, "Did not find Erase::Line(EraseMode::ToStart)");
    }

    #[test]
    fn test_parser_device_control() {
        let input = "A\x1B[sB\x1B[uC\x1B[?25lD\x1B[?25hE";
        let result = parse_ansi_annotated(input);
        assert_eq!(result.text, "ABCDE");
        let mut save = false;
        let mut restore = false;
        let mut hide = false;
        let mut show = false;
        for p in &result.points {
            match p.code {
                AnsiEscape::Device(DeviceControl::SaveCursor) => save = true,
                AnsiEscape::Device(DeviceControl::RestoreCursor) => restore = true,
                AnsiEscape::Device(DeviceControl::HideCursor) => hide = true,
                AnsiEscape::Device(DeviceControl::ShowCursor) => show = true,
                _ => {}
            }
        }
        assert!(save, "Did not find DeviceControl::SaveCursor");
        assert!(restore, "Did not find DeviceControl::RestoreCursor");
        assert!(hide, "Did not find DeviceControl::HideCursor");
        assert!(show, "Did not find DeviceControl::ShowCursor");
    }

    #[test]
    fn test_parser_malformed_sequences() {
        // Malformed or incomplete escape sequences should be ignored/skipped
        let input = "A\x1B[31B\x1B[999ZC\x1B[38;2;1;2mD";
        let result = parse_ansi_annotated(input);
        assert_eq!(result.text, "ACD");
        // Should not panic or produce unknown codes
        for p in &result.points {
            match p.code {
                AnsiEscape::Sgr(_)
                | AnsiEscape::Cursor(_)
                | AnsiEscape::Erase(_)
                | AnsiEscape::Device(_) => {}
            }
        }
    }

    #[test]
    fn test_parser_multiple_sgr_in_one_sequence() {
        // Only the first SGR is returned as a point, but all should be parsed
        let input = "A\x1B[1;31;4mB\x1B[0m";
        let result = parse_ansi_annotated(input);
        assert_eq!(result.text, "AB");
        let sgr_points: Vec<_> = result
            .points
            .iter()
            .filter_map(|p| {
                if let AnsiEscape::Sgr(attr) = p.code {
                    Some(attr)
                } else {
                    None
                }
            })
            .collect();
        assert!(sgr_points.contains(&SgrAttribute::Bold));
        assert!(sgr_points.contains(&SgrAttribute::Foreground(Color::Red)));
        assert!(sgr_points.contains(&SgrAttribute::Underline));
        assert!(sgr_points.contains(&SgrAttribute::Reset));
    }
}
