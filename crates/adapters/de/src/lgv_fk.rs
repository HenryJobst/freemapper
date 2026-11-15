//! The Feinkartierung of Hamburg (ADR 0149): the surveyed street space (`feinkartierung_strasse`,
//! curbs, railings, walls, bollards, benches, lamps, stair areas) and public green
//! (`feinkartierung_gruen`, fences, handrails, walls, stairs, benches) of the LGV, as a set of
//! extra layers that brings its own rules. The engine draws it without knowing Hamburg.
//!
//! Each of the six collections is asked once in the region box, in EPSG:25832, through all of its
//! pages; its features are sorted into the layers by [`layer_of`]. A street area is kept only when
//! it is a stair (the surfaces come later), a green area only when it is a stair or a wall.

use freemapper_adapter_api::alkis::FetchArea;
use freemapper_adapter_api::error::FetchError;
use freemapper_adapter_api::extras::{
    CONTROL_PROPERTY, ExtraEstimate, ExtraLayerInfo, ExtraNotes, ExtraSet, ExtraSetInfo,
    FetchedExtra, SetDrawing, SetRank,
};
use freemapper_adapter_api::fetch::Fetcher;
use freemapper_contract::MapKind;
use freemapper_sources::ogc_api::{collection_features_in, invalid};
use serde_json::Value;

use crate::crs::{BoundingBox, Crs};

/// The datasets of the Feinkartierung on the OGC API of the LGV.
const BASE: &str = "https://api.hamburg.de/datasets/v1";

/// The collections, each asked once: dataset and collection.
pub const COLLECTIONS: [(&str, &str); 6] = [
    ("feinkartierung_strasse", "linien"),
    ("feinkartierung_strasse", "punkte"),
    ("feinkartierung_strasse", "strassenflaechen"),
    ("feinkartierung_gruen", "linienobjekte"),
    ("feinkartierung_gruen", "punktobjekte"),
    ("feinkartierung_gruen", "flaechenobjekte"),
];

/// The layers, in the order they are fetched and read.
pub const LAYERS: [&str; 7] = [
    "fk-strasse-linien",
    "fk-strasse-punkte",
    "fk-strasse-gruen",
    "fk-gruen-linien",
    "fk-gruen-punkte",
    "fk-gruen-flaechen",
    "fk-treppen",
];

/// The layer of the stair areas of both surveys.
pub const STAIR_LAYER: &str = "fk-treppen";

/// The street areas kept as green of the street space (verges, beds, shrubs): candidates on every
/// map, which the training map takes (the user's decision of 2026-11-15). The paving materials
/// stay out; the street is a paved area on the map already.
pub const STREET_GREEN: [&str; 5] = [
    "Grasfläche, Wiese",
    "Sträucher und Bodendecker bis 1m Höhe",
    "Sträucher ab 1m Höhe",
    "Hecke",
    "Sukzession, Wildwuchs",
];

/// The endpoint of the items of `collection` in `dataset`.
pub fn items_url(dataset: &str, collection: &str) -> String {
    format!("{BASE}/{dataset}/collections/{collection}/items")
}

fn text<'a>(feature: &'a Value, key: &str) -> &'a str {
    feature["properties"][key].as_str().unwrap_or_default()
}

/// The layer a feature of `collection` goes to; `None`: it is left out.
pub fn layer_of(collection: &str, feature: &Value) -> Option<&'static str> {
    Some(match collection {
        "linien" => "fk-strasse-linien",
        "punkte" => "fk-strasse-punkte",
        "strassenflaechen" if text(feature, "nutzung") == "Treppe" => STAIR_LAYER,
        "strassenflaechen" if STREET_GREEN.contains(&text(feature, "inhalt")) => "fk-strasse-gruen",
        "linienobjekte" => "fk-gruen-linien",
        "punktobjekte" => "fk-gruen-punkte",
        "flaechenobjekte" => match text(feature, "kategorie_1") {
            "Treppen" => STAIR_LAYER,
            "Mauern, Wände" => "fk-gruen-flaechen",
            _ => return None,
        },
        _ => return None,
    })
}

