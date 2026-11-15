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

/// Brandenburg: the Naturdenkmale and Landschaftsbestandteile of the Landkreise in the forest
/// service (data of 2017), and the biotope register of the LfU, of which only the protected
/// biotopes count.
pub fn brandenburg() -> Vec<Box<dyn PermitSource>> {
    vec![
        with_gaps(
            source(
                "naturdenkmale",
                Wording {
                    en: "Landesbetrieb Forst Brandenburg",
                    de: "Landesbetrieb Forst Brandenburg",
                },
                Wording {
                    en: "Naturdenkmale and geschützte Landschaftsbestandteile of the Landkreise",
                    de: "Naturdenkmale und geschützte Landschaftsbestandteile der Landkreise",
                },
                crate::lfb_permits::CREDIT,
                &["naturdenkmale", "landschaftsbestandteile"],
                OBJECTS,
                "https://www.brandenburg-forst.de/geoserver/IWFK/ows",
                "application/json",
                {
                    const LAYERS: &[Layer] = &[
                        (
                            "naturdenkmale",
                            "IWFK:lkrs_nd_pu",
                            object(ND, Some("name"), Some("rvo_rgl")),
                        ),
                        (
                            "landschaftsbestandteile",
                            "IWFK:lkrs_glb_fl",
                            object(GLB, Some("name"), Some("rvo_rgl")),
                        ),
                    ];
                    LAYERS
                },
            ),
            Wording {
                en: "the data are of the end of 2017, later ones are missing",
                de: "Die Daten haben den Stand Ende 2017, spätere fehlen",
            },
        ),
        source(
            "biotope",
            Wording {
                en: "Landesamt für Umwelt Brandenburg",
                de: "Landesamt für Umwelt Brandenburg",
            },
            Wording {
                en: "protected biotopes of the biotope register",
                de: "geschützte Biotope der Biotopkartierung",
            },
            "© Landesamt für Umwelt Brandenburg; Datenlizenz Deutschland – Namensnennung – Version 2.0 (https://www.govdata.de/dl-de/by-2-0)",
            &["flaechen", "punkte"],
            OBJECTS,
            "https://inspire.brandenburg.de/services/bbk_wfs",
            "application/geo%2Bjson",
            {
                const PROTECTED: Option<(&str, &[&str])> =
                    Some(("bbgnat_t", &["geschützter Biotop nach § 18"]));
                const LAYERS: &[Layer] = &[
                    (
                        "flaechen",
                        "app:bbk_fl",
                        Reading::Object {
                            kind: BIOTOPE,
                            name: Some("biotyp_t"),
                            citation: None,
                            only: PROTECTED,
                        },
                    ),
                    (
                        "punkte",
                        "app:bbk_pu",
                        Reading::Object {
                            kind: BIOTOPE,
                            name: Some("biotyp_t"),
                            citation: None,
                            only: PROTECTED,
                        },
                    ),
                ];
                LAYERS
            },
        ),
    ]
}

/// Berlin: the biotopes of the Umweltatlas whose protection is certain (`schutz_ges` 1, not the
/// suspected ones).
pub fn berlin() -> Box<dyn PermitSource> {
    const CERTAIN: Option<(&str, &[&str])> = Some(("schutz_ges", &["1"]));
    source(
        "biotope",
        crate::berlin_permits::LABEL,
        Wording {
            en: "protected biotopes (Umweltatlas, biotope types 2024)",
            de: "geschützte Biotope (Umweltatlas, Biotoptypen 2024)",
        },
        crate::berlin_permits::LICENCE,
        &["flaechen", "punkte"],
        OBJECTS,
        "https://gdi.berlin.de/services/wfs/ua_biotoptypen_2024",
        "application/json",
        {
            const LAYERS: &[Layer] = &[
                (
                    "flaechen",
                    "ua_biotoptypen_2024:bc_gesetzschutz_f",
                    Reading::Object {
                        kind: BIOTOPE,
                        name: Some("bt_name"),
                        citation: None,
                        only: CERTAIN,
                    },
                ),
                (
                    "punkte",
                    "ua_biotoptypen_2024:ba_gesetzschutz_p",
                    Reading::Object {
                        kind: BIOTOPE,
                        name: Some("bt_name"),
                        citation: None,
                        only: CERTAIN,
                    },
                ),
            ];
            LAYERS
        },
    )
}

