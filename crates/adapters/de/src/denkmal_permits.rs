//! The listed monuments of seven states for the permit notes (ADR 0144), read by the tables of
//! [`wfs_permits`](crate::wfs_permits): the WFS of the state offices for monuments in Hesse,
//! Baden-Württemberg, Bremen, Saxony, Brandenburg, Rhineland-Palatinate and North
//! Rhine-Westphalia. The credit lines are verbatim from the capabilities or the catalogue
//! entries (research `denkmale-2026-11-15.md`); Saxony's entry names CC BY 3.0 DE without a
//! credit text, so the line names the office that publishes the data.

use std::path::Path;

use freemapper_adapter_api::permits::{
    PermitFinding, PermitSection, PermitSource, PermitSourceInfo, Wording,
};
use geo_types::Rect;
use serde_json::Value;

use crate::fetch::{CacheDir, FetchError, Fetcher};

use crate::crs::Crs;
use crate::wfs_permits::{Layer, Reading, gml_source, native_source, source};

const MONUMENTS: &[PermitSection] = &[PermitSection::Monuments];

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

/// The city of Potsdam is not in the BLDAM's data.
const POTSDAM: Wording = Wording {
    en: "the city of Potsdam keeps its own list of monuments, which is not in these data",
    de: "Die Stadt Potsdam führt ihre Denkmalliste selbst; sie ist nicht in diesen Daten",
};

/// GML 3.2 as the MapServer, deegree and cardo services take it.
const GML32: &str = "application%2Fgml%2Bxml%3B%20version%3D3.2";

const fn monument(kind: &'static str) -> Reading {
    Reading::Monument {
        kind,
        kind_from: None,
        link: None,
    }
}

const fn linked(kind: &'static str, attribute: &'static str, list: &'static str) -> Reading {
    Reading::Monument {
        kind,
        kind_from: None,
        link: Some((attribute, list)),
    }
}

/// Hesse: DenkXweb of the Landesamt für Denkmalpflege Hessen, GeoJSON, with the link into the
/// monument list.
pub fn hessen() -> Box<dyn PermitSource> {
    const LIST: &str = "http://denkxweb.denkmalpflege-hessen.de/";
    source(
        "denkmale",
        Wording {
            en: "Landesamt für Denkmalpflege Hessen",
            de: "Landesamt für Denkmalpflege Hessen",
        },
        Wording {
            en: "Kulturdenkmäler, Gesamtanlagen, archaeological monuments (DenkXweb)",
            de: "Kulturdenkmäler, Gesamtanlagen, Bodendenkmäler (DenkXweb)",
        },
        "Datenlizenz Deutschland – Namensnennung – Version 2.0 (dl-de/by-2-0); Quellenvermerk: Landesamt für Denkmalpflege Hessen (LfDH)",
        &["kulturdenkmale", "gesamtanlagen", "bodendenkmale"],
        MONUMENTS,
        "https://geodienste.denkx.de/geoserver/denkx/wfs",
        "application/json",
        {
            const LAYERS: &[Layer] = &[
                (
                    "kulturdenkmale",
                    "denkx:baudenkmal",
                    linked("Kulturdenkmal", "publicationSource", LIST),
                ),
                (
                    "gesamtanlagen",
                    "denkx:flaechendenkmal",
                    linked("Gesamtanlage", "publicationSource", LIST),
                ),
                (
                    "bodendenkmale",
                    "denkx:archaeologischesDenkmal",
                    linked("Bodendenkmal", "publicationSource", LIST),
                ),
            ];
            LAYERS
        },
    )
}

const BW_CREDIT: &str = "Es gelten die Lizenzbedingungen „Datenlizenz Deutschland -Namensnennung -Version 2.0“ bzw. „dl-de/by-2-0” (https://www.govdata.de/dl-de/by-2-0) mit Angabe der Datenquelle: dl-de/by-2-0: Landesamt für Denkmalpflege im Regierungspräsidium Stuttgart, www.denkmalpflege-bw.de";

const BW_LABEL: Wording = Wording {
    en: "Landesamt für Denkmalpflege Baden-Württemberg",
    de: "Landesamt für Denkmalpflege Baden-Württemberg",
};

