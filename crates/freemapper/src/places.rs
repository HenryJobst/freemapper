//! Place search: find the map area by name or OSM id (Nominatim; spec §4.1 allows it for a single
//! element). Data © OpenStreetMap contributors, ODbL.

use serde_json::Value;

use crate::area::{AreaError, MapArea};
use crate::fetch::{FetchError, Fetcher};

const NOMINATIM: &str = "https://nominatim.openstreetmap.org";
/// Results per search.
const SEARCH_LIMIT: u32 = 5;

/// One match of a place search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceCandidate {
    /// `way/<id>`, `relation/<id>` or `node/<id>`.
    pub osm_id: String,
    pub name: String,
    pub display_name: String,
    /// OSM key and value, e.g. `amenity` / `school`.
    pub category: String,
    pub kind: String,
    /// Whether the place has a polygon and can be a map area.
    pub has_area: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum PlaceError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error(transparent)]
    Area(#[from] AreaError),
}

/// Percent-encodes a query string value (RFC 3986 unreserved characters stay).
fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub fn search_url(query: &str) -> String {
    format!(
        "{NOMINATIM}/search?q={}&format=jsonv2&limit={SEARCH_LIMIT}&polygon_geojson=1",
        encode(query)
    )
}

/// Lookup URL for an id like `way/992423119`.
pub fn lookup_url(osm_id: &str) -> Result<String, AreaError> {
    let invalid = || AreaError::Place(format!("{osm_id:?} is not an OSM id like way/992423119"));
    let (kind, id) = osm_id.split_once('/').ok_or_else(invalid)?;
    let prefix = match kind {
        "way" => 'W',
        "relation" => 'R',
        "node" => 'N',
        _ => return Err(invalid()),
    };
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid());
    }
    Ok(format!(
        "{NOMINATIM}/lookup?osm_ids={prefix}{id}&format=jsonv2&polygon_geojson=1"
    ))
}

fn results(json: &str) -> Result<Vec<Value>, AreaError> {
    serde_json::from_str::<Vec<Value>>(json)
        .map_err(|e| AreaError::Place(format!("invalid answer from the place search: {e}")))
}

/// Whether the result has a polygon a map area can be made from (one part only).
fn has_polygon(result: &Value) -> bool {
    match result["geojson"]["type"].as_str() {
        Some("Polygon") => true,
        Some("MultiPolygon") => result["geojson"]["coordinates"]
            .as_array()
            .is_some_and(|parts| parts.len() == 1),
        _ => false,
    }
}

/// Search results, places with an area first, otherwise in Nominatim's order.
pub fn parse_search(json: &str) -> Result<Vec<PlaceCandidate>, AreaError> {
    let mut places: Vec<PlaceCandidate> = results(json)?
        .iter()
        .filter_map(|r| {
            Some(PlaceCandidate {
                osm_id: format!("{}/{}", r["osm_type"].as_str()?, r["osm_id"].as_u64()?),
                name: r["name"].as_str().unwrap_or_default().to_string(),
                display_name: r["display_name"].as_str().unwrap_or_default().to_string(),
                category: r["category"].as_str().unwrap_or_default().to_string(),
                kind: r["type"].as_str().unwrap_or_default().to_string(),
                has_area: has_polygon(r),
            })
        })
        .collect();
    places.sort_by_key(|p| !p.has_area);
    Ok(places)
}

