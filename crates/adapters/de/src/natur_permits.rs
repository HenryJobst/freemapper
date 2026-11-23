// SPDX-License-Identifier: AGPL-3.0-or-later

//! Naturdenkmale, geschützte Landschaftsbestandteile and gesetzlich geschützte Biotope of the
//! states (ADR 0146), read by the tables of [`wfs_permits`](crate::wfs_permits). The national
//! sources hold none of them. The credit lines are verbatim from the capabilities or the
//! catalogue entries (research `naturschutzobjekte-2026-11-15.md`).

use freemapper_adapter_api::permits::{PermitSection, PermitSource, ProtectedObjectKind, Wording};

use crate::denkmal_permits::with_gaps;
use crate::wfs_permits::{Layer, Reading, gml_source, source};

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
    }
}

const ND: ProtectedObjectKind = ProtectedObjectKind::NaturalMonument;
const GLB: ProtectedObjectKind = ProtectedObjectKind::ProtectedLandscapePart;
const BIOTOPE: ProtectedObjectKind = ProtectedObjectKind::Biotope;

/// The service of the Bavarian LfU, which Bavaria's manifest names as a host.
pub const BAYERN: &str = "https://www.lfu.bayern.de/gdi/wfs/natur/schutzgebiete";

const LFULG: Wording = Wording {
    en: "Sächsisches Landesamt für Umwelt, Landwirtschaft und Geologie",
    de: "Sächsisches Landesamt für Umwelt, Landwirtschaft und Geologie",
};

const LFULG_CREDIT: &str = "Datenlizenz Deutschland 2.0 - Variante mit Namensnennung; Quelle: Sächsisches Landesamt für Umwelt, Landwirtschaft und Geologie";

/// Saxony: the Flächennaturdenkmale and the register of protected biotopes of the LfULG. The
/// Naturdenkmale that are single trees or rocks are kept by the Landkreise.
pub fn sachsen() -> Vec<Box<dyn PermitSource>> {
    vec![
        with_gaps(
            source(
                "naturdenkmale",
                LFULG,
                Wording {
                    en: "Flächennaturdenkmale",
                    de: "Flächennaturdenkmale",
                },
                LFULG_CREDIT,
                &["naturdenkmale"],
                OBJECTS,
                "https://luis.sachsen.de/arcgis/services/natur/schutzgebiete/MapServer/WFSServer",
                "GEOJSON",
                {
                    const LAYERS: &[Layer] = &[(
                        "naturdenkmale",
                        "schutzgebiete:Flaechennaturdenkmaeler",
                        object(ND, Some("NAME"), None),
                    )];
                    LAYERS
                },
            ),
            Wording {
                en: "Naturdenkmale that are single trees or rocks are kept by the Landkreise and are not in these data",
                de: "Naturdenkmale, die einzelne Bäume oder Felsen sind, führen die Landkreise; sie fehlen hier",
            },
        ),
        source(
            "biotope",
            LFULG,
            Wording {
                en: "register of the legally protected biotopes",
                de: "Verzeichnis der gesetzlich geschützten Biotope",
            },
            LFULG_CREDIT,
            &["flaechen", "punkte"],
            OBJECTS,
            "https://luis.sachsen.de/arcgis/services/natur/gesetz_gesch_biotope/MapServer/WFSServer",
            "GEOJSON",
            {
                const LAYERS: &[Layer] = &[
                    (
                        "flaechen",
                        "gesetz_gesch_biotope:BTPVZ_f_neu",
                        object(BIOTOPE, Some("BTTP26_BEZ"), None),
                    ),
                    (
                        "punkte",
                        "gesetz_gesch_biotope:BTPVZ_p_neu",
                        object(BIOTOPE, Some("BTTP26_BEZ"), None),
                    ),
                ];
                LAYERS
            },
        ),
    ]
}