/// Baden-Württemberg: the two WFS of the Landesamt für Denkmalpflege behind the LGL's proxy,
/// GeoJSON; the type names go without their prefix.
pub fn baden_wuerttemberg() -> Vec<Box<dyn PermitSource>> {
    vec![
        source(
            "denkmale",
            BW_LABEL,
            Wording {
                en: "Kulturdenkmale and Gesamtanlagen",
                de: "Kulturdenkmale und Gesamtanlagen",
            },
            BW_CREDIT,
            &["kulturdenkmale", "gesamtanlagen"],
            MONUMENTS,
            "https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LAD_Kulturdenkmale_Bau_Kunstdenkmalpflege",
            "application/json",
            {
                const LAYERS: &[Layer] = &[
                    (
                        "kulturdenkmale",
                        "v_bau_kunstdenkmalpflege_kulturdenkmale",
                        monument("Kulturdenkmal"),
                    ),
                    (
                        "gesamtanlagen",
                        "v_bau_kunstdenkmalpflege_gesamtanlagen",
                        monument("Gesamtanlage"),
                    ),
                ];
                LAYERS
            },
        ),
        source(
            "archaeologie",
            BW_LABEL,
            Wording {
                en: "archaeological monuments and Grabungsschutzgebiete",
                de: "archäologische Kulturdenkmale und Grabungsschutzgebiete",
            },
            BW_CREDIT,
            &["kulturdenkmale", "grabungsschutzgebiete"],
            MONUMENTS,
            "https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LAD_Archaeologische_Kulturdenkmale_BW",
            "application/json",
            {
                const LAYERS: &[Layer] = &[
                    (
                        "kulturdenkmale",
                        "v_archaeologie_kulturdenkmale",
                        monument("Archäologisches Kulturdenkmal"),
                    ),
                    (
                        "grabungsschutzgebiete",
                        "v_archaeologie_grabungsschutzgebiete",
                        monument("Grabungsschutzgebiet"),
                    ),
                ];
                LAYERS
            },
        ),
    ]
}

/// Bremen: the Kulturdenkmale of the Landesamt für Denkmalpflege, GML 3.2, with the link into
/// the monument list. The archaeological monuments need the owner's consent and stay out.
pub fn bremen() -> Box<dyn PermitSource> {
    const LIST: &str = "https://denkmalpflege.bremen.de/";
    gml_source(
        "denkmale",
        Wording {
            en: "Landesamt für Denkmalpflege Bremen",
            de: "Landesamt für Denkmalpflege Bremen",
        },
        Wording {
            en: "Einzeldenkmale and Ensembles",
            de: "Einzeldenkmale und Ensembles",
        },
        "Creative Commons Namensnennung (CC-BY), Quellenvermerk: Freie Hansestadt Bremen, Senator für Kultur",
        &["einzeldenkmale", "ensembles"],
        MONUMENTS,
        "https://geodienste.bremen.de/wfs_kulturdenkmale",
        GML32,
        {
            const LAYERS: &[Layer] = &[
                (
                    "einzeldenkmale",
                    "app:einzeldenkmale",
                    linked("Einzeldenkmal", "link", LIST),
                ),
                (
                    "ensembles",
                    "app:ensembles",
                    linked("Ensemble", "link", LIST),
                ),
            ];
            LAYERS
        },
    )
}

/// Saxony: the Kulturdenkmale of the Landesamt für Denkmalpflege Sachsen (cardo), GML 3.2, as
/// areas and as points.
pub fn sachsen() -> Box<dyn PermitSource> {
    gml_source(
        "denkmale",
        Wording {
            en: "Landesamt für Denkmalpflege Sachsen",
            de: "Landesamt für Denkmalpflege Sachsen",
        },
        Wording {
            en: "Kulturdenkmale",
            de: "Kulturdenkmale",
        },
        "Landesamt für Denkmalpflege Sachsen, Creative Commons Namensnennung 3.0 Deutschland (CC BY 3.0 DE)",
        &["flaechen", "punkte"],
        MONUMENTS,
        "https://cardomap3.idu.de/lfds/public/ogc.ashx?PKGID=5",
        GML32,
        {
            const LAYERS: &[Layer] = &[
                ("flaechen", "cls:L1", monument("Kulturdenkmal")),
                ("punkte", "cls:L3", monument("Kulturdenkmal")),
            ];
            LAYERS
        },
    )
}

const BLDAM: Wording = Wording {
    en: "BLDAM",
    de: "BLDAM",
};

