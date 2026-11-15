//! The permit source of Saxony (ADR 0139): two WMS of the Staatsbetrieb Sachsenforst, asked at
//! points through [`gfi_permits`](crate::gfi_permits). "Wald nach SächsWaldG" gives the Revier
//! and the Forstbezirk at a point, "Waldfunktionen in Sachsen" the functions of the forest there:
//! Erholungswald, Naturwaldzellen, Bodenschutzwald and the drinking water protection zones (in
//! the forest only). Both are dl-de/by-2-0; the credit is the one the capabilities of the
//! functions name.
//!
//! The Reviere cover all of Saxony. A Revier is in charge in the state forest; for other forest
//! the lower forest authority of the Landkreis or the kreisfreie Stadt is, as the source says.

use freemapper_adapter_api::permits::{
    FindingKind, ForestFunctionKind, PermitSection, PermitSource, Wording,
};
use serde_json::Value;

use crate::gfi_permits::{Query, source};
use crate::wfs_permits::{text, zone_name};

const CREDIT: &str = "Datenlizenz Deutschland 2.0 – Namensnennung (dl-de/by-2-0) unter Angabe der Quelle: \"Geodaten Sachsenforst\"";

const ADMINISTRATION: &str = "Staatsbetrieb Sachsenforst";

/// The layers of "Waldfunktionen in Sachsen" the notes read, by their numbers in the WMS:
/// Erholungswald (7), Naturwaldzelle (46), Bodenschutzwald nach SächsWaldG (101) and the zones of
/// the drinking water protection areas of reservoirs (80 to 84), streams (86 to 88) and ground
/// water (90 to 94).
const FUNCTION_LAYERS: &str = "7,46,101,80,81,82,83,84,86,87,88,90,91,92,93,94";

const QUERIES: &[Query] = &[
    Query {
        key: "reviere",
        base: "https://www.forsten.sachsen.de/kartendienste/wald/MapServer/WMSServer",
        layers: "Reviere",
        kept: &["REV_NAME", "FBZ_NAME"],
        one: true,
        read: revier,
    },
    Query {
        key: "waldfunktionen",
        base: "https://www.forsten.sachsen.de/kartendienste/waldfunktionen/MapServer/WMSServer",
        layers: FUNCTION_LAYERS,
        kept: &["WFZ", "GNM", "KNM"],
        one: false,
        read: function,
    },
];

/// The source of Saxony.
pub fn sources() -> Vec<Box<dyn PermitSource>> {
    vec![source(
        "sachsenforst",
        Wording {
            en: "Staatsbetrieb Sachsenforst",
            de: "Staatsbetrieb Sachsenforst",
        },
        Wording {
            en: "Forstbezirke and Reviere (in charge in the state forest; for other forest the \
                 lower forest authority of the Landkreis), Erholungswald, Naturwaldzellen, \
                 Bodenschutzwald, drinking water protection zones in the forest",
            de: "Forstbezirke und Reviere (zuständig im Staatswald; für anderen Wald die untere \
                 Forstbehörde des Landkreises), Erholungswald, Naturwaldzellen, Bodenschutzwald, \
                 Trinkwasserschutzzonen im Wald",
        },
        CREDIT,
        &["reviere", "waldfunktionen"],
        &[
            PermitSection::ForestUnits,
            PermitSection::ForestFunctions,
            PermitSection::WaterProtection,
        ],
        QUERIES,
    )]
}

/// The Forstbezirk and the Revier at a point.
fn revier(feature: &Value) -> Option<FindingKind> {
    Some(FindingKind::ForestUnit {
        administration: ADMINISTRATION.to_string(),
        office: office(&text(feature, "FBZ_NAME")?),
        unit: text(feature, "REV_NAME").map(|r| format!("Revier {r}")),
    })
}

/// The office of a Forstbezirk as the notes name it. The Reviere write its name in at most 40
/// characters, which cuts the one office that is no place, the national park's.
fn office(name: &str) -> String {
    if name.starts_with("Nationalpark") {
        "Nationalpark- und Forstverwaltung Sächsische Schweiz".to_string()
    } else {
        format!("Forstbezirk {name}")
    }
}

