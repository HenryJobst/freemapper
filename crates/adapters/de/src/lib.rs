//! The region adapters of the German states (ADR 0082): what freemapper needs to map each of
//! them. The crate knows the api crate and the generic readers of `freemapper-sources`, and
//! nothing of the engine.
//!
//! Terms follow `docs/glossary.md`.

pub mod geosn;
pub mod lgb;
pub mod tile_fetch;

// The state modules were written inside `freemapper`. These private modules give them the paths
// they use there, so the files moved without a change to what they import.

#[allow(unused_imports)]
mod crs {
    pub(crate) use freemapper_georef::crs::*;
}

#[allow(unused_imports)]
mod tiles {
    pub(crate) use freemapper_georef::tiles::*;
}

#[allow(unused_imports)]
mod note {
    pub(crate) use freemapper_adapter_api::note;
    pub(crate) use freemapper_adapter_api::note::*;
}

#[allow(unused_imports)]
mod fetch {
    pub(crate) use freemapper_adapter_api::cache::CacheDir;
    pub(crate) use freemapper_adapter_api::error::FetchError;
    pub(crate) use freemapper_adapter_api::fetch::*;

    #[cfg(test)]
    pub(crate) mod test_support {
        pub(crate) use freemapper_adapter_api::testing::MapFetcher;
        pub(crate) use freemapper_sources::testing::*;
    }
}

#[allow(unused_imports)]
mod raster {
    pub(crate) use freemapper_sources::raster::*;
}

#[allow(unused_imports)]
mod sources {
    pub(crate) use crate::{geosn, lgb};
    pub(crate) use freemapper_sources::wms;
}
