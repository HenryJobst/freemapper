// SPDX-License-Identifier: AGPL-3.0-or-later

//! City tree cadastres (single trees with a species, a survey date and sometimes a crown
//! diameter) for the 417/418 candidates.
//!
//! One row per city (the adapter of its state names it): where the points come from ([`TreeFormat`], the
//! endpoints), the field names, the kinds that are no tree (stumps), the licence line and the
//! envelope that decides whether a map area needs the dataset. Potsdam is the first row
//! (`potsdam`, an OpenDataSoft GeoJSON export); the WFS, OGC API and CSV formats of the other
//! cities (Leipzig, Dresden, Hamburg, Frankfurt, Kiel, Lübeck, Rostock, Köln, Dortmund, Münster)
//! come with the row that first needs them. Every format normalises to the same stored feature:
//! a point in the map's CRS with the `tree_*` properties of [`parse_trees`].

#[cfg(any(test, feature = "testing"))]
use std::borrow::Cow;
use std::collections::HashSet;

use chrono::NaiveDate;
use serde_json::{Map, Value, json};

#[cfg(any(test, feature = "testing"))]
use crate::crs::BoundingBox;
use crate::crs::{Crs, LatLon, ProjectedPoint};
use crate::fetch::{FetchError, Fetcher, get_limited_with_retries};
use crate::nas::PartFile;
use crate::ogc_api::{AlkisError, FetchArea, invalid};
pub use freemapper_adapter_api::extras::{TreeDataset, TreeFields, TreeFormat};
use freemapper_adapter_api::records::AdapterId;

/// 256 MB, as for the NAS answers: the whole Potsdam envelope (283 772 points) is about 110 MB.
#[cfg(any(test, feature = "testing"))]
const MAX_ANSWER_BYTES: u64 = 256 << 20;

#[cfg(any(test, feature = "testing"))]
/// Potsdam, "Bäume der Stadtkarte" (`opendata.potsdam.de`, dl-de/zero-2-0): 283 772 points in
/// three datasets, no crown diameter. The envelope is the data extent (E 359 000–375 000,
/// N 5 801 000–5 820 000, EPSG:25833) rounded outward.
pub const POTSDAM: TreeDataset = TreeDataset {
    id: Cow::Borrowed("potsdam"),
    name: Cow::Borrowed("Potsdam tree cadastre"),
    region: AdapterId::built_in("brandenburg"),
    crs: Crs::Utm33,
    envelope: BoundingBox {
        min_e: 359_000.0,
        min_n: 5_801_000.0,
        max_e: 375_000.0,
        max_n: 5_820_000.0,
    },
    format: TreeFormat::OdsGeoJsonExport {
        geo_field: Cow::Borrowed("geo_point_2d"),
    },
    endpoints: Cow::Borrowed(&[
        Cow::Borrowed(
            "https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/baeume-1-stadtkarte-potsdam/exports/geojson",
        ),
        Cow::Borrowed(
            "https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/baeume-2-stadtkarte-potsdam/exports/geojson",
        ),
        Cow::Borrowed(
            "https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/baeume-3-stadtkarte-potsdam/exports/geojson",
        ),
    ]),
    fields: TreeFields {
        kind: Cow::Borrowed("ggartbez"),
        id: Cow::Borrowed("pktnr"),
        date: Some(Cow::Borrowed("datum")),
        species: None,
        crown: None,
        gone: None,
    },
    skipped_kinds: Cow::Borrowed(&[Cow::Borrowed("Baumstumpf")]),
    attribution: Cow::Borrowed("Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0"),
    max_bytes: MAX_ANSWER_BYTES,
};

/// One answer: the stored features (points in the map's CRS) and the ones that were left out.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TreeAnswer {
    pub features: Vec<Value>,
    /// Features that were no usable tree point, as `<id>: <why>`.
    pub skipped: Vec<String>,
    /// The next page of an OGC API collection.
    pub next: Option<String>,
}

/// Pages of one OGC API collection read at most: 100 000 trees.
const MAX_PAGES: usize = 10;