/// Brandenburg: the Baudenkmale and the Bodendenkmale of the BLDAM, GML 3.2, asked in its CRS
/// (a box in WGS 84 finds nothing there).
pub fn brandenburg() -> Vec<Box<dyn PermitSource>> {
    vec![
        with_gaps(
            native_source(
                Crs::Utm32,
                true,
                "baudenkmale",
                BLDAM,
                Wording {
                    en: "Baudenkmale, Gartendenkmale, Denkmalbereiche",
                    de: "Baudenkmale, Gartendenkmale, Denkmalbereiche",
                },
                "© Denkmaldaten/BLDAM {year}",
                &["punkte", "flaechen", "gartendenkmale", "denkmalbereiche"],
                MONUMENTS,
                "https://gis-bldam-brandenburg.de/ows/baudenkmale",
                GML32,
                {
                    const LAYERS: &[Layer] = &[
                        ("punkte", "ms:Baudenkmal-Punkt", monument("Baudenkmal")),
                        ("flaechen", "ms:Baudenkmal-Polygon", monument("Baudenkmal")),
                        (
                            "gartendenkmale",
                            "ms:Baudenkmal-Gartendenkmal",
                            monument("Gartendenkmal"),
                        ),
                        (
                            "denkmalbereiche",
                            "ms:Denkmal-mit-Flaechencharakter",
                            monument("Denkmalbereich"),
                        ),
                    ];
                    LAYERS
                },
            ),
            POTSDAM,
        ),
        native_source(
            Crs::Utm32,
            true,
            "bodendenkmale",
            BLDAM,
            Wording {
                en: "Bodendenkmale",
                de: "Bodendenkmale",
            },
            "Quelle: © Denkmaldaten/Brandenburgisches Landesamt für Denkmalpflege und Archäologisches Landesmuseum (BLDAM), dl-de/by-2-0",
            &["flaechen"],
            MONUMENTS,
            "https://gis-bldam-brandenburg.de/ows/bodendenkmale",
            GML32,
            {
                const LAYERS: &[Layer] = &[(
                    "flaechen",
                    "ms:Bodendenkmal-Flaechen",
                    monument("Bodendenkmal"),
                )];
                LAYERS
            },
        ),
    ]
}

/// Rhineland-Palatinate: the monuments of the GDKE through the geoportal's proxy, GeoJSON in
/// its CRS (with a box in WGS 84 it finds nothing).
pub fn rheinland_pfalz() -> Box<dyn PermitSource> {
    native_source(
        Crs::Utm32,
        false,
        "denkmale",
        Wording {
            en: "Generaldirektion Kulturelles Erbe",
            de: "Generaldirektion Kulturelles Erbe",
        },
        Wording {
            en: "Einzeldenkmäler, Denkmalzonen, bauliche Gesamtanlagen",
            de: "Einzeldenkmäler, Denkmalzonen, bauliche Gesamtanlagen",
        },
        "Datenlizenz Deutschland – Namensnennung – Version 2.0 (dl-de/by-2-0); Quellenvermerk: GDKE RLP",
        &["punkte", "flaechen", "denkmalzonen", "gesamtanlagen"],
        MONUMENTS,
        "https://www.geoportal.rlp.de/owsproxy/00000000000000000000000000000000/a2e0fb04feed3a72cf42d53cf6e2165e",
        "application/json",
        {
            const LAYERS: &[Layer] = &[
                ("punkte", "gdke:edm_punkte", monument("Einzeldenkmal")),
                ("flaechen", "gdke:edm_flaechen", monument("Einzeldenkmal")),
                ("denkmalzonen", "gdke:denkmalzonen", monument("Denkmalzone")),
                (
                    "gesamtanlagen",
                    "gdke:bga",
                    monument("Bauliche Gesamtanlage"),
                ),
            ];
            LAYERS
        },
    )
}

/// North Rhine-Westphalia: the monuments of the state's WFS, GML 3.2 in its CRS, each of the
/// kind its attribute `kategorie` names. The data are still being gathered Gemeinde by
/// Gemeinde.
pub fn nordrhein_westfalen() -> Box<dyn PermitSource> {
    const KIND: Reading = Reading::Monument {
        kind: "Denkmal",
        kind_from: Some("kategorie"),
        link: None,
    };
    with_gaps(
        native_source(
            Crs::Utm32,
            true,
            "denkmale",
            Wording {
                en: "Land NRW",
                de: "Land NRW",
            },
            Wording {
                en: "monuments under the Denkmalschutzgesetz",
                de: "Denkmäler nach dem Denkmalschutzgesetz",
            },
            "Datenlizenz Deutschland – Zero – Version 2.0 (https://www.govdata.de/dl-de/zero-2-0)",
            &["punkte", "flaechen"],
            MONUMENTS,
            "https://www.wfs.nrw.de/wfs/denkmal",
            GML32,
            {
                const LAYERS: &[Layer] = &[
                    ("punkte", "denkmal:denkmal_punkte", KIND),
                    ("flaechen", "denkmal:denkmal_flaechen", KIND),
                ];
                LAYERS
            },
        ),
        Wording {
            en: "the state gathers the monuments Gemeinde by Gemeinde, and not every Gemeinde is in them yet",
            de: "Das Land trägt die Denkmäler Gemeinde für Gemeinde zusammen, und noch ist nicht jede Gemeinde darin",
        },
    )
}
