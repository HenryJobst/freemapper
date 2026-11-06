//! Map coordinates as stored by OOM: 1/1000 mm on paper, y pointing south.

/// The coordinate starts a cubic Bézier curve (it and the next 3 coordinates).
pub const FLAG_CURVE_START: u8 = 1;
/// Last coordinate of a closed ring (equal to the ring's first coordinate).
pub const FLAG_CLOSE: u8 = 2;
/// Marks the end of a sub-path (ring).
pub const FLAG_HOLE: u8 = 16;
/// Dash point, effect depends on the symbol.
pub const FLAG_DASH: u8 = 32;

/// A map coordinate in 1/1000 mm with OOM coordinate flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MapCoord {
    pub x: i32,
    pub y: i32,
    pub flags: u8,
}

impl MapCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y, flags: 0 }
    }
}
