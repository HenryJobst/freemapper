//! The generic readers of freemapper (ADR 0082): what reads a standard service or a common file
//! format and knows no state. A region adapter is built from them, by a manifest that names a
//! service or by code that calls them.
//!
//! Terms follow `docs/glossary.md`.

pub mod float_tiff;
pub mod height_model;
pub mod nas;
pub mod ogc_api;
pub mod raster;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod trees;
pub mod wcs;
pub mod wfs;
pub mod wms;
pub mod xlsx;
pub mod xml;
pub mod zip_range;

// The readers were written inside `freemapper`. These private modules give them the paths they
// use there, so the files moved without a change to what they import.

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
    pub(crate) use freemapper_adapter_api::error::FetchError;
    pub(crate) use freemapper_adapter_api::fetch::*;

    #[cfg(test)]
    pub(crate) mod test_support {
        pub(crate) use crate::testing::*;
        pub(crate) use freemapper_adapter_api::testing::MapFetcher;
    }
}

#[allow(unused_imports)]
mod fsutil {
    pub(crate) use freemapper_adapter_api::fs::*;
}
