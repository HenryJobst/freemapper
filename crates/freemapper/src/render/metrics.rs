//! Helvetica and Helvetica-Bold advance widths (1/1000 em) for the WinAnsi characters 32–255, the
//! PDF standard fonts freemapper uses for text (metrically equal to Arial).
//!
//! Derived from Adobe's Core 14 AFM files `Helvetica.afm` and `Helvetica-Bold.afm`: only the widths
//! of the WinAnsi characters were extracted (modification by freemapper). The notices of the AFM
//! files and their ReadMe:
//!
//! Copyright (c) 1985, 1987, 1989, 1990, 1997 Adobe Systems Incorporated. All Rights Reserved.
//! Helvetica is a trademark of Linotype-Hell AG and/or its subsidiaries.
//!
//! This file and the 14 PostScript(R) AFM files it accompanies may be used, copied, and distributed
//! for any purpose and without charge, with or without modification, provided that all copyright
//! notices are retained; that the AFM files are not distributed without this file; that all
//! modifications to this file or any of the AFM files are prominently noted in the modified
//! file(s); and that this paragraph is not modified. Adobe Systems has no responsibility or
//! obligation to support the use of the AFM files.

/// Widths of WinAnsi bytes 32..=255; 0 for bytes without a character.
pub(crate) const HELVETICA: [u16; 224] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556,
    556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667,
    611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667,
    667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500,
    222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
    0, 556, 0, 222, 556, 333, 1000, 556, 556, 333, 1000, 667, 333, 1000, 0, 611, 0, 0, 222, 222,
    333, 333, 350, 556, 1000, 333, 1000, 500, 333, 944, 0, 500, 667, 278, 333, 556, 556, 556, 556,
    260, 556, 333, 737, 370, 556, 584, 333, 737, 333, 400, 584, 333, 333, 333, 556, 537, 278, 333,
    333, 365, 556, 834, 834, 834, 611, 667, 667, 667, 667, 667, 667, 1000, 722, 667, 667, 667, 667,
    278, 278, 278, 278, 722, 722, 778, 778, 778, 778, 778, 584, 778, 722, 722, 722, 722, 667, 667,
    611, 556, 556, 556, 556, 556, 556, 889, 500, 556, 556, 556, 556, 278, 278, 278, 278, 556, 556,
    556, 556, 556, 556, 556, 584, 611, 556, 556, 556, 556, 500, 556, 500,
];

pub(crate) const HELVETICA_BOLD: [u16; 224] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556,
    556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611, 975, 722, 722, 722, 722, 667,
    611, 778, 722, 278, 556, 722, 611, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667,
    667, 611, 333, 278, 333, 584, 556, 333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556,
    278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584,
    0, 556, 0, 278, 556, 500, 1000, 556, 556, 333, 1000, 667, 333, 1000, 0, 611, 0, 0, 278, 278,
    500, 500, 350, 556, 1000, 333, 1000, 556, 333, 944, 0, 500, 667, 278, 333, 556, 556, 556, 556,
    280, 556, 333, 737, 370, 556, 584, 333, 737, 333, 400, 584, 333, 333, 333, 611, 556, 278, 333,
    333, 365, 556, 834, 834, 834, 611, 722, 722, 722, 722, 722, 722, 1000, 722, 667, 667, 667, 667,
    278, 278, 278, 278, 722, 722, 778, 778, 778, 778, 778, 584, 778, 722, 722, 722, 722, 667, 667,
    611, 556, 556, 556, 556, 556, 556, 889, 556, 556, 556, 556, 556, 278, 278, 278, 278, 611, 611,
    611, 611, 611, 611, 611, 584, 611, 611, 611, 611, 611, 556, 611, 556,
];

/// WinAnsi (Windows-1252) characters 0x80–0x9F; the other bytes 0x20–0xFF equal Latin-1.
const CP1252_HIGH: [(char, u8); 27] = [
    ('€', 0x80),
    ('‚', 0x82),
    ('ƒ', 0x83),
    ('„', 0x84),
    ('…', 0x85),
    ('†', 0x86),
    ('‡', 0x87),
    ('ˆ', 0x88),
    ('‰', 0x89),
    ('Š', 0x8A),
    ('‹', 0x8B),
    ('Œ', 0x8C),
    ('Ž', 0x8E),
    ('‘', 0x91),
    ('’', 0x92),
    ('“', 0x93),
    ('”', 0x94),
    ('•', 0x95),
    ('–', 0x96),
    ('—', 0x97),
    ('˜', 0x98),
    ('™', 0x99),
    ('š', 0x9A),
    ('›', 0x9B),
    ('œ', 0x9C),
    ('ž', 0x9E),
    ('Ÿ', 0x9F),
];

/// The WinAnsi byte of a character; "?" for characters WinAnsi lacks.
pub(crate) fn win_ansi(c: char) -> u8 {
    match c as u32 {
        0x20..=0x7E | 0xA0..=0xFF => c as u8,
        _ => CP1252_HIGH
            .iter()
            .find(|(ch, _)| *ch == c)
            .map_or(b'?', |&(_, b)| b),
    }
}

/// Advance width of `text` in em.
pub(crate) fn text_width(text: &str, bold: bool) -> f64 {
    let table = if bold { &HELVETICA_BOLD } else { &HELVETICA };
    text.chars()
        .map(|c| f64::from(table[usize::from(win_ansi(c)) - 32]))
        .sum::<f64>()
        / 1000.0
}

/// Arial / Helvetica ascent and line spacing in em (reference §6).
pub(crate) const ASCENT: f64 = 0.905;
pub(crate) const LINE_SPACING: f64 = 1.149;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_ansi_covers_german_and_typography() {
        assert_eq!(win_ansi('A'), b'A');
        assert_eq!(win_ansi('ß'), 0xDF);
        assert_eq!(win_ansi('Ä'), 0xC4);
        assert_eq!(win_ansi('©'), 0xA9);
        assert_eq!(win_ansi('·'), 0xB7);
        assert_eq!(win_ansi('–'), 0x96);
        assert_eq!(win_ansi('€'), 0x80);
        assert_eq!(win_ansi('Ω'), b'?');
    }

    #[test]
    fn widths_follow_the_afm_files() {
        assert_eq!(text_width("A", false), 0.667);
        assert_eq!(text_width("A", true), 0.722);
        assert!(
            (text_width("Maßstab", false) - (0.833 + 0.556 + 0.611 + 0.5 + 0.278 + 0.556 + 0.556))
                .abs()
                < 1e-9
        );
        assert_eq!(text_width("Ω", false), text_width("?", false));
    }
}
