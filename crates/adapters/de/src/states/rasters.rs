// SPDX-License-Identifier: AGPL-3.0-or-later

//! The raster backend of each agency.

use crate::geobasis_nrw::GeobasisNrwBackend;
use crate::geobremen::GeoBremenBackend;
use crate::geosn::GeosnBackend;
use crate::hvbg::HvbgBackend;
use crate::laiv_mv::LaivMvBackend;
use crate::ldbv::LdbvBackend;
use crate::lgb::LgbBackend;
use crate::lgl_bw::LglBwBackend;
use crate::lgln::LglnBackend;
use crate::lgv::LgvBackend;
use crate::lvermgeo_rp::LvermgeoRpBackend;
use crate::lvermgeo_sh::LvermgeoShBackend;
use crate::lvgl::LvglBackend;
use freemapper_adapter_api::rasters::RasterBackend;
use freemapper_sources::wms::WmsService;

use crate::crs::Crs;
use crate::states::{Agency, State};

/// The raster backend of `agency` that keeps its addresses in the code; `None` for LVermGeo,
/// whose rasters the manifest of Saxony-Anhalt declares as services, and for the agencies
/// whose `[orthophoto]` states the WMS they ask ([`addressed`], [`backend_with_wms`]).
pub fn backend(agency: Agency) -> Option<Box<dyn RasterBackend>> {
    Some(match agency {
        Agency::Lgb => Box::new(LgbBackend::new()),
        Agency::Lgv => Box::new(LgvBackend::new()),
        Agency::LaivMv => Box::new(LaivMvBackend::new()),
        Agency::GeoBremen => Box::new(GeoBremenBackend::new()),
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
        Agency::LglBw => (State::BadenWuerttemberg, Crs::Utm32),
        Agency::Ldbv => (State::Bayern, Crs::Utm32),
        _ => return None,
    })
}

/// The backend of `agency` that asks `wms` for the orthophoto; `None` for an agency that is
/// not [`addressed`].
pub fn backend_with_wms(agency: Agency, wms: WmsService) -> Option<Box<dyn RasterBackend>> {
    Some(match agency {
        Agency::GeoSn => Box::new(GeosnBackend::with_wms(wms)),
        Agency::Hvbg => Box::new(HvbgBackend::with_wms(wms)),
        Agency::LvermGeoSh => Box::new(LvermgeoShBackend::with_wms(wms)),
        Agency::GeobasisNrw => Box::new(GeobasisNrwBackend::with_wms(wms)),
        Agency::Lgln => Box::new(LglnBackend::with_wms(wms)),
        Agency::LvermGeoRp => Box::new(LvermgeoRpBackend::with_wms(wms)),
        Agency::Lvgl => Box::new(LvglBackend::with_wms(wms)),
        Agency::LglBw => Box::new(LglBwBackend::with_wms(wms)),
        Agency::Ldbv => Box::new(LdbvBackend::with_wms(wms)),
        _ => return None,
    })
}