/// Reads an answer of `dataset` into stored features in `crs`.
///
/// A stored feature is `{"type":"Feature","id":<tree id>,"geometry":Point,"properties":{
/// "tree_dataset", "tree_id", "tree_kind", "tree_date" (YYYY-MM-DD), "tree_species" and
/// "tree_crown_m" when the dataset has them}}`. Kinds in [`TreeDataset::skipped_kinds`] are left
/// out silently; a feature without a point or a tree number is listed in `skipped`.
pub fn parse_trees(text: &str, dataset: &TreeDataset, crs: Crs) -> Result<TreeAnswer, AlkisError> {
    let page: Value = serde_json::from_str(text)
        .map_err(|e| invalid(format!("{}: not GeoJSON: {e}", dataset.name)))?;
    let features = page["features"]
        .as_array()
        .filter(|_| page["type"] == "FeatureCollection")
        .ok_or_else(|| invalid(format!("{}: not a GeoJSON FeatureCollection", dataset.name)))?;
    // A WFS names the CRS of its answer; GeoJSON without one is lon/lat.
    let projected = match &dataset.format {
        TreeFormat::WfsGeoJson { .. } => {
            let matched = page["numberMatched"].as_u64();
            let returned = page["numberReturned"]
                .as_u64()
                .unwrap_or(features.len() as u64);
            let cut = match matched {
                Some(matched) => returned < matched,
                None => features.len() >= TreeFormat::WFS_LIMIT,
            };
            if cut {
                let of = matched.map_or_else(|| "more".to_string(), |m| m.to_string());
                return Err(invalid(format!(
                    "{} returned {returned} of {of} trees; choose a smaller area",
                    dataset.name
                )));
            }
            let named = page["crs"]["properties"]["name"]
                .as_str()
                .unwrap_or_default();
            [Crs::Utm32, Crs::Utm33]
                .into_iter()
                .find(|crs| named.ends_with(&crs.epsg().to_string()))
        }
        _ => None,
    };
    let mut answer = TreeAnswer::default();
    for feature in features {
        normalise(feature, dataset, crs, projected, &mut answer);
    }
    if dataset.format == TreeFormat::OgcApiItems {
        answer.next = page["links"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|link| link["rel"] == "next")
            .and_then(|link| link["href"].as_str())
            .map(str::to_string);
    }
    Ok(answer)
}

/// Reads a CSV file of `dataset` ([`TreeFormat::Csv`]) into stored features in the CRS of
/// `area`, keeping the trees within a metre of `area.bbox`. A row of a kind in
/// [`TreeDataset::skipped_kinds`] or outside the area is left out silently; a row without a
/// position or with too few values is listed in `skipped`. A file without the position columns
/// is an error.
pub fn parse_tree_csv(
    csv: &str,
    dataset: &TreeDataset,
    area: &FetchArea,
) -> Result<TreeAnswer, AlkisError> {
    let TreeFormat::Csv {
        x_field,
        y_field,
        separator,
    } = &dataset.format
    else {
        return Err(invalid(format!("{}: not a CSV cadastre", dataset.name)));
    };
    let mut lines = csv.lines();
    let header: Vec<&str> = lines
        .next()
        .unwrap_or_default()
        .trim_start_matches('\u{feff}')
        .split(*separator)
        .map(str::trim)
        .collect();
    let column = |name: &str| {
        header
            .iter()
            .position(|h| *h == name)
            .ok_or_else(|| invalid(format!("{}: no column {name} in the CSV", dataset.name)))
    };
    let (x, y) = (column(x_field)?, column(y_field)?);
    let keep = area.bbox.buffered(1.0);
    let mut answer = TreeAnswer::default();
    for (number, line) in lines.enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        let values: Vec<&str> = line.split(*separator).map(str::trim).collect();
        if values.len() < header.len() {
            answer
                .skipped
                .push(format!("row {}: {} values", number + 2, values.len()));
            continue;
        }
        let properties: Map<String, Value> = header
            .iter()
            .zip(&values)
            .map(|(key, value)| (key.to_string(), Value::from(*value)))
            .collect();
        let kind = text(&properties, &dataset.fields.kind);
        if kind
            .as_deref()
            .is_some_and(|k| dataset.skipped_kinds.iter().any(|kind| kind == k))
        {
            continue;
        }
        let number = |i: usize| values[i].replace(',', ".").parse::<f64>().ok();
        let position = number(x).zip(number(y)).and_then(|(easting, northing)| {
            dataset.crs.inverse(ProjectedPoint { easting, northing })
        });
        let Some(position) = position else {
            let id = text(&properties, &dataset.fields.id).unwrap_or_default();
            answer.skipped.push(format!("{id}: no point"));
            continue;
        };
        let p = area.crs.forward(position);
        if !(keep.min_e..=keep.max_e).contains(&p.easting)
            || !(keep.min_n..=keep.max_n).contains(&p.northing)
        {
            continue;
        }
        let feature = json!({
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [position.lon, position.lat]},
            "properties": properties,
        });
        normalise(&feature, dataset, area.crs, None, &mut answer);
    }
    Ok(answer)
}

