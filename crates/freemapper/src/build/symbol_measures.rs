//! Measures of the symbols as the norm draws them at 100 %, shared by the rules of the build.
//!
//! A rule that needs the width of a line or the size of a point symbol takes it from here, so
//! each measure of the norm is written down once.

use crate::measure::{PaperMm, SymbolMm, SymbolScale};

/// The gap between two line symbols of one colour (ISSprOM §3.3).
pub const LINE_GAP_MM: SymbolMm = SymbolMm(0.15);
/// The width of the impassable wall (515).
pub const WALL_MM: SymbolMm = SymbolMm(0.40);
/// The width of the line of the impassable fence (518); its ticks reach [`FENCE_TICK_MM`] from
/// the centre line.
pub const FENCE_MM: SymbolMm = SymbolMm(0.40);
pub const FENCE_TICK_MM: SymbolMm = SymbolMm(0.65);
/// The width of the hedge (410.1).
pub const HEDGE_MM: SymbolMm = SymbolMm(0.40);
/// The passable retained wall (513.2): the width of its line, how far its half dots reach from
/// the centre line, and its width with the half dots.
pub const RETAINED_WALL_LINE_MM: SymbolMm = SymbolMm(0.21);
pub const RETAINED_WALL_TICK_MM: SymbolMm = SymbolMm(0.30);
pub const RETAINED_WALL_MM: SymbolMm = SymbolMm(0.60);

/// The impassable cliff (201, ISOM 2017-2): the width of its line, and how far its tags reach from
/// the centre line as the bundled symbol draws them.
pub const CLIFF_MM: SymbolMm = SymbolMm(0.35);
pub const CLIFF_TICK_MM: SymbolMm = SymbolMm(0.575);

/// The point symbols the build emits and their size on paper at ISSprOM 1:4000 (mm): the larger
/// of the norm's outer measure and the drawn extent (the larger side of its box) of the bundled
/// symbol. `true`: a native
/// school symbol, whose size is already the school size and is not enlarged.
pub const POINT_SIZES_MM: [(&str, f64, bool); 13] = [
    // Small fountain or well: OM 0.9.
    ("311", 0.90, false),
    // Spring: OM 1.2 × 0.6, the longer side.
    ("312", 1.20, false),
    // Prominent water feature: OM 1.2, drawn 1.45.
    ("313", 1.45, false),
    // Hydrant (school set).
    ("321", 1.30, true),
    // Prominent large tree: the white mask of the 2022 errata (1.4), the bundled set draws 1.2.
    ("417", 1.40, false),
    // Prominent bush or small tree.
    ("418", 0.80, false),
    // High tower: OM 2.1, drawn 2.35.
    ("524", 2.35, false),
    // Small tower: OM 1.5, drawn 1.72.
    ("525", 1.72, false),
    // Cairn, memorial or small monument: OM 1.0.
    ("526", 1.00, false),
    // Prominent man-made feature – x: OM 1.2, drawn 1.264.
    ("531", 1.27, false),
    // Table, pole or small pylon, lamppost (school set).
    ("542", 1.72, true),
    ("544", 1.10, true),
    ("545", 1.40, true),
];

/// The size of the point symbol `code` on the sheet of a map at `scale`: [`POINT_SIZES_MM`]
/// enlarged like the symbols (§3.1), the native school symbols as they are.
pub fn point_size_mm(code: &str, scale: SymbolScale) -> Option<PaperMm> {
    let &(_, size, native) = POINT_SIZES_MM.iter().find(|(c, _, _)| *c == code)?;
    Some(if native {
        PaperMm(size)
    } else {
        SymbolMm(size).on_paper(scale)
    })
}
