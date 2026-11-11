//! Progress lines and warnings as values (ADR 0092, stage 3). The types, the macro `note!` and
//! the catalogs live in `freemapper-adapter-api`, because the readers and the adapters report
//! through them (ADR 0082); this module keeps the old paths working.

pub use freemapper_adapter_api::note::*;

// The macro `note!` (and, by the same name, the module it comes from).
pub(crate) use freemapper_adapter_api::note;
