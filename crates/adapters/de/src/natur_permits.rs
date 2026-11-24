// SPDX-License-Identifier: AGPL-3.0-or-later

//! Naturdenkmale, geschützte Landschaftsbestandteile and gesetzlich geschützte Biotope of the
//! states (ADR 0146), read by the tables of [`wfs_permits`](crate::wfs_permits). The national
//! sources hold none of them. The credit lines are verbatim from the capabilities or the
//! catalogue entries (research `naturschutzobjekte-2026-11-15.md`).

use std::path::Path;

use freemapper_adapter_api::permits::{
    PermitFinding, PermitSection, PermitSource, PermitSourceInfo, ProtectedObjectKind, Wording,
};
use geo_types::Rect;
use serde_json::Value;

use crate::fetch::{CacheDir, FetchError, Fetcher};
use crate::wfs_permits::Reading;

/// A source whose data have the gaps `gaps` (ADR 0144); everything else is `inner`'s.
struct WithGaps {
    inner: Box<dyn PermitSource>,
    info: PermitSourceInfo,
}

fn with_gaps(inner: Box<dyn PermitSource>, gaps: Wording) -> Box<dyn PermitSource> {
    let info = PermitSourceInfo {
        gaps: Some(gaps),
        ..inner.info().clone()
    };
    Box::new(WithGaps { inner, info })
}

impl PermitSource for WithGaps {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        self.inner.fetch(fetcher, bbox, part)
    }

    fn fetch_cached(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
        cache: &CacheDir,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        self.inner.fetch_cached(fetcher, bbox, part, cache)
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        self.inner.findings(layer, features)
    }
}

const OBJECTS: &[PermitSection] = &[PermitSection::ProtectedObjects];

const fn object(
    kind: ProtectedObjectKind,
    name: Option<&'static str>,
    citation: Option<&'static str>,
) -> Reading {
    Reading::Object {
        kind,
        name,
        citation,
        only: None,
        point_names: &[],
    }
}

const ND: ProtectedObjectKind = ProtectedObjectKind::NaturalMonument;
const GLB: ProtectedObjectKind = ProtectedObjectKind::ProtectedLandscapePart;

/// The map service of the NLWKN, which Lower Saxony's manifest names as a host.
pub const NIEDERSACHSEN: &str =
    "https://www.umweltkarten-niedersachsen.de/arcgis/rest/services/Natur_wms/MapServer";

/// Lower Saxony: the Naturdenkmale and geschützte Landschaftsbestandteile of the NLWKN's map
/// service (ArcGIS REST, ADR 0148), areas and points. Its biotopes are kept by the Landkreise.
pub fn niedersachsen() -> Box<dyn PermitSource> {
    with_gaps(
        crate::arcgis_permits::source(
            "naturschutz",
            Wording {
                en: "NLWKN",
                de: "NLWKN",
            },
            Wording {
                en: "Naturdenkmale and geschützte Landschaftsbestandteile",
                de: "Naturdenkmale und geschützte Landschaftsbestandteile",
            },
            "Datenlizenz Deutschland – Namensnennung – Version 2.0 (dl-de/by-2-0); Quelle: Niedersächsischer Landesbetrieb für Wasserwirtschaft, Küsten- und Naturschutz (NLWKN)",
            &[
                "naturdenkmale-flaechen",
                "naturdenkmale-punkte",
                "landschaftsbestandteile-flaechen",
                "landschaftsbestandteile-punkte",
            ],
            OBJECTS,
            NIEDERSACHSEN,
            {
                const LAYERS: &[crate::arcgis_permits::RestLayer] = &[
                    (
                        "naturdenkmale-flaechen",
                        190,
                        object(ND, Some("name"), None),
                    ),
                    ("naturdenkmale-punkte", 7, object(ND, Some("name"), None)),
                    (
                        "landschaftsbestandteile-flaechen",
                        10,
                        object(GLB, Some("name"), None),
                    ),
                    (
                        "landschaftsbestandteile-punkte",
                        12,
                        object(GLB, Some("name"), None),
                    ),
                ];
                LAYERS
            },
        ),
        Wording {
            en: "the protected biotopes are kept by the Landkreise and are not in them",
            de: "Die geschützten Biotope führen die Landkreise; sie fehlen darin",
        },
    )
}