/// Saxony-Anhalt: the Naturdenkmale and geschützte Landschaftsbestandteile of the LAU. Its
/// biotopes are only given out on request, its single trees are not in the service.
pub fn sachsen_anhalt() -> Box<dyn PermitSource> {
    with_gaps(
        source(
            "naturschutz",
            Wording {
                en: "Landesamt für Umweltschutz Sachsen-Anhalt",
                de: "Landesamt für Umweltschutz Sachsen-Anhalt",
            },
            Wording {
                en: "Naturdenkmale and geschützte Landschaftsbestandteile",
                de: "Naturdenkmale und geschützte Landschaftsbestandteile",
            },
            "Datenlizenz Deutschland Namensnennung 2.0; Quellenvermerk: Landesamt für Umweltschutz Sachsen-Anhalt",
            &["flaechen", "punkte", "landschaftsbestandteile"],
            OBJECTS,
            "https://geodatenportal.sachsen-anhalt.de/ows_lau_wfs_naturschutz",
            "GEOJSON",
            {
                const LAYERS: &[Layer] = &[
                    (
                        "flaechen",
                        "ps:Fl%C3%A4chenhafte_Naturdenkmale",
                        object(ND, Some("Gebietsname"), None),
                    ),
                    (
                        "punkte",
                        "ps:Fl%C3%A4chennaturdenkmale",
                        object(ND, Some("Gebietsname"), None),
                    ),
                    (
                        "landschaftsbestandteile",
                        "ps:Gesch%C3%BCtzte_Landschaftsbestandteile",
                        object(GLB, Some("Gebietsname"), None),
                    ),
                ];
                LAYERS
            },
        ),
        Wording {
            en: "Naturdenkmale that are single trees and the protected biotopes are not in them",
            de: "Naturdenkmale, die einzelne Bäume sind, und die geschützten Biotope fehlen darin",
        },
    )
}

/// Bavaria: the Naturdenkmale and Landschaftsbestandteile of the LfU.
pub fn bayern() -> Box<dyn PermitSource> {
    source(
        "naturschutz",
        Wording {
            en: "Bayerisches Landesamt für Umwelt",
            de: "Bayerisches Landesamt für Umwelt",
        },
        Wording {
            en: "Naturdenkmale and Landschaftsbestandteile",
            de: "Naturdenkmale und Landschaftsbestandteile",
        },
        "Creative Commons Namensnennung 4.0 International (CC BY 4.0); Datenquelle: Bayerisches Landesamt für Umwelt, www.lfu.bayern.de",
        &[
            "naturdenkmale-punkte",
            "naturdenkmale-flaechen",
            "landschaftsbestandteile-punkte",
            "landschaftsbestandteile-flaechen",
        ],
        OBJECTS,
        BAYERN,
        "GEOJSON",
        {
            const LAYERS: &[Layer] = &[
                (
                    "naturdenkmale-punkte",
                    "natur_wfsschutzgebiete:naturdenkmal_punktfoermig",
                    object(ND, Some("name"), None),
                ),
                (
                    "naturdenkmale-flaechen",
                    "natur_wfsschutzgebiete:naturdenkmal_flaechig",
                    object(ND, Some("name"), None),
                ),
                (
                    "landschaftsbestandteile-punkte",
                    "natur_wfsschutzgebiete:landschaftsbestandteil_punktfoermig",
                    object(GLB, Some("name"), None),
                ),
                (
                    "landschaftsbestandteile-flaechen",
                    "natur_wfsschutzgebiete:landschaftsbestandteil_flaechig",
                    object(GLB, Some("name"), None),
                ),
            ];
            LAYERS
        },
    )
}

