// SPDX-License-Identifier: AGPL-3.0-or-later

//! The raster backend of each agency.

use crate::geobasis_nrw::GeobasisNrwBackend;
use crate::geobremen::{self, GeoBremenBackend};
use crate::geosn::GeosnBackend;
use crate::hvbg::HvbgBackend;
use crate::laiv_mv::{self, LaivMvBackend};
use crate::ldbv::LdbvBackend;
use crate::lgb::LgbBackend;
use crate::lgl_bw::LglBwBackend;
use crate::lgln::{self, LglnBackend};
use crate::lgv::{self, LgvBackend};
use crate::lvermgeo_rp::LvermgeoRpBackend;
use crate::lvermgeo_sh::LvermgeoShBackend;
use crate::lvgl::LvglBackend;
use freemapper_adapter_api::rasters::{RasterBackend, RasterConfig};
use freemapper_sources::wms::WmsService;

use crate::crs::Crs;
use crate::states::{Agency, State};

/// The raster backend of `agency` that keeps its addresses in the code; `None` for LVermGeo,
/// whose rasters the manifest of Saxony-Anhalt declares as services, and for the agencies
/// whose `[orthophoto]` states the WMS they ask ([`addressed`], [`backend_with_wms`]).
pub fn backend(agency: Agency) -> Option<Box<dyn RasterBackend>> {
    Some(match agency {
        Agency::Lgb => Box::new(LgbBackend::new()),
        _ => return None,
    })
}

/// The state whose manifest holds the addresses of `agency`, and the CRS of its WMS, for the
/// agencies whose `[orthophoto]` says where the WMS is (issue #273); `None` for the others.
pub fn addressed(agency: Agency) -> Option<(State, Crs)> {
    Some(match agency {
        Agency::GeoSn => (State::Saxony, Crs::Utm33),
        Agency::Hvbg => (State::Hessen, Crs::Utm32),
        Agency::LvermGeoSh => (State::SchleswigHolstein, Crs::Utm32),
        Agency::GeobasisNrw => (State::NordrheinWestfalen, Crs::Utm32),
        Agency::Lgln => (State::Niedersachsen, Crs::Utm32),
        Agency::LvermGeoRp => (State::RheinlandPfalz, Crs::Utm32),
        Agency::Lvgl => (State::Saarland, Crs::Utm32),
        Agency::Lgv => (State::Hamburg, Crs::Utm32),
        Agency::LaivMv => (State::MecklenburgVorpommern, Crs::Utm33),
        Agency::GeoBremen => (State::Bremen, Crs::Utm32),
        Agency::LglBw => (State::BadenWuerttemberg, Crs::Utm32),
        Agency::Ldbv => (State::Bayern, Crs::Utm32),
        _ => return None,
    })
}

/// Whether the `[orthophoto]` of `agency` states a WMS (`url`, `layer`, `max_px`): not that of
/// the LAiV M-V, whose orthophoto comes from a WCS the `endpoints` name, nor that of GeoBremen,
/// whose WMS layers change with every flight.
pub fn takes_wms(agency: Agency) -> bool {
    !matches!(agency, Agency::LaivMv | Agency::GeoBremen)
}

/// The backend of an [`addressed`] agency that [`takes_wms`] no WMS, made of the addresses
/// `config` holds; `Ok(None)` for the others, an error when `config` lacks an address.
pub fn backend_without_wms(
    agency: Agency,
    config: &RasterConfig,
) -> Result<Option<Box<dyn RasterBackend>>, String> {
    Ok(Some(match agency {
        Agency::LaivMv => Box::new(LaivMvBackend::with_addresses(laiv_mv::Addresses::of(
            config,
        )?)),
        Agency::GeoBremen => Box::new(GeoBremenBackend::with_addresses(geobremen::Addresses::of(
            config,
        )?)),
        _ => return Ok(None),
    }))
}

/// The backend of `agency` that asks `wms` for the orthophoto and the other addresses `config`
/// holds (the built-in ones where the manifest left them out); `Ok(None)` for an agency that is
/// not [`addressed`], an error when `config` lacks an address the backend asks.
pub fn backend_with_wms(
    agency: Agency,
    wms: WmsService,
    config: &RasterConfig,
) -> Result<Option<Box<dyn RasterBackend>>, String> {
    Ok(Some(match agency {
        Agency::GeoSn => Box::new(GeosnBackend::with_wms(wms)),
        Agency::Hvbg => Box::new(HvbgBackend::with_wms(wms)),
        Agency::LvermGeoSh => Box::new(LvermgeoShBackend::with_wms(wms)),
        Agency::GeobasisNrw => Box::new(GeobasisNrwBackend::with_addresses(
            wms,
            config.require("geobasis-nrw", "hm")?,
        )),
        Agency::Lgln => Box::new(LglnBackend::with_addresses(
            wms,
            lgln::Addresses::of(config)?,
        )),
        Agency::LvermGeoRp => Box::new(LvermgeoRpBackend::with_wms(wms)),
        Agency::Lvgl => Box::new(LvglBackend::with_wms(wms)),
        Agency::LglBw => Box::new(LglBwBackend::with_wms(wms)),
        Agency::Ldbv => Box::new(LdbvBackend::with_wms(wms)),
        Agency::Lgv => Box::new(LgvBackend::with_addresses(wms, lgv::Addresses::of(config)?)),
        _ => return Ok(None),
    }))
}