/// The control site a point stands for (ADR 0074), as the Berlin survey names them.
pub fn control_of(collection: &str, feature: &Value) -> Option<&'static str> {
    match collection {
        "punkte" => match text(feature, "inhalt") {
            "Poller, Pfosten, Wegesperre" => Some("bollard"),
            "Bank" => Some("bench"),
            "Lampe" => Some("street_lamp"),
            _ => None,
        },
        "punktobjekte" => match text(feature, "kategorie_3") {
            "Poller" => Some("bollard"),
            "Bank" | "Tisch-Bank-Kombination" => Some("bench"),
            "Mastleuchten" => Some("street_lamp"),
            _ => None,
        },
        _ => None,
    }
}

/// The Feinkartierung as the set of extra layers of the Hamburg adapter.
pub struct Feinkartierung;

static INFO: ExtraSetInfo = ExtraSetInfo {
    key: "lgv-feinkartierung",
    crs: Crs::Utm32,
    notes: ExtraNotes {
        none_stored: "progress-feinkartierung-none-stored",
        removing: "progress-feinkartierung-removing",
        download: "progress-feinkartierung-download",
        features: "progress-feinkartierung-features",
        kept: "warning-feinkartierung-layer-kept",
        dropped: "warning-feinkartierung-layer-skipped",
    },
    // Blankenese (live run of 2026-11-15, 1.0 by 0.7 km): 3.5 MB for the six collections, most
    // of it the street areas, which are asked whole and kept only where they are stairs.
    estimate: ExtraEstimate {
        what: "Hamburg Feinkartierung (street space and public green)",
        megabytes_per_km2: 6.0,
    },
};

static DRAWING: SetDrawing = SetDrawing {
    rules: include_str!("../rules/lgv-feinkartierung.toml"),
    tag_prefix: "hamburg",
    source_prefix: "lgv-feinkartierung",
    rank: SetRank::Survey,
    layer_ranks: &[],
    credited_layers: &[],
    point_dedups: &[],
    kept_tags: &[],
    control_kinds: &[],
    fountain_tags: &[],
    winning_areas: &[],
    stair_layers: &[STAIR_LAYER],
    // The green of the street space: candidates on the base map (the user's decision of
    // 2026-11-15).
    candidate_layers: &["fk-strasse-gruen"],
    generic: true,
};

/// The map kinds that take the set: the street space is for sprint and schoolyard maps.
fn takes(kind: MapKind) -> bool {
    matches!(kind, MapKind::Sprint | MapKind::Schoolyard)
}

impl ExtraSet for Feinkartierung {
    fn info(&self) -> &ExtraSetInfo {
        &INFO
    }

    fn layers(&self, kind: MapKind) -> Vec<ExtraLayerInfo> {
        if !takes(kind) {
            return Vec::new();
        }
        LAYERS.iter().map(|key| ExtraLayerInfo { key }).collect()
    }