/// Baden-Württemberg: the Naturdenkmale of the LUBW, single ones and areas. The biotopes stay
/// out: their terms of use need a request for any commercial use.
pub fn baden_wuerttemberg() -> Vec<Box<dyn PermitSource>> {
    const LABEL: Wording = Wording {
        en: "LUBW",
        de: "LUBW",
    };
    vec![
        source(
            "naturdenkmale",
            LABEL,
            Wording {
                en: "Naturdenkmale, single",
                de: "Naturdenkmale, Einzelgebilde",
            },
            crate::lubw_permits::CREDIT,
            &["naturdenkmale"],
            OBJECTS,
            "https://rips-gdi.lubw.baden-wuerttemberg.de/arcgis/services/wfs/Naturdenkmal_Einzelgebilde/MapServer/WFSServer",
            "GEOJSON",
            {
                const LAYERS: &[Layer] = &[(
                    "naturdenkmale",
                    "Naturdenkmal_Einzelgebilde",
                    object(ND, Some("OBJEKT"), None),
                )];
                LAYERS
            },
        ),
        source(
            "flaechennaturdenkmale",
            LABEL,
            Wording {
                en: "Naturdenkmale, areas",
                de: "Naturdenkmale, flächenhaft",
            },
            crate::lubw_permits::CREDIT,
            &["naturdenkmale"],
            OBJECTS,
            "https://rips-gdi.lubw.baden-wuerttemberg.de/arcgis/services/wfs/Naturdenkmal_Flaechenhaft/MapServer/WFSServer",
            "GEOJSON",
            {
                const LAYERS: &[Layer] = &[(
                    "naturdenkmale",
                    "Naturdenkmal_Flaechenhaft",
                    object(ND, Some("OBJEKT"), None),
                )];
                LAYERS
            },
        ),
    ]
}

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

const LFU_RP: Wording = Wording {
    en: "Landesamt für Umwelt Rheinland-Pfalz",
    de: "Landesamt für Umwelt Rheinland-Pfalz",
};

const LFU_RP_CREDIT: &str = "Landesamt für Umwelt Rheinland-Pfalz; Nutzung entsprechend den Lizenzbestimmungen der Open Database License (ODbL) mit Quellenangabe";

/// Rhineland-Palatinate: the Naturdenkmale and geschützte Landschaftsbestandteile of the LfU's
/// nature conservation service (one WFS per topic, ODbL). Its biotope mapping holds the
/// unprotected biotopes too and stays out.
pub fn rheinland_pfalz() -> Vec<Box<dyn PermitSource>> {
    vec![
        source(
            "naturdenkmale",
            LFU_RP,
            Wording {
                en: "Naturdenkmale",
                de: "Naturdenkmale",
            },
            LFU_RP_CREDIT,
            &["naturdenkmale"],
            OBJECTS,
            "https://geoportal.naturschutz.rlp.de/kartendienst_naturschutz/apps/service/wfs_getmap.php?mapfile=naturdenkmale",
            "application/json;%20subtype=geojson",
            {
                const LAYERS: &[Layer] = &[(
                    "naturdenkmale",
                    "ms:naturdenkmale",
                    object(ND, Some("bezeichnung"), Some("rvolink")),
                )];
                LAYERS
            },
        ),
        source(
            "landschaftsbestandteile",
            LFU_RP,
            Wording {
                en: "geschützte Landschaftsbestandteile",
                de: "geschützte Landschaftsbestandteile",
            },
            LFU_RP_CREDIT,
            &["landschaftsbestandteile"],
            OBJECTS,
            "https://geoportal.naturschutz.rlp.de/kartendienst_naturschutz/apps/service/wfs_getmap.php?mapfile=glb",
            "application/json;%20subtype=geojson",
            {
                const LAYERS: &[Layer] = &[(
                    "landschaftsbestandteile",
                    "ms:glb",
                    object(GLB, Some("bezeichnung"), Some("rvolink")),
                )];
                LAYERS
            },
        ),
    ]
}

/// The INSPIRE download service of the protected sites of Schleswig-Holstein, which the state's
/// manifest names as a host.
pub const SCHLESWIG_HOLSTEIN: &str = "https://service.gdi-sh.de/SH_INSPIREDOWNLOAD_AI_PS";

