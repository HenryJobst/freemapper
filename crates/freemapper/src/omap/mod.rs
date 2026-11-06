//! OpenOrienteering Mapper file format (XML, version 9).

pub mod coord;
pub mod symbol_set;

pub use coord::MapCoord;
pub use symbol_set::{SymbolCode, SymbolSet};

#[derive(Debug, thiserror::Error)]
pub enum OmapError {
    #[error("invalid OOM XML: {0}")]
    Xml(String),
    #[error("element <{0}> not found")]
    MissingElement(&'static str),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