/// The text of a property, whether the service serves it as a string or a number.
fn text<'a>(properties: &'a Map<String, Value>, key: &str) -> Option<std::borrow::Cow<'a, str>> {
    match properties.get(key)? {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().into()),
        Value::Number(n) => Some(n.to_string().into()),
        _ => None,
    }
}

/// Adds the stored form of one point feature (a Point, or a MultiPoint of one point) to
/// `answer`: in `projected` when it names a CRS, in WGS84 lon/lat otherwise (or, from a WFS, in
/// the CRS of the dataset when the numbers are metres). A tree with a
/// value in [`TreeFields::gone`] is left out silently.
fn normalise(
    feature: &Value,
    dataset: &TreeDataset,
    crs: Crs,
    projected: Option<Crs>,
    answer: &mut TreeAnswer,
) {
    let empty = Map::new();
    let properties = feature["properties"].as_object().unwrap_or(&empty);
    let fields = &dataset.fields;
    let kind = text(properties, &fields.kind);
    if kind
        .as_deref()
        .is_some_and(|k| dataset.skipped_kinds.iter().any(|kind| kind == k))
    {
        return;
    }
    if fields
        .gone
        .as_deref()
        .is_some_and(|gone| text(properties, gone).is_some())
    {
        return;
    }
    let Some(id) = text(properties, &fields.id) else {
        answer.skipped.push(format!(
            "{}: no tree number",
            text(properties, "objectid").unwrap_or_default()
        ));
        return;
    };
    let geometry = &feature["geometry"];
    let coordinates = match geometry["type"].as_str() {
        Some("Point") => Some(&geometry["coordinates"]),
        Some("MultiPoint") => geometry["coordinates"]
            .as_array()
            .filter(|points| points.len() == 1)
            .map(|points| &points[0]),
        _ => None,
    };
    let point = coordinates
        .and_then(|c| Some((c[0].as_f64()?, c[1].as_f64()?)))
        .filter(|(x, y)| x.is_finite() && y.is_finite());
    // A WFS that names no CRS answers lon/lat (Kiel) or metres in the CRS asked (Köln).
    let in_metres =
        |x: f64| x.abs() > 360.0 && matches!(dataset.format, TreeFormat::WfsGeoJson { .. });
    let position = point.and_then(|(x, y)| match projected {
        Some(from) => from.inverse(ProjectedPoint {
            easting: x,
            northing: y,
        }),
        None if in_metres(x) => dataset.crs.inverse(ProjectedPoint {
            easting: x,
            northing: y,
        }),
        None => Some(LatLon { lat: y, lon: x }),
    });
    let Some(position) = position else {
        answer.skipped.push(format!("{id}: no point"));
        return;
    };
    let p = crs.forward(position);
    // Centimetres are plenty for a tree.
    let round = |v: f64| (v * 100.0).round() / 100.0;
    let mut stored = Map::new();
    stored.insert("tree_dataset".into(), dataset.id.clone().into());
    stored.insert("tree_id".into(), id.as_ref().into());
    if let Some(kind) = kind {
        stored.insert("tree_kind".into(), kind.as_ref().into());
    }
    // A date that does not parse leaves the fetch date as the currency.
    let date = fields
        .date
        .as_deref()
        .and_then(|f| text(properties, f))
        .and_then(|d| NaiveDate::parse_from_str(d.get(..10)?, "%Y-%m-%d").ok());
    if let Some(date) = date {
        stored.insert("tree_date".into(), date.to_string().into());
    }
    if let Some(species) = fields.species.as_deref().and_then(|f| text(properties, f)) {
        stored.insert("tree_species".into(), species.as_ref().into());
    }
    let crown = fields
        .crown
        .as_deref()
        .and_then(|f| text(properties, f))
        .and_then(|c| c.replace(',', ".").parse::<f64>().ok())
        .filter(|c| c.is_finite() && *c > 0.0);
    if let Some(crown) = crown {
        stored.insert("tree_crown_m".into(), crown.into());
    }
    answer.features.push(json!({
        "type": "Feature",
        "id": id.as_ref(),
        "geometry": {"type": "Point", "coordinates": [round(p.easting), round(p.northing)]},
        "properties": stored,
    }));
}