/// Schleswig-Holstein: the Naturdenkmale and geschützte Landschaftsbestandteile of the INSPIRE
/// service (a deegree WFS that answers GML 3.2 only, CC BY 4.0). The Kreise reported few of
/// them: the Naturdenkmale come almost all from the Kreis Schleswig-Flensburg.
pub fn schleswig_holstein() -> Box<dyn PermitSource> {
    with_gaps(
        gml_source(
            "naturschutz",
            Wording {
                en: "INSPIRE protected sites of Schleswig-Holstein",
                de: "INSPIRE-Schutzgebiete Schleswig-Holstein",
            },
            Wording {
                en: "Naturdenkmale, geschützte Landschaftsbestandteile",
                de: "Naturdenkmale, geschützte Landschaftsbestandteile",
            },
            "© GeoBasis-DE/LVermGeo SH/CC BY 4.0",
            &["naturdenkmale", "landschaftsbestandteile"],
            OBJECTS,
            SCHLESWIG_HOLSTEIN,
            "application%2Fgml%2Bxml%3B%20version%3D3.2",
            {
                const LAYERS: &[Layer] = &[
                    (
                        "naturdenkmale",
                        "app:Naturdenkmale",
                        object(ND, Some("text"), None),
                    ),
                    (
                        "landschaftsbestandteile",
                        "app:geschuetzte_Landschaftsbestandteile",
                        object(GLB, Some("text"), None),
                    ),
                ];
                LAYERS
            },
        ),
        Wording {
            en: "The Naturdenkmale come almost only from the Kreis Schleswig-Flensburg, the Landschaftsbestandteile only from Kiel, Lübeck and the Kreise Plön, Schleswig-Flensburg and Stormarn; ask the lower nature conservation authority for the others",
            de: "Die Naturdenkmale stammen fast nur aus dem Kreis Schleswig-Flensburg, die Landschaftsbestandteile nur aus Kiel, Lübeck und den Kreisen Plön, Schleswig-Flensburg und Stormarn; die übrigen kennt die untere Naturschutzbehörde",
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_adapter_api::permits::FindingKind;

    macro_rules! fixture {
        ($name:literal) => {
            include_str!(concat!("../../../../tests/fixtures/permits-states/", $name))
        };
    }

    /// Schleswig-Holstein's INSPIRE answers in GML 3.2 become Naturdenkmale (points) and
    /// Landschaftsbestandteile (areas) named by their `text`, in WGS 84 with the longitude first.
    #[test]
    fn schleswig_holstein_objects_from_the_inspire_service() {
        use crate::fetch::test_support::MapFetcher;
        use geo_types::{Coord, Rect};
        let base = format!(
            "{SCHLESWIG_HOLSTEIN}?service=WFS&version=2.0.0&request=GetFeature\
             &outputFormat=application%2Fgml%2Bxml%3B%20version%3D3.2\
             &srsName=urn:ogc:def:crs:EPSG::4326&typeNames="
        );
        let fetcher = MapFetcher::default();
        for (t, file) in [
            ("app:Naturdenkmale", fixture!("sh-naturdenkmale.gml")),
            (
                "app:geschuetzte_Landschaftsbestandteile",
                fixture!("sh-landschaftsbestandteile.gml"),
            ),
        ] {
            fetcher.insert_prefix(&format!("{base}{t}&"), file.as_bytes().to_vec());
        }
        let source = schleswig_holstein();
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(Coord { x: 9.55, y: 54.60 }, Coord { x: 9.90, y: 54.78 });
        let layers = source
            .fetch(&fetcher, &bbox, &part.path().join("x.part"))
            .unwrap();
        let found = |i: usize| -> Vec<(ProtectedObjectKind, String)> {
            source
                .findings(source.info().layers[i], &layers[i])
                .into_iter()
                .map(|f| {
                    for c in geo::CoordsIter::coords_iter(&f.geometry) {
                        assert!(
                            (8.0..11.5).contains(&c.x) && (53.3..55.1).contains(&c.y),
                            "{c:?}"
                        );
                    }
                    match f.kind {
                        FindingKind::ProtectedObject { kind, name, .. } => (kind, name.unwrap()),
                        other => panic!("{other:?}"),
                    }
                })
                .collect()
        };
        let monuments = found(0);
        assert_eq!(monuments.len(), 4);
        assert!(monuments.iter().all(|(kind, _)| *kind == ND));
        assert!(
            monuments
                .iter()
                .any(|(_, name)| name == "6 Stieleichen an der Schlei zwischen Boknis und Pagerö")
        );
        assert_eq!(
            found(1),
            [
                "Biotopflächen Segeberger Landstrasse",
                "Schlamersdorfer Trockenhang",
                "Schmiederedder"
            ]
            .map(|name| (GLB, name.to_string()))
        );
        assert!(source.info().gaps.is_some());
    }
}