/// A function of the forest at a point, by its key `WFZ`: 7110 Erholungswald, 4151
/// Naturwaldzelle, 1110 Bodenschutzwald, 211.. a zone of a drinking water protection area,
/// whose short name `KNM` ends in the zone ("§ TW GW Z III", "§ TW TS Z II A").
fn function(feature: &Value) -> Option<FindingKind> {
    let key = text(feature, "WFZ")?;
    let name = text(feature, "GNM");
    let forest = |kind, name| Some(FindingKind::ForestFunction { kind, name });
    match key.as_str() {
        "7110" => forest(ForestFunctionKind::Recreation, name),
        "4151" => forest(ForestFunctionKind::Natural, name),
        "1110" => forest(
            ForestFunctionKind::Protected,
            Some("Bodenschutzwald".into()),
        ),
        k if k.len() == 5 && k.starts_with("211") => Some(FindingKind::WaterProtection {
            name,
            zone: text(feature, "KNM")
                .and_then(|n| n.split_once(" Z ").map(|(_, zone)| zone_name(zone.trim()))),
            citation: None,
            authority: None,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(layer: &str, collection: &str) -> Vec<FindingKind> {
        // A stored answer: the trimmed live features with a cell each, as the fetch writes them.
        let value: Value = serde_json::from_str(collection).unwrap();
        let cell = serde_json::json!({"type": "Polygon", "coordinates": [[
            [13.8, 51.05], [13.81, 51.05], [13.81, 51.06], [13.8, 51.06], [13.8, 51.05]]]});
        let features: Vec<Value> = value["features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| serde_json::json!({"type": "Feature", "geometry": cell, "properties": f["properties"]}))
            .collect();
        let sources = sources();
        sources[0]
            .findings(layer, &features)
            .into_iter()
            .map(|f| f.kind)
            .collect()
    }

    #[test]
    fn a_revier_names_its_forstbezirk() {
        assert_eq!(
            kinds(
                "reviere",
                include_str!("../../../../tests/fixtures/permits-states/sn-reviere.geojson")
            ),
            [
                FindingKind::ForestUnit {
                    administration: "Staatsbetrieb Sachsenforst".into(),
                    office: "Forstbezirk Dresden".into(),
                    unit: Some("Revier Bühlau".into()),
                },
                // The Reviere cut the name of the national park's office at 40 characters.
                FindingKind::ForestUnit {
                    administration: "Staatsbetrieb Sachsenforst".into(),
                    office: "Nationalpark- und Forstverwaltung Sächsische Schweiz".into(),
                    unit: Some("Revier Hohnstein".into()),
                },
            ]
        );
    }

    #[test]
    fn the_functions_read_their_key_and_the_zone() {
        assert_eq!(
            kinds(
                "waldfunktionen",
                include_str!("../../../../tests/fixtures/permits-states/sn-waldfunktionen.geojson")
            ),
            [
                FindingKind::ForestFunction {
                    kind: ForestFunctionKind::Protected,
                    name: Some("Bodenschutzwald".into()),
                },
                FindingKind::ForestFunction {
                    kind: ForestFunctionKind::Natural,
                    name: Some("Wodrich".into()),
                },
                FindingKind::WaterProtection {
                    name: Some("Arnsdorf-Wasserwerk Karswald".into()),
                    zone: Some("Schutzzone III".into()),
                    citation: None,
                    authority: None,
                },
                FindingKind::WaterProtection {
                    name: Some("Talsperre Eibenstock".into()),
                    zone: Some("Schutzzone II A".into()),
                    citation: None,
                    authority: None,
                },
                FindingKind::WaterProtection {
                    name: Some("Speichersystem Radeburg (WW Rödern)".into()),
                    zone: Some("Schutzzone I".into()),
                    citation: None,
                    authority: None,
                },
            ]
        );
    }
}
