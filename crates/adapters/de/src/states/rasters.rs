//! The raster backend of each agency.

use crate::geobasis_nrw::GeobasisNrwBackend;
use crate::geosn::GeosnBackend;
use crate::hvbg::HvbgBackend;
use crate::laiv_mv::LaivMvBackend;
use crate::lgb::LgbBackend;
use crate::lgln::LglnBackend;
use crate::lgv::LgvBackend;
use crate::lvermgeo::LvermgeoBackend;
use crate::lvermgeo_rp::LvermgeoRpBackend;
use crate::lvermgeo_sh::LvermgeoShBackend;
use freemapper_adapter_api::rasters::RasterBackend;

use crate::states::Agency;

/// The raster backend of `provider`.
pub fn backend(agency: Agency) -> Box<dyn RasterBackend> {
    match agency {
        Agency::Lgb => Box::new(LgbBackend::new()),
        Agency::GeoSn => Box::new(GeosnBackend::new()),
        Agency::LvermGeo => Box::new(LvermgeoBackend::new()),
        Agency::Lgv => Box::new(LgvBackend::new()),
        Agency::Hvbg => Box::new(HvbgBackend::new()),
        Agency::LvermGeoSh => Box::new(LvermgeoShBackend::new()),
        Agency::LaivMv => Box::new(LaivMvBackend::new()),
        Agency::GeobasisNrw => Box::new(GeobasisNrwBackend::new()),
        Agency::Lgln => Box::new(LglnBackend::new()),
        Agency::LvermGeoRp => Box::new(LvermgeoRpBackend::new()),
    }
}
