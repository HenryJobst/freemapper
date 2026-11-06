//! OpenOrienteering Mapper file format (XML, version 9).

pub mod coord;
pub mod enlarge;
pub mod model;
pub mod symbol_set;
pub mod writer;

pub use coord::MapCoord;
pub use model::{
    MapDocument, MapObject, MapObjectKind, MapPart, TemplateKind, TemplateRef, closed_ring,
};
pub use symbol_set::{SymbolCode, SymbolSet};
pub use writer::{XmlFormat, write_map};

#[derive(Debug, thiserror::Error)]
pub enum OmapError {
    #[error("invalid OOM XML: {0}")]
    Xml(String),
    #[error("element <{0}> not found")]
    MissingElement(&'static str),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
