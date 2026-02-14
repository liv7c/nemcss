//! This module provides helpers to convert a cursor position
//! from utf16 to utf8.

use ropey::Rope;
use tower_lsp::lsp_types::{Position, PositionEncodingKind};

/// Converts a cursor position provided by the LSP client
/// to a byte offset in the text document.
/// The lsp col is the character number.
/// Position provided by the LSP client gives the following information:
/// - the line number
/// - the character number
///
/// This function checks the encoding to determine how to calculate the byte offset.
/// It manages 2 cases:
/// - utf8 encoding: the byte offset is the same as the character number
/// - utf16 encoding: the byte offset is calculated by counting the number of bytes
pub fn lsp_col_to_byte(rope: &Rope, position: &Position, encoding: &PositionEncodingKind) -> usize {
    let line = rope.line(position.line as usize);

    if *encoding == PositionEncodingKind::UTF8 {
        return position.character as usize;
    }

    let mut utf16_offset: u32 = 0;
    let mut byte_offset: usize = 0;

    for ch in line.chars() {
        if utf16_offset == position.character {
            break;
        }

        // returns the number of code units in the character
        // 1 code unit = 2 bytes for utf16
        utf16_offset += ch.len_utf16() as u32;
        byte_offset += ch.len_utf8();
    }

    byte_offset
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rope(s: &str) -> Rope {
        Rope::from(s)
    }

    fn pos(character: u32) -> Position {
        Position { line: 0, character }
    }

    #[test]
    fn test_utf8_passthrough() {
        let result = lsp_col_to_byte(&rope("hello"), &pos(3), &PositionEncodingKind::UTF8);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_utf16_ascii_same_as_byte() {
        let result = lsp_col_to_byte(&rope("hello"), &pos(3), &PositionEncodingKind::UTF16);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_utf16_emoji() {
        let result = lsp_col_to_byte(&rope("g😀lo"), &pos(3), &PositionEncodingKind::UTF16);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_utf16_2_byte_character() {
        let result = lsp_col_to_byte(&rope("café marcel"), &pos(5), &PositionEncodingKind::UTF16);
        assert_eq!(result, 6);
    }
}
