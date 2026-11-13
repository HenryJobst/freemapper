//! The region adapters of the German states (ADR 0082): what freemapper needs to map each of
//! them. The crate knows the api crate and the generic readers of `freemapper-sources`, and
//! nothing of the engine.
//!
//! Terms follow `docs/glossary.md`.

pub mod berlin;
pub mod berlin_als;
pub mod berlin_permits;
pub mod boundaries;
pub mod geobasis_nrw;
pub mod geosn;
pub mod hvbg;
pub mod laiv_mv;
mod las_files;
pub mod lgb;
pub mod lgb_als;
pub mod lgln;
pub mod lgv;
pub mod lvermgeo;
pub mod lvermgeo_rp;
pub mod lvermgeo_rp_las;
pub mod lvermgeo_sh;
pub mod lvgl;
pub mod lvgl_las;
pub mod nrw_3dm;
pub mod states;
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

    /// The HTTP client of the engine, for the live tests of the state modules.
    #[cfg(test)]
    pub(crate) use freemapper::fetch::HttpFetcher;
    /// The cache as the tests of the state modules name it.
    #[cfg(test)]
    pub(crate) use freemapper_adapter_api::cache::CacheDir as Cache;

    #[cfg(any(test, feature = "testing"))]
    pub(crate) mod test_support {
        pub(crate) use freemapper_adapter_api::testing::MapFetcher;
        pub(crate) use freemapper_sources::testing::*;
    }
}

#[allow(unused_imports)]
mod clock {
    pub(crate) use freemapper_adapter_api::clock::*;
}

/// The map kinds, as the suite contract names them.
#[allow(unused_imports)]
mod project {
    pub(crate) use freemapper_contract::MapKind;
}

#[allow(unused_imports)]
mod raster {
    pub(crate) use freemapper_sources::raster::*;
}

#[allow(unused_imports)]
mod sources {
    /// The questions a state module asks about the boundary of its own state.
    pub(crate) mod region {
        pub(crate) use crate::boundaries::*;
    }
    pub(crate) use crate::{
        geobasis_nrw, geosn, hvbg, laiv_mv, lgb, lgln, lgv, lvermgeo, lvermgeo_rp, lvermgeo_sh,
    };
    pub(crate) use freemapper_sources::{
        float_tiff, height_model, metalink, ogc_api as alkis, wcs, wfs as alkis_wfs, wms, xlsx,
        zip_range,
    };
}

/// What `derive` does with the files a backend stores, for the tests of the backends.
#[cfg(test)]
mod derive {
    pub(crate) mod terrain {
        use std::path::Path;

        use crate::raster::{GridSpec, Raster};

        /// A Float32 GeoTIFF with its world file (`.tfw`) as a raster, `NaN` where it holds no
        /// data: what `derive` reads of a height model stored per map area.
        pub(crate) fn read_float_raster(path: &Path) -> Result<Raster<f32>, String> {
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            let world = std::fs::read(path.with_extension("tfw")).map_err(|e| e.to_string())?;
            let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(bytes))
                .map_err(|e| e.to_string())?;
            let (width, height) = decoder.dimensions().map_err(|e| e.to_string())?;
            let tiff::decoder::DecodingResult::F32(values) =
                decoder.read_image().map_err(|e| e.to_string())?
            else {
                return Err("not 32-bit float".to_string());
            };
            let grid = GridSpec::from_world_file(&String::from_utf8_lossy(&world), width, height)
                .map_err(|e| e.to_string())?;
            Ok(Raster {
                grid,
                data: values
                    .into_iter()
                    .map(|z| if z < -9998.0 { f32::NAN } else { z })
                    .collect(),
            })
        }
    }
}