    fn has(&self, key: &str) -> bool {
        LAYERS.contains(&key)
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        region_bbox: &BoundingBox,
        kind: MapKind,
    ) -> Vec<FetchedExtra> {
        if !takes(kind) {
            return Vec::new();
        }
        let bbox = FetchArea::new(*region_bbox, Crs::Utm32).bbox;
        let mut layers: Vec<(&'static str, Result<Vec<Value>, _>)> =
            LAYERS.iter().map(|key| (*key, Ok(Vec::new()))).collect();
        for (dataset, collection) in COLLECTIONS {
            let answer =
                collection_features_in(fetcher, &items_url(dataset, collection), &bbox, Crs::Utm32);
            let features = match answer {
                Ok(features) => features,
                Err(error) => {
                    // The layers this collection feeds fail; the others go on.
                    for (key, layer) in &mut layers {
                        if fed_by(collection).contains(key) {
                            *layer =
                                Err(FetchError::Alkis(invalid(format!("{collection}: {error}"))));
                        }
                    }
                    continue;
                }
            };
            for mut feature in features {
                let Some(key) = layer_of(collection, &feature) else {
                    continue;
                };
                // The API numbers the features of each collection; the engine reads text ids,
                // and the stair layer holds two collections.
                let id = match &feature["id"] {
                    Value::Number(n) => Some(n.to_string()),
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                };
                if let Some(id) = id {
                    feature["id"] = Value::from(format!("{collection}.{id}"));
                }
                if let Some(site) = control_of(collection, &feature) {
                    feature["properties"][CONTROL_PROPERTY] = Value::from(site);
                }
                if let Some((_, Ok(layer))) = layers.iter_mut().find(|(k, _)| *k == key) {
                    layer.push(feature);
                }
            }
        }
        layers
            .into_iter()
            .map(|(key, features)| FetchedExtra { key, features })
            .collect()
    }

    fn drawing(&self) -> Option<&SetDrawing> {
        Some(&DRAWING)
    }
}

/// The layers the features of `collection` go to.
fn fed_by(collection: &str) -> &'static [&'static str] {
    match collection {
        "linien" => &["fk-strasse-linien"],
        "punkte" => &["fk-strasse-punkte"],
        "strassenflaechen" => &["fk-strasse-gruen", STAIR_LAYER],
        "linienobjekte" => &["fk-gruen-linien"],
        "punktobjekte" => &["fk-gruen-punkte"],
        "flaechenobjekte" => &["fk-gruen-flaechen", STAIR_LAYER],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_adapter_api::testing::MapFetcher;
    use freemapper_sources::ogc_api::collection_url_in;
    use serde_json::json;

    fn feature(properties: Value) -> Value {
        json!({"type": "Feature", "geometry": {"type": "Point", "coordinates": [552_900.0, 5_934_900.0]}, "properties": properties})
    }

    #[test]
    fn the_features_go_to_their_layers() {
        let stair = feature(json!({"nutzung": "Treppe"}));
        let walk = feature(json!({"nutzung": "Gehweg"}));
        assert_eq!(layer_of("strassenflaechen", &stair), Some(STAIR_LAYER));
        assert_eq!(layer_of("strassenflaechen", &walk), None);
        // The green of the street space; the paving materials stay out.
        let surface = |inhalt: &str| feature(json!({"nutzung": "Grünfläche", "inhalt": inhalt}));
        assert_eq!(
            layer_of("strassenflaechen", &surface("Grasfläche, Wiese")),
            Some("fk-strasse-gruen")
        );
        assert_eq!(
            layer_of("strassenflaechen", &surface("Sträucher ab 1m Höhe")),
            Some("fk-strasse-gruen")
        );
        assert_eq!(
            layer_of("strassenflaechen", &surface("Natursteinpflaster")),
            None
        );
        assert_eq!(
            layer_of("strassenflaechen", &surface("wassergebunden (Grandfläche)")),
            None
        );
        let green = |kategorie: &str| feature(json!({"kategorie_1": kategorie}));
        assert_eq!(
            layer_of("flaechenobjekte", &green("Treppen")),
            Some(STAIR_LAYER)
        );
        assert_eq!(
            layer_of("flaechenobjekte", &green("Mauern, Wände")),
            Some("fk-gruen-flaechen")
        );
        assert_eq!(layer_of("flaechenobjekte", &green("Wege")), None);
        assert_eq!(layer_of("linien", &walk), Some("fk-strasse-linien"));
        // Every layer is fed by some collection, and every fed layer is one of the set.
        for key in LAYERS {
            assert!(
                COLLECTIONS.iter().any(|(_, c)| fed_by(c).contains(&key)),
                "{key}"
            );
        }
    }

    #[test]
    fn points_name_the_control_site_they_stand_for() {
        let street = |inhalt: &str| feature(json!({"inhalt": inhalt}));
        assert_eq!(
            control_of("punkte", &street("Poller, Pfosten, Wegesperre")),
            Some("bollard")
        );
        assert_eq!(control_of("punkte", &street("Bank")), Some("bench"));
        assert_eq!(control_of("punkte", &street("Lampe")), Some("street_lamp"));
        assert_eq!(control_of("punkte", &street("Verkehrsampel")), None);
        let green = |k3: &str| feature(json!({"kategorie_3": k3}));
        assert_eq!(
            control_of("punktobjekte", &green("Findling / Bruchsteine")),
            None
        );
        assert_eq!(
            control_of("punktobjekte", &green("Mastleuchten")),
            Some("street_lamp")
        );
    }

    #[test]
    fn a_fetch_asks_each_collection_once_and_sorts_its_features() {
        // Blankenese in EPSG:25833 (the region box).
        let region = BoundingBox {
            min_e: 552_737.0 - 400_000.0,
            min_n: 5_934_347.0,
            max_e: 553_646.0 - 400_000.0,
            max_n: 5_935_057.0,
        };
        let bbox = FetchArea::new(region, Crs::Utm32).bbox;
        let fetcher = MapFetcher::default();
        let page = |features: Vec<Value>| {
            json!({"type": "FeatureCollection", "features": features, "links": []})
                .to_string()
                .into_bytes()
        };
        let serve = |dataset: &str, collection: &str, features: Vec<Value>| {
            fetcher.insert(
                &collection_url_in(&items_url(dataset, collection), &bbox, Crs::Utm32),
                page(features),
            );
        };
        serve(
            "feinkartierung_strasse",
            "linien",
            vec![{
                let mut railing = feature(json!({"inhalt": "Geländer"}));
                railing["id"] = json!(7);
                railing
            }],
        );
        serve(
            "feinkartierung_strasse",
            "punkte",
            vec![
                feature(json!({"inhalt": "Bank"})),
                feature(json!({"inhalt": "Verkehrsampel"})),
            ],
        );
        serve(
            "feinkartierung_strasse",
            "strassenflaechen",
            vec![
                feature(json!({"nutzung": "Treppe"})),
                feature(json!({"nutzung": "Gehweg"})),
            ],
        );
        serve("feinkartierung_gruen", "linienobjekte", vec![]);
        serve("feinkartierung_gruen", "punktobjekte", vec![]);
        // The green areas fail: their two layers fail, the others are fetched.
        let fetched = Feinkartierung.fetch(&fetcher, &region, MapKind::Sprint);
        let counts: Vec<(&str, Option<usize>)> = fetched
            .iter()
            .map(|f| (f.key, f.features.as_ref().ok().map(Vec::len)))
            .collect();
        assert_eq!(
            counts,
            [
                ("fk-strasse-linien", Some(1)),
                ("fk-strasse-punkte", Some(2)),
                ("fk-strasse-gruen", Some(0)),
                ("fk-gruen-linien", Some(0)),
                ("fk-gruen-punkte", Some(0)),
                ("fk-gruen-flaechen", None),
                ("fk-treppen", None),
            ]
        );
        let bench = &fetched[1].features.as_ref().unwrap()[0];
        // The numeric ids of the API become text, named by their collection.
        let railing = &fetched[0].features.as_ref().unwrap()[0];
        assert_eq!(railing["id"], "linien.7");
        assert_eq!(bench["properties"][CONTROL_PROPERTY], "bench");
        assert_eq!(fetcher.calls.get(), COLLECTIONS.len());
        // A forest map takes none of it.
        assert!(
            Feinkartierung
                .fetch(&fetcher, &region, MapKind::Forest)
                .is_empty()
        );
    }
}