/// Hamburg: the Naturdenkmale of the protected areas service and the protected biotopes of the
/// biotope register, each with the credit its service names.
pub fn hamburg() -> Vec<Box<dyn PermitSource>> {
    vec![
        source(
            "naturdenkmale",
            crate::lgv_permits::LABEL,
            Wording {
                en: "Naturdenkmale",
                de: "Naturdenkmale",
            },
            "Datenlizenz Deutschland Namensnennung 2.0, Quellenvermerk: Freie und Hansestadt Hamburg, Behörde für Umwelt und Energie",
            &["naturdenkmale"],
            OBJECTS,
            "https://geodienste.hamburg.de/HH_WFS_Schutzgebiete",
            "application/geo%2Bjson",
            {
                const LAYERS: &[Layer] = &[(
                    "naturdenkmale",
                    "de.hh.up:nr_f_naturdenkmal",
                    object(ND, Some("schutzgebiet"), Some("rechtsgrundlage")),
                )];
                LAYERS
            },
        ),
        source(
            "biotope",
            crate::lgv_permits::LABEL,
            Wording {
                en: "protected biotopes of the biotope register",
                de: "geschützte Biotope des Biotopkatasters",
            },
            "Datenlizenz Deutschland Namensnennung 2.0, Quellenvermerk: Freie und Hansestadt Hamburg, Behörde für Umwelt, Klima, Energie und Agrarwirtschaft",
            &["flaechen", "punkte"],
            OBJECTS,
            "https://geodienste.hamburg.de/HH_WFS_Biotopkataster",
            "application/geo%2Bjson",
            {
                const LAYERS: &[Layer] = &[
                    (
                        "flaechen",
                        "de.hh.up:flaechenhafte_biotope_geschuetzt",
                        object(BIOTOPE, Some("paragraf"), None),
                    ),
                    (
                        "punkte",
                        "de.hh.up:punkthafte_biotope_geschuetzt",
                        object(BIOTOPE, Some("paragraf"), None),
                    ),
                ];
                LAYERS
            },
        ),
    ]
}

/// Saarland: the Naturdenkmale and the protected biotopes of the Geoportal Saarland. Its
/// geschützte Landschaftsbestandteile stay out: one ordinance among them was found invalid, and
/// the table does not mark which.
pub fn saarland() -> Vec<Box<dyn PermitSource>> {
    vec![
        source(
            "naturdenkmale",
            crate::state_permits::SAARLAND,
            Wording {
                en: "Naturdenkmale",
                de: "Naturdenkmale",
            },
            crate::state_permits::SAARLAND_CREDIT,
            &["naturdenkmale"],
            OBJECTS,
            "https://geoportal.saarland.de/arcgis/services/Internet/Schutzgebiete_INS/MapServer/WFSServer",
            "GEOJSON",
            {
                const LAYERS: &[Layer] = &[(
                    "naturdenkmale",
                    "Naturdenkmal_Flaechen",
                    object(ND, Some("NAME"), Some("RECHTSGR")),
                )];
                LAYERS
            },
        ),
        source(
            "biotope",
            crate::state_permits::SAARLAND,
            Wording {
                en: "protected biotopes of the biotope register",
                de: "geschützte Biotope des Biotopkatasters",
            },
            crate::state_permits::SAARLAND_CREDIT,
            &["biotope"],
            OBJECTS,
            "https://geoportal.saarland.de/arcgis/services/Internet/Biotopkataster/MapServer/WFSServer",
            "GEOJSON",
            {
                const LAYERS: &[Layer] = &[(
                    "biotope",
                    "Geschuetzte_Biotope_f",
                    object(BIOTOPE, Some("BEZEICHNUNG"), None),
                )];
                LAYERS
            },
        ),
    ]
}

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

/// Hesse: the protected biotopes of the old Hessian biotope mapping (1992 to 2006, INSPIRE).
pub fn hessen() -> Box<dyn PermitSource> {
    with_gaps(
        source(
            "biotope",
            Wording {
                en: "HLNUG",
                de: "HLNUG",
            },
            Wording {
                en: "protected biotopes of the Hessian biotope mapping",
                de: "geschützte Biotope der Hessischen Biotopkartierung",
            },
            crate::state_permits::HLNUG_CREDIT,
            &["biotope"],
            OBJECTS,
            "https://geodienste-umwelt.hessen.de/arcgis/services/inspire/lebensraeume_biotope/MapServer/WFSServer",
            "GEOJSON",
            {
                const LAYERS: &[Layer] = &[(
                    "biotope",
                    "inspire_lebensraeume_biotope:GESCH_BIOTOPE",
                    object(BIOTOPE, Some("BIOTYP_BEZ"), None),
                )];
                LAYERS
            },
        ),
        Wording {
            en: "the mapping is of 1992 to 2006; newer biotopes are missing",
            de: "Die Kartierung stammt aus den Jahren 1992 bis 2006; neuere Biotope fehlen",
        },
    )
}

/// North Rhine-Westphalia: the protected biotopes of LINFOS, without their type (the service
/// gives it only in a report page).
pub fn nordrhein_westfalen() -> Box<dyn PermitSource> {
    source(
        "biotope",
        Wording {
            en: "LINFOS",
            de: "LINFOS",
        },
        Wording {
            en: "legally protected biotopes",
            de: "gesetzlich geschützte Biotope",
        },
        crate::wuh_permits::LINFOS_CREDIT,
        &["flaechen", "punkte"],
        OBJECTS,
        "https://www.wfs.nrw.de/umwelt/linfos",
        "GEOJSON",
        {
            const LAYERS: &[Layer] = &[
                (
                    "flaechen",
                    "wfs_linfos:gbt_polygon",
                    object(BIOTOPE, None, None),
                ),
                (
                    "punkte",
                    "wfs_linfos:gbt_point",
                    object(BIOTOPE, None, None),
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
