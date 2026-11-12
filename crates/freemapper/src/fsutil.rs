//! freemapper's side of the shared crate `freemapper-fs`: its wording of a refusal and the names
//! of partial files. The code lives in `freemapper-adapter-api`, because the readers write
//! through it too (ADR 0082); this module keeps the old paths working.

pub(crate) use freemapper_adapter_api::fs::*;
