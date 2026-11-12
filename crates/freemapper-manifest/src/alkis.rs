//! `[alkis]` of a manifest as the ALKIS source of the readers.

use std::borrow::Cow;

use freemapper_adapter_api::alkis::AlkisSource;
use freemapper_sources::alkis_source::{Normaliser, OgcApiAlkis, WfsAlkis};
use freemapper_sources::wfs::WfsService;

use crate::model::{Alkis, Normalise, Wfs};

/// The WFS a manifest declares, as the reader asks it.
pub fn wfs_service(wfs: &Wfs) -> WfsService {
    WfsService {
        name: Cow::Owned(wfs.name.clone()),
        url: Cow::Owned(wfs.url.clone()),
        buildings_type: Cow::Owned(wfs.buildings_type.clone()),
        land_use_type: Cow::Owned(wfs.land_use_type.clone()),
        crs: wfs.crs,
        square_bez: wfs.square_bez.clone().map(Cow::Owned),
    }
}

/// The OGC API a manifest declares, with its fallback; `None` for a WFS.
pub fn ogc_api(alkis: &Alkis) -> Option<OgcApiAlkis> {
    match alkis {
        Alkis::Wfs(_) => None,
        Alkis::OgcApi {
            buildings,
            land_use,
            crs,
            normalise,
            fallback,
        } => Some(OgcApiAlkis {
            buildings: Cow::Owned(buildings.clone()),
            land_use: Cow::Owned(land_use.clone()),
            crs: *crs,
            fallback: fallback.as_ref().map(wfs_service),
            normalise: match normalise {
                Normalise::Lgb => Normaliser::Lgb,
                Normalise::HamburgAscii => Normaliser::HamburgAscii,
            },
        }),
    }
}

/// The ALKIS source of `[alkis]`: buildings and land use in the LGB schema.
pub fn alkis_source(alkis: &Alkis) -> Box<dyn AlkisSource> {
    match (alkis, ogc_api(alkis)) {
        (_, Some(api)) => Box::new(api),
        (Alkis::Wfs(wfs), None) => Box::new(WfsAlkis {
            service: wfs_service(wfs),
        }),
        (Alkis::OgcApi { .. }, None) => unreachable!("an OGC API table gives an OGC API source"),
    }
}
