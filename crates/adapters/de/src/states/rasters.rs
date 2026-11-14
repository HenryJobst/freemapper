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

use crate::states::Agency;

/// The raster backend of `agency`; `None` for LVermGeo, whose rasters the manifest of
/// Saxony-Anhalt declares as services.
pub fn backend(agency: Agency) -> Option<Box<dyn RasterBackend>> {
    Some(match agency {
        Agency::Lgb => Box::new(LgbBackend::new()),
        Agency::GeoSn => Box::new(GeosnBackend::new()),
        Agency::LvermGeo => return None,
        Agency::Lgv => Box::new(LgvBackend::new()),
        Agency::Hvbg => Box::new(HvbgBackend::new()),
        Agency::LvermGeoSh => Box::new(LvermgeoShBackend::new()),
        Agency::LaivMv => Box::new(LaivMvBackend::new()),
        Agency::GeobasisNrw => Box::new(GeobasisNrwBackend::new()),
        Agency::Lgln => Box::new(LglnBackend::new()),
        Agency::LvermGeoRp => Box::new(LvermgeoRpBackend::new()),
        Agency::Lvgl => Box::new(LvglBackend::new()),
        Agency::LglBw => Box::new(LglBwBackend::new()),
        Agency::GeoBremen => Box::new(GeoBremenBackend::new()),
        Agency::Ldbv => Box::new(LdbvBackend::new()),
    })
}