/// The trees of one dataset in a map area, in the map's CRS.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeFeatures {
    pub features: Vec<Value>,
    pub skipped: Vec<String>,
    pub regions: Vec<AdapterId>,
}

/// One dataset as fetched; an empty `features` is a dataset without trees in the area.
#[derive(Debug)]
pub struct FetchedTreeLayer {
    pub dataset: &'static TreeDataset,
    pub features: Result<TreeFeatures, FetchError>,
}

/// All endpoints of `dataset` for `area`, each point once (by tree number and position). A
/// dataset whose endpoint fails fails as a whole (a half-filled file would pass for complete).
/// Each answer streams into a file `part` names right before the download (removed afterwards),
/// so one over the dataset's `max_bytes` is stopped while it is read.
pub fn fetch_dataset(
    fetcher: &dyn Fetcher,
    dataset: &TreeDataset,
    area: &FetchArea,
    part: PartFile,
) -> Result<TreeFeatures, FetchError> {
    let mut seen = HashSet::new();
    let mut fetched = TreeFeatures {
        features: Vec::new(),
        skipped: Vec::new(),
        regions: vec![dataset.region.clone()],
    };
    for endpoint in dataset.endpoints.iter() {
        let mut next = Some(dataset.url(endpoint, &area.region_bbox).ok_or_else(|| {
            FetchError::Alkis(invalid(format!("{}: area out of range", dataset.name)))
        })?);
        let mut pages = 0;
        while let Some(url) = next.take() {
            pages += 1;
            if pages > MAX_PAGES {
                return Err(FetchError::Alkis(invalid(format!(
                    "{}: more than {MAX_PAGES} pages; choose a smaller area",
                    dataset.name
                ))));
            }
            // An error page with status 200 is not a FeatureCollection and is asked again; an
            // answer over the cap is not.
            let answer =
                get_limited_with_retries(fetcher, &url, &part()?, dataset.max_bytes, |bytes| {
                    let text = String::from_utf8_lossy(bytes);
                    Ok(match dataset.format {
                        TreeFormat::Csv { .. } => parse_tree_csv(&text, dataset, area)?,
                        _ => parse_trees(&text, dataset, area.crs)?,
                    })
                })?;
            for feature in answer.features {
                let c = &feature["geometry"]["coordinates"];
                let key = format!("{}@{},{}", feature["id"], c[0], c[1]);
                if seen.insert(key) {
                    fetched.features.push(feature);
                }
            }
            fetched.skipped.extend(answer.skipped);
            next = answer.next;
        }
    }
    Ok(fetched)
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    /// A Potsdam export answer: three trees of the Stadtkarte (Sanssouci area, 2009) and a stump.
    pub const POTSDAM: &str = include_str!("../../../tests/fixtures/potsdam-trees.geojson");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use std::path::PathBuf;

    /// A city whose trees are an OGC API collection without a survey date, as in Hamburg.
    const CITY: TreeDataset = TreeDataset {
        id: Cow::Borrowed("city"),
        name: Cow::Borrowed("City trees"),
        region: AdapterId::built_in("hamburg"),
        crs: Crs::Utm32,
        envelope: BoundingBox {
            min_e: 540_000.0,
            min_n: 5_920_000.0,
            max_e: 590_000.0,
            max_n: 5_960_000.0,
        },
        format: TreeFormat::OgcApiItems,
        endpoints: Cow::Borrowed(&[Cow::Borrowed(
            "https://api.hamburg.de/datasets/v1/strassenbaumkataster/collections/strassenbaumkataster_hh/items",
        )]),
        fields: TreeFields {
            kind: Cow::Borrowed("gattung_deutsch"),
            id: Cow::Borrowed("baumid"),
            date: None,
            species: Some(Cow::Borrowed("art_deutsch")),
            crown: Some(Cow::Borrowed("kronendurchmesser")),
            gone: None,
        },
        skipped_kinds: Cow::Borrowed(&[]),
        attribution: Cow::Borrowed("City"),
        max_bytes: MAX_ANSWER_BYTES,
    };

    /// The first page of the Hamburg street trees near Blankenese (2026-11-15): three oaks as
    /// one-point MultiPoints in CRS84, and a link to the next page.
    const PAGE: &str = include_str!("../../../tests/fixtures/hamburg-trees.geojson");

    #[test]
    fn an_ogc_api_page_is_read() {
        let answer = parse_trees(PAGE, &CITY, Crs::Utm32).unwrap();
        assert!(answer.skipped.is_empty(), "{:?}", answer.skipped);
        assert_eq!(answer.features.len(), 3);
        let oak = &answer.features[0];
        assert_eq!(oak["id"], "200004579");
        assert_eq!(oak["geometry"]["type"], "Point");
        let [e, n] = [0, 1].map(|i| oak["geometry"]["coordinates"][i].as_f64().unwrap());
        assert!(
            (555_000.0..556_000.0).contains(&e) && (5_934_000.0..5_935_000.0).contains(&n),
            "{e} {n}"
        );
        let properties = &oak["properties"];
        assert_eq!(properties["tree_kind"], "Eiche");
        assert_eq!(properties["tree_species"], "Stiel-Eiche");
        assert_eq!(properties["tree_crown_m"], 14.0);
        assert!(properties.get("tree_date").is_none(), "{properties}");
        assert_eq!(
            answer.next.as_deref(),
            Some(
                "https://api.hamburg.de/datasets/v1/strassenbaumkataster/collections/strassenbaumkataster_hh/items?bbox=9.836%2C53.553%2C9.851%2C53.560&f=json&limit=3&offset=3"
            )
        );
    }

    #[test]
    fn the_ogc_api_url_asks_the_box_in_wgs84() {
        // Blankenese in EPSG:25833 (the region box): asked as lon,lat in CRS84.
        let bbox = BoundingBox {
            min_e: 552_737.0 - 500_000.0 + 101_000.0,
            min_n: 5_934_347.0,
            max_e: 553_646.0 - 500_000.0 + 101_000.0,
            max_n: 5_935_057.0,
        };
        let url = CITY.url(&CITY.endpoints[0], &bbox).unwrap();
        let (base, query) = url.split_once('?').unwrap();
        assert_eq!(base, CITY.endpoints[0]);
        let params: Vec<&str> = query.split('&').collect();
        assert_eq!(params[1..], ["limit=10000", "f=json"]);
        let numbers: Vec<f64> = params[0]
            .strip_prefix("bbox=")
            .unwrap()
            .split(',')
            .map(|v| v.parse().unwrap())
            .collect();
        assert_eq!(numbers.len(), 4);
        assert!(
            numbers[0] < numbers[2] && numbers[1] < numbers[3],
            "{numbers:?}"
        );
        assert!(
            (5.0..16.0).contains(&numbers[0]) && (47.0..56.0).contains(&numbers[1]),
            "lon,lat: {numbers:?}"
        );
    }

    #[test]
    fn an_ogc_api_dataset_follows_its_next_links() {
        let page: Value = serde_json::from_str(PAGE).unwrap();
        let next = page["links"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| l["rel"] == "next")
            .unwrap()["href"]
            .as_str()
            .unwrap()
            .to_string();
        // The last page: the same three trees under other numbers, and no next link.
        let mut last = page.clone();
        for (i, feature) in last["features"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .enumerate()
        {
            feature["properties"]["baumid"] = json!(900 + i);
        }
        last["links"] = json!([]);
        let area = FetchArea::new(
            BoundingBox {
                min_e: 552_737.0,
                min_n: 5_934_347.0,
                max_e: 553_646.0,
                max_n: 5_935_057.0,
            },
            Crs::Utm32,
        );
        let fetcher = MapFetcher::default();
        let first = CITY.url(&CITY.endpoints[0], &area.region_bbox).unwrap();
        fetcher.insert(&first, PAGE.as_bytes().to_vec());
        fetcher.insert(&next, last.to_string().into_bytes());
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("trees.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        let fetched = fetch_dataset(&fetcher, &CITY, &area, &part).unwrap();
        assert_eq!(fetched.features.len(), 6);
        assert_eq!(*fetcher.log.borrow(), [first, next]);
    }

    /// A city whose trees are one CSV file in EPSG:25832, as in Lübeck: semicolons, decimal
    /// commas, tree numbers that start again in every street, and empty planting spots.
    const CSV_CITY: TreeDataset = TreeDataset {
        id: Cow::Borrowed("csv-city"),
        name: Cow::Borrowed("CSV city trees"),
        region: AdapterId::built_in("schleswigholstein"),
        crs: Crs::Utm32,
        envelope: BoundingBox {
            min_e: 602_000.0,
            min_n: 5_959_000.0,
            max_e: 625_000.0,
            max_n: 5_984_000.0,
        },
        format: TreeFormat::Csv {
            x_field: Cow::Borrowed("RECHTSWERT"),
            y_field: Cow::Borrowed("HOCHWERT"),
            separator: ';',
        },
        endpoints: Cow::Borrowed(&[Cow::Borrowed(
            "https://opendata.luebeck.de/bereich/5.660/stadtgruen/baumkataster/baumkataster.csv",
        )]),
        fields: TreeFields {
            kind: Cow::Borrowed("GATTUNG"),
            id: Cow::Borrowed("BAUMNUMMER"),
            date: None,
            species: Some(Cow::Borrowed("GA_LANG")),
            crown: None,
            gone: None,
        },
        skipped_kinds: Cow::Borrowed(&[Cow::Borrowed("Fehlplatz")]),
        attribution: Cow::Borrowed("CSV city"),
        max_bytes: MAX_ANSWER_BYTES,
    };

    /// Four rows: a linden in the area, an empty spot, a maple outside, a row without a position.
    const CSV: &str = "BAUMNUMMER;HOCHWERT;RECHTSWERT;GATTUNG;GA_LANG;STRASSE;STANDORT;PFLANZJAHR\n \
         1;5969800,5;610900,25;Tilia;Tilia cordata, Winterlinde;Koberg;vor 2;1950\n \
         2;5969810,0;610910,0;Fehlplatz;Fehlplatz;Koberg;;\n \
         1;5975000,0;615000,0;Acer;Acer platanoides, Spitzahorn;Elsewhere;;1990\n \
         3;;610905,0;Acer;Acer campestre, Feldahorn;Koberg;;\n";

    /// The Lübeck old town example (EPSG:25832).
    fn old_town() -> FetchArea {
        FetchArea {
            region_bbox: BoundingBox {
                min_e: 0.0,
                min_n: 0.0,
                max_e: 0.0,
                max_n: 0.0,
            },
            crs: Crs::Utm32,
            bbox: BoundingBox {
                min_e: 610_424.0,
                min_n: 5_969_571.0,
                max_e: 611_440.0,
                max_n: 5_970_293.0,
            },
        }
    }

    #[test]
    fn a_csv_cadastre_keeps_the_trees_of_the_area() {
        let answer = parse_tree_csv(CSV, &CSV_CITY, &old_town()).unwrap();
        assert_eq!(answer.skipped, ["3: no point"]);
        assert_eq!(answer.features.len(), 1, "{:?}", answer.features);
        let linden = &answer.features[0];
        assert_eq!(linden["id"], "1");
        assert_eq!(
            linden["geometry"]["coordinates"],
            json!([610_900.25, 5_969_800.5])
        );
        let properties = &linden["properties"];
        assert_eq!(properties["tree_kind"], "Tilia");
        assert_eq!(properties["tree_species"], "Tilia cordata, Winterlinde");
        assert!(properties.get("tree_date").is_none(), "{properties}");
        // Not a CSV with the named columns.
        let err = parse_tree_csv("a;b\n1;2\n", &CSV_CITY, &old_town()).unwrap_err();
        assert!(err.message.contains("RECHTSWERT"), "{}", err.message);
    }

    #[test]
    fn a_csv_cadastre_is_fetched_whole_and_cut_to_the_area() {
        assert_eq!(
            CSV_CITY.url(&CSV_CITY.endpoints[0], &old_town().region_bbox),
            Some(CSV_CITY.endpoints[0].to_string())
        );
        let fetcher = MapFetcher::default();
        fetcher.insert(&CSV_CITY.endpoints[0], CSV.as_bytes().to_vec());
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("trees.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        let fetched = fetch_dataset(&fetcher, &CSV_CITY, &old_town(), &part).unwrap();
        assert_eq!(fetched.features.len(), 1);
        assert_eq!(fetcher.calls.get(), 1);
    }

    /// A city whose trees are a WFS answering GeoJSON in its CRS, as in Leipzig, with a field
    /// that holds the date a tree was felled.
    const WFS_CITY: TreeDataset = TreeDataset {
        id: Cow::Borrowed("wfs-city"),
        name: Cow::Borrowed("WFS city trees"),
        region: AdapterId::built_in("sachsen"),
        crs: Crs::Utm33,
        envelope: BoundingBox {
            min_e: 307_000.0,
            min_n: 5_680_000.0,
            max_e: 329_000.0,
            max_n: 5_703_000.0,
        },
        format: TreeFormat::WfsGeoJson {
            type_name: Cow::Borrowed("OpenData:Baeume"),
            output_format: Cow::Borrowed("application/json"),
        },
        endpoints: Cow::Borrowed(&[Cow::Borrowed(
            "https://geodienste.leipzig.de/l3/OpenData/Baeume/wfs",
        )]),
        fields: TreeFields {
            kind: Cow::Borrowed("gattung"),
            id: Cow::Borrowed("baumnummer"),
            date: None,
            species: Some(Cow::Borrowed("ga_lang_deutsch")),
            crown: Some(Cow::Borrowed("kr_durchm")),
            gone: Some(Cow::Borrowed("gefaellt_am")),
        },
        skipped_kinds: Cow::Borrowed(&[]),
        attribution: Cow::Borrowed("WFS city"),
        max_bytes: MAX_ANSWER_BYTES,
    };

    /// A Leipzig answer (2026-11-15): a maple, and a felled tree.
    const WFS_ANSWER: &str = r#"{"type":"FeatureCollection","numberMatched":2,"numberReturned":2,
        "crs":{"type":"name","properties":{"name":"urn:ogc:def:crs:EPSG::25833"}},"features":[
        {"type":"Feature","geometry":{"type":"Point","coordinates":[316919.01,5691016.99]},
         "properties":{"baumnummer":"G4519","gattung":"Acer","ga_lang_deutsch":"Feld-Ahorn","kr_durchm":1.5,"gefaellt_am":null}},
        {"type":"Feature","geometry":{"type":"Point","coordinates":[316950.0,5691020.0]},
         "properties":{"baumnummer":"G4520","gattung":"Tilia","ga_lang_deutsch":"Winter-Linde","kr_durchm":8,"gefaellt_am":"2025-03-01"}}]}"#;

    #[test]
    fn a_wfs_answer_in_its_crs_is_read_without_the_felled_trees() {
        let answer = parse_trees(WFS_ANSWER, &WFS_CITY, Crs::Utm33).unwrap();
        assert!(answer.skipped.is_empty(), "{:?}", answer.skipped);
        assert_eq!(answer.features.len(), 1);
        let maple = &answer.features[0];
        assert_eq!(maple["id"], "G4519");
        assert_eq!(
            maple["geometry"]["coordinates"],
            json!([316_919.01, 5_691_016.99])
        );
        assert_eq!(maple["properties"]["tree_crown_m"], 1.5);
        assert_eq!(maple["properties"]["tree_species"], "Feld-Ahorn");
        // A cut answer is refused: the area is too large for one request.
        let cut = WFS_ANSWER.replace(r#""numberMatched":2"#, r#""numberMatched":5"#);
        let err = parse_trees(&cut, &WFS_CITY, Crs::Utm33).unwrap_err();
        assert!(err.message.contains("2 of 5"), "{}", err.message);
    }

    #[test]
    fn a_wfs_answer_in_wgs84_is_projected() {
        // Kiel's server answers lon/lat whatever is asked, and names no CRS.
        let kiel = r#"{"type":"FeatureCollection","features":[{"type":"Feature",
            "geometry":{"type":"Point","coordinates":[10.14678061,54.33031936]},
            "properties":{"baumnummer":100092167,"gattung":"Cupressus"}}]}"#;
        let answer = parse_trees(kiel, &WFS_CITY, Crs::Utm32).unwrap();
        let [e, n] = [0, 1].map(|i| {
            answer.features[0]["geometry"]["coordinates"][i]
                .as_f64()
                .unwrap()
        });
        assert!(
            (574_000.0..576_000.0).contains(&e) && (6_020_000.0..6_022_000.0).contains(&n),
            "{e} {n}"
        );
        assert_eq!(answer.features[0]["id"], "100092167");
    }

    #[test]
    fn the_wfs_url_asks_the_box_in_the_crs_of_the_dataset() {
        // The Leipzig box above in EPSG:25833, the CRS of the dataset too.
        let bbox = BoundingBox {
            min_e: 316_800.0,
            min_n: 5_690_700.0,
            max_e: 317_300.0,
            max_n: 5_691_200.0,
        };
        assert_eq!(
            WFS_CITY.url(&WFS_CITY.endpoints[0], &bbox).unwrap(),
            "https://geodienste.leipzig.de/l3/OpenData/Baeume/wfs?SERVICE=WFS&VERSION=2.0.0\
             &REQUEST=GetFeature&TYPENAMES=OpenData:Baeume&SRSNAME=urn:ogc:def:crs:EPSG::25833\
             &BBOX=316799,5690699,317301,5691201,urn:ogc:def:crs:EPSG::25833&COUNT=100000\
             &OUTPUTFORMAT=application/json"
        );
        // An endpoint with a query of its own gets the parameters after it.
        let url = WFS_CITY
            .url("https://example.org/ogc.ashx?NodeId=1633", &bbox)
            .unwrap();
        assert!(
            url.starts_with("https://example.org/ogc.ashx?NodeId=1633&SERVICE=WFS&"),
            "{url}"
        );
    }

    #[test]
    fn a_wfs_answer_in_metres_without_a_crs_name_is_in_the_crs_asked() {
        // Köln's server answers in the CRS asked and names none.
        let unnamed = r#"{"type":"FeatureCollection","numberMatched":1,"numberReturned":1,"features":[
            {"type":"Feature","geometry":{"type":"Point","coordinates":[316919.01,5691016.99]},
             "properties":{"baumnummer":"G4519","gattung":"Acer"}}]}"#;
        let answer = parse_trees(unnamed, &WFS_CITY, Crs::Utm33).unwrap();
        assert_eq!(
            answer.features[0]["geometry"]["coordinates"],
            json!([316_919.01, 5_691_016.99])
        );
    }
}