/// The map area of a lookup answer for `osm_id`. An area of several separate parts (a
/// multipolygon with several outer rings) needs `part`: 1 is the largest. A single polygon is
/// part 1.
pub fn area_from_lookup(
    json: &str,
    osm_id: &str,
    part: Option<usize>,
) -> Result<MapArea, AreaError> {
    let found = results(json)?;
    let place = found.first().ok_or_else(|| {
        AreaError::Place(format!(
            "{osm_id} not found. Nominatim lists only tagged OSM objects, so an untagged outer way \
             of a multipolygon is not there. Use the multipolygon relation instead (--osm \
             relation/<id>), or save the outline as GeoJSON and use --geojson"
        ))
    })?;
    let geojson = &place["geojson"];
    if !matches!(geojson["type"].as_str(), Some("Polygon" | "MultiPolygon")) {
        return Err(AreaError::Place(format!(
            "{osm_id} is a point, not an area; pick its grounds (a way or relation)"
        )));
    }
    let mut parts = match geojson["type"].as_str() {
        Some("MultiPolygon") => geojson["coordinates"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|rings| {
                let polygon = serde_json::json!({"type": "Polygon", "coordinates": rings});
                MapArea::from_geojson(&polygon.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => vec![MapArea::from_geojson(&geojson.to_string())?],
    };
    let size = |a: &MapArea| {
        use geo::Area as _;
        a.projected_polygon().unsigned_area()
    };
    parts.sort_by(|a, b| size(b).total_cmp(&size(a)));
    let count = parts.len();
    let index = match part {
        Some(n) if (1..=count).contains(&n) => n - 1,
        Some(n) => {
            return Err(AreaError::Place(format!(
                "{osm_id} has {count} part{}; there is no part {n}",
                if count == 1 { "" } else { "s" }
            )));
        }
        None if count == 1 => 0,
        None => {
            let sizes: Vec<String> = parts.iter().map(|p| square_metres(size(p))).collect();
            return Err(AreaError::Place(format!(
                "{osm_id} is an area of {count} separate parts ({}). A map area is one polygon: \
                 pick a part with --part <n> (1 is the largest), or save the outline you want as \
                 GeoJSON and use --geojson",
                sizes.join(", ")
            )));
        }
    };
    let mut area = parts.swap_remove(index);
    area.source_id = Some(osm_id.to_string());
    Ok(area)
}

/// `7900 m²`: an area rounded to 100 m² from 1000 m², else to 10 m².
fn square_metres(area: f64) -> String {
    let step = if area >= 1000.0 { 100.0 } else { 10.0 };
    format!("{} m²", (area / step).round() * step)
}

pub fn find_places(fetcher: &dyn Fetcher, query: &str) -> Result<Vec<PlaceCandidate>, PlaceError> {
    let bytes = fetcher.get(&search_url(query))?;
    Ok(parse_search(&String::from_utf8_lossy(&bytes))?)
}

pub fn place_area(
    fetcher: &dyn Fetcher,
    osm_id: &str,
    part: Option<usize>,
) -> Result<MapArea, PlaceError> {
    let bytes = fetcher.get(&lookup_url(osm_id)?)?;
    Ok(area_from_lookup(
        &String::from_utf8_lossy(&bytes),
        osm_id,
        part,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    const SEARCH: &str = include_str!("../../../tests/fixtures/nominatim-search.json");
    const LOOKUP: &str = include_str!("../../../tests/fixtures/nominatim-lookup.json");

    #[test]
    fn multi_part_multipolygon_has_no_area() {
        let json = r#"[
            {"osm_type":"relation","osm_id":1,"name":"Split","geojson":{"type":"MultiPolygon",
             "coordinates":[[[[0,0],[1,0],[1,1],[0,0]]],[[[2,2],[3,2],[3,3],[2,2]]]]}},
            {"osm_type":"relation","osm_id":2,"name":"One","geojson":{"type":"MultiPolygon",
             "coordinates":[[[[0,0],[1,0],[1,1],[0,0]]]]}}
        ]"#;
        let places = parse_search(json).unwrap();
        assert_eq!(places[0].osm_id, "relation/2");
        assert!(places[0].has_area);
        assert!(!places[1].has_area);
    }

    #[test]
    fn invalid_answer_is_named() {
        let err = parse_search("<html>busy</html>").unwrap_err();
        assert!(
            err.to_string()
                .contains("invalid answer from the place search"),
            "{err}"
        );
    }

    #[test]
    fn search_url_encodes_query() {
        let url = search_url("Clara-Zetkin-Schule Strausberg");
        assert!(
            url.starts_with("https://nominatim.openstreetmap.org/search?"),
            "{url}"
        );
        assert!(url.contains("q=Clara-Zetkin-Schule%20Strausberg"), "{url}");
        assert!(
            url.contains("format=jsonv2")
                && url.contains("polygon_geojson=1")
                && url.contains("limit=5")
        );
        assert!(search_url("Straße & Platz").contains("q=Stra%C3%9Fe%20%26%20Platz"));
    }

    #[test]
    fn parse_search_fixture() {
        let places = parse_search(SEARCH).unwrap();
        assert_eq!(places.len(), 1);
        let school = &places[0];
        assert_eq!(school.osm_id, "way/992423119");
        assert_eq!(school.name, "Clara-Zetkin-Schule");
        assert_eq!(
            (school.category.as_str(), school.kind.as_str()),
            ("amenity", "school")
        );
        assert!(school.has_area);
        assert!(school.display_name.contains("Strausberg"));
    }

    #[test]
    fn areas_come_first_and_empty_is_fine() {
        let json = r#"[
            {"osm_type":"node","osm_id":1,"name":"Bus stop","display_name":"Bus stop, Strausberg","category":"highway","type":"bus_stop","geojson":{"type":"Point","coordinates":[13.8,52.5]}},
            {"osm_type":"way","osm_id":2,"name":"Schule","display_name":"Schule, Strausberg","category":"amenity","type":"school","geojson":{"type":"Polygon","coordinates":[[[13.8,52.5],[13.81,52.5],[13.81,52.51],[13.8,52.5]]]}}
        ]"#;
        let ids: Vec<String> = parse_search(json)
            .unwrap()
            .into_iter()
            .map(|p| p.osm_id)
            .collect();
        assert_eq!(ids, ["way/2", "node/1"]);
        assert!(parse_search("[]").unwrap().is_empty());
        assert!(parse_search("<html>").is_err());
    }

    #[test]
    fn lookup_url_for_osm_ids() {
        assert!(
            lookup_url("way/992423119")
                .unwrap()
                .contains("osm_ids=W992423119")
        );
        assert!(lookup_url("relation/5").unwrap().contains("osm_ids=R5"));
        assert!(lookup_url("node/7").unwrap().contains("osm_ids=N7"));
        for bad in ["992423119", "way/abc", "street/1"] {
            assert!(lookup_url(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn area_from_lookup_fixture() {
        let area = area_from_lookup(LOOKUP, "way/992423119", None).unwrap();
        assert_eq!(area.source_id.as_deref(), Some("way/992423119"));
        assert_eq!(area.polygon.exterior().0.len(), 9);
        let c = area.centroid();
        assert!(
            (c.lat - 52.5401).abs() < 0.001 && (c.lon - 13.8442).abs() < 0.001,
            "{c:?}"
        );
    }

    #[test]
    fn point_is_not_an_area() {
        let json = r#"[{"osm_type":"node","osm_id":7,"name":"Hydrant","geojson":{"type":"Point","coordinates":[13.8,52.5]}}]"#;
        let err = area_from_lookup(json, "node/7", None).unwrap_err();
        assert!(err.to_string().contains("is a point, not an area"), "{err}");
        assert!(
            area_from_lookup("[]", "way/1", None)
                .unwrap_err()
                .to_string()
                .contains("not found")
        );
    }

    #[test]
    fn place_area_and_find_places_use_the_fetcher() {
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &lookup_url("way/992423119").unwrap(),
            LOOKUP.as_bytes().to_vec(),
        );
        fetcher.insert(
            &search_url("Clara-Zetkin-Schule Strausberg"),
            SEARCH.as_bytes().to_vec(),
        );
        assert_eq!(
            place_area(&fetcher, "way/992423119", None)
                .unwrap()
                .polygon
                .exterior()
                .0
                .len(),
            9
        );
        assert_eq!(
            find_places(&fetcher, "Clara-Zetkin-Schule Strausberg")
                .unwrap()
                .len(),
            1
        );
    }

    const MULTIPART: &str = include_str!("../../../tests/fixtures/nominatim-lookup-multipart.json");

    #[test]
    fn multi_part_lookup_names_the_parts_and_the_way_out() {
        let err = area_from_lookup(MULTIPART, "relation/16446395", None).unwrap_err();
        let text = err.to_string();
        for needle in [
            "relation/16446395 is an area of 2 separate parts",
            "7900 m²",
            "1400 m²",
            "--part <n>",
            "--geojson",
        ] {
            assert!(text.contains(needle), "{needle:?} in {text}");
        }
    }

    #[test]
    fn a_part_of_a_multi_part_area_can_be_picked() {
        use geo::Area as _;
        let size = |part| {
            area_from_lookup(MULTIPART, "relation/16446395", Some(part))
                .unwrap()
                .projected_polygon()
                .unsigned_area()
        };
        // Part 1 is the largest, though the lookup lists it second.
        assert!((size(1) - 7900.0).abs() < 100.0, "{}", size(1));
        assert!((size(2) - 1400.0).abs() < 100.0, "{}", size(2));
        let area = area_from_lookup(MULTIPART, "relation/16446395", Some(1)).unwrap();
        assert_eq!(area.source_id.as_deref(), Some("relation/16446395"));
        let err = area_from_lookup(MULTIPART, "relation/16446395", Some(3)).unwrap_err();
        assert!(
            err.to_string()
                .contains("relation/16446395 has 2 parts; there is no part 3"),
            "{err}"
        );
        // A single polygon is part 1.
        assert!(area_from_lookup(LOOKUP, "way/992423119", Some(1)).is_ok());
        assert!(area_from_lookup(LOOKUP, "way/992423119", Some(2)).is_err());
    }

    #[test]
    fn missing_way_suggests_the_relation_or_geojson() {
        let text = area_from_lookup("[]", "way/48378476", None)
            .unwrap_err()
            .to_string();
        for needle in [
            "way/48378476 not found",
            "untagged",
            "relation",
            "--geojson",
        ] {
            assert!(text.contains(needle), "{needle:?} in {text}");
        }
    }

    #[test]
    #[ignore = "network"]
    fn live_find_the_school() {
        let fetcher = crate::fetch::HttpFetcher::new();
        let places = find_places(&fetcher, "Clara-Zetkin-Schule Strausberg").unwrap();
        assert_eq!(
            places.first().map(|p| p.osm_id.as_str()),
            Some("way/992423119")
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
        let area = place_area(&fetcher, "way/992423119", None).unwrap();
        assert!(area.polygon.exterior().0.len() >= 4);
    }
}
