// SPDX-License-Identifier: AGPL-3.0-or-later

//! Naturdenkmale, geschützte Landschaftsbestandteile and gesetzlich geschützte Biotope of the
//! states (ADR 0146), read by the tables of [`wfs_permits`](crate::wfs_permits). The national
//! sources hold none of them. The credit lines are verbatim from the capabilities or the
//! catalogue entries (research `naturschutzobjekte-2026-11-15.md`).

use freemapper_adapter_api::permits::{PermitSection, PermitSource, ProtectedObjectKind, Wording};

use crate::denkmal_permits::with_gaps;
use crate::wfs_permits::{Layer, Reading, source};

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
