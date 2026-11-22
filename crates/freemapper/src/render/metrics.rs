// SPDX-License-Identifier: AGPL-3.0-or-later

//! Text widths for the layout of the map sheet, from the font the PDF embeds (Arimo, metrically
//! equal to Arial and Helvetica; core ADR 0045), and the letters the OCAD export writes with
//! their base letter.

use freemapper_render::fonts::{face, text_width as arimo_width};

/// The Polish letters that have no WinAnsi code (`ó` and `Ó` are Latin-1) and the letter each is
/// written with in the OCAD export (ADR 0164): the OCAD file carries single-byte text. The PDF
/// shows the letters themselves since it embeds Arimo.
pub(crate) const POLISH_BASE: [(char, u8); 16] = [
    ('ą', b'a'),
    ('ć', b'c'),
    ('ę', b'e'),
    ('ł', b'l'),
    ('ń', b'n'),
    ('ś', b's'),
    ('ź', b'z'),
    ('ż', b'z'),
    ('Ą', b'A'),
    ('Ć', b'C'),
    ('Ę', b'E'),
    ('Ł', b'L'),
    ('Ń', b'N'),
    ('Ś', b'S'),
    ('Ź', b'Z'),
    ('Ż', b'Z'),
];

/// Advance width of `text` in em, in Arimo regular or bold.
pub(crate) fn text_width(text: &str, bold: bool) -> f64 {
    arimo_width(text, face(bold, false))
}

/// Arial / Helvetica ascent and line spacing in em (reference §6).
pub(crate) const ASCENT: f64 = 0.905;
pub(crate) const LINE_SPACING: f64 = 1.149;

#[cfg(test)]
mod tests {
    use super::*;

    /// Arimo has the letters of the languages the program knows, not only those of WinAnsi: a
    /// letter with a glyph does not count with the width of the fallback `.notdef`.
    #[test]
    fn widths_cover_the_letters_beyond_winansi() {
        let notdef = text_width("\u{4E2D}", false);
        for letter in ['ł', 'ą', 'č', 'ř', 'ő', 'ž', 'ș', 'ā'] {
            let text = letter.to_string();
            assert!(text_width(&text, false) > 0.0, "{letter}");
            assert!(
                (text_width(&text, false) - notdef).abs() > 1e-9,
                "{letter} has a glyph of its own"
            );
        }
    }

    #[test]
    fn bold_is_wider_than_regular() {
        assert!(text_width("Maßstab", true) > text_width("Maßstab", false));
    }

    /// The letters of the OCAD export keep their base letter (ADR 0164).
    #[test]
    fn polish_base_letters_are_listed() {
        assert_eq!(POLISH_BASE.len(), 16);
        assert!(POLISH_BASE.contains(&('ł', b'l')));
        assert!(POLISH_BASE.contains(&('Ż', b'Z')));
    }
}
