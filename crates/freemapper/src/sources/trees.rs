//! City tree cadastres (single trees with a species, a survey date and sometimes a crown
//! diameter) for the 417/418 candidates.
//!
//! One row per city in [`TREE_DATASETS`]: where the points come from ([`TreeFormat`], the
//! endpoints), the field names, the kinds that are no tree (stumps), the licence line and the
//! envelope that decides whether a map area needs the dataset. Potsdam is the first row
//! ([`POTSDAM`], an OpenDataSoft GeoJSON export); the WFS, OGC API and CSV formats of the other
//! cities (Leipzig, Dresden, Hamburg, Frankfurt, Kiel, Lübeck, Rostock, Köln, Dortmund, Münster)
//! come with the row that first needs them. Every format normalises to the same stored feature:
//! a point in the map's CRS with the `tree_*` properties of [`parse_trees`].

use std::collections::HashSet;

use chrono::NaiveDate;
use serde_json::{Map, Value, json};

use crate::crs::{BoundingBox, Crs, LatLon, ProjectedPoint};
use crate::fetch::{FetchError, Fetcher, get_limited_with_retries};
use crate::sources::alkis::{AlkisError, FetchArea, invalid};
use crate::sources::alkis_nas::PartFile;
use crate::sources::region::{Region, touches_region};

/// How a dataset is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeFormat {
    /// An OpenDataSoft `exports/geojson` endpoint, asked with `where=in_bbox(<geo_field>, …)` in
    /// WGS84 (no paging). The geometry is WGS84 too.
    OdsGeoJsonExport { geo_field: &'static str },
}

/// The field names of a dataset (properties of the served features).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeFields {
    /// The kind of the point (`Laubbaum`, `Baumstumpf`).
    pub kind: &'static str,
    /// The tree's number, the `freemapper:source_id`.
    pub id: &'static str,
    /// The survey date (`2009-06-10T00:00:00+00:00`).
    pub date: &'static str,
    pub species: Option<&'static str>,
    /// The crown diameter in metres.
    pub crown: Option<&'static str>,
}

/// One city's tree cadastre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeDataset {
    /// The file name under `data/trees/` and the `freemapper:source` suffix (`trees/<id>`).
    pub id: &'static str,
    /// For messages.
    pub name: &'static str,
    /// The state the city lies in; the dataset is asked only for areas that touch it.
    pub region: Region,
    /// The CRS of `envelope`.
    pub crs: Crs,
    /// The data extent rounded outward: an area outside asks for nothing.
    pub envelope: BoundingBox,
    pub format: TreeFormat,
    /// The endpoints, whose points are merged (Potsdam serves 283 772 points in three datasets).
    pub endpoints: &'static [&'static str],
    pub fields: TreeFields,
    /// Kinds that are no tree and are not stored.
    pub skipped_kinds: &'static [&'static str],
    /// The line for the attribution once a tree of the dataset is drawn.
    pub attribution: &'static str,
    /// The largest answer of one endpoint (bytes); a larger one fails the dataset.
    pub max_bytes: u64,
}

/// 256 MB, as for the NAS answers: the whole Potsdam envelope (283 772 points) is about 110 MB.
const MAX_ANSWER_BYTES: u64 = 256 << 20;

/// Potsdam, "Bäume der Stadtkarte" (`opendata.potsdam.de`, dl-de/zero-2-0): 283 772 points in
/// three datasets, no crown diameter. The envelope is the data extent (E 359 000–375 000,
/// N 5 801 000–5 820 000, EPSG:25833) rounded outward.
pub const POTSDAM: TreeDataset = TreeDataset {
    id: "potsdam",
    name: "Potsdam tree cadastre",
    region: Region::Brandenburg,
    crs: Crs::Utm33,
    envelope: BoundingBox {
        min_e: 359_000.0,
        min_n: 5_801_000.0,
        max_e: 375_000.0,
        max_n: 5_820_000.0,
    },
    format: TreeFormat::OdsGeoJsonExport {
        geo_field: "geo_point_2d",
    },
    endpoints: &[
        "https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/baeume-1-stadtkarte-potsdam/exports/geojson",
        "https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/baeume-2-stadtkarte-potsdam/exports/geojson",
        "https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/baeume-3-stadtkarte-potsdam/exports/geojson",
    ],
    fields: TreeFields {
        kind: "ggartbez",
        id: "pktnr",
        date: "datum",
        species: None,
        crown: None,
    },
    skipped_kinds: &["Baumstumpf"],
    attribution: "Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0",
    max_bytes: MAX_ANSWER_BYTES,
};

/// The tree cadastres, one per city.
pub const TREE_DATASETS: [TreeDataset; 1] = [POTSDAM];

/// The datasets a map area needs: those whose envelope the part of the area inside their state
/// intersects (the envelope of Potsdam reaches into Berlin, which has its own register).
/// `region_bbox` is the area in EPSG:25833.
pub fn tree_datasets_for(region_bbox: &BoundingBox) -> Vec<&'static TreeDataset> {
    TREE_DATASETS
        .iter()
        .filter(|d| needs(d, region_bbox))
        .collect()
}

/// Whether the area in `region_bbox` (EPSG:25833) needs `dataset`.
fn needs(dataset: &TreeDataset, region_bbox: &BoundingBox) -> bool {
    let area = FetchArea::new(*region_bbox, Crs::Utm33).bbox_for(dataset.crs);
    if !intersects(&area, &dataset.envelope) {
        return false;
    }
    // The state's boundary is in EPSG:25833: clip the area to the envelope there.
    if dataset.crs != Crs::Utm33 {
        return touches_region(dataset.region, region_bbox);
    }
    let e = &dataset.envelope;
    let clip = BoundingBox {
        min_e: area.min_e.max(e.min_e),
        min_n: area.min_n.max(e.min_n),
        max_e: area.max_e.min(e.max_e),
        max_n: area.max_n.min(e.max_n),
    };
    touches_region(dataset.region, &clip)
}

fn intersects(a: &BoundingBox, b: &BoundingBox) -> bool {
    a.min_e <= b.max_e && b.min_e <= a.max_e && a.min_n <= b.max_n && b.min_n <= a.max_n
}

/// The request grows the area by this much (ground metres): a tree on its edge is in.
const REQUEST_MARGIN_M: f64 = 1.0;

/// The WGS84 envelope (lat_min, lon_min, lat_max, lon_max) of `bbox` (EPSG:25833) grown by
/// [`REQUEST_MARGIN_M`], from points along its outline (the edges bend in WGS84).
fn wgs84_bbox(bbox: &BoundingBox) -> Option<(f64, f64, f64, f64)> {
    const STEPS: u32 = 8;
    let bbox = bbox.buffered(REQUEST_MARGIN_M);
    let mut out: Option<(f64, f64, f64, f64)> = None;
    for i in 0..=STEPS {
        let t = f64::from(i) / f64::from(STEPS);
        let e = bbox.min_e + t * (bbox.max_e - bbox.min_e);
        let n = bbox.min_n + t * (bbox.max_n - bbox.min_n);
        for (easting, northing) in [
            (e, bbox.min_n),
            (e, bbox.max_n),
            (bbox.min_e, n),
            (bbox.max_e, n),
        ] {
            let p = Crs::Utm33.inverse(ProjectedPoint { easting, northing })?;
            out = Some(match out {
                None => (p.lat, p.lon, p.lat, p.lon),
                Some((a, b, c, d)) => (a.min(p.lat), b.min(p.lon), c.max(p.lat), d.max(p.lon)),
            });
        }
    }
    out
}

impl TreeDataset {
    /// The request for `endpoint` for the points in `region_bbox` (EPSG:25833, grown by 1 m).
    /// The box is asked as `lat_min,lon_min,lat_max,lon_max`. `None` for an area no position
    /// projects to.
    pub fn url(&self, endpoint: &str, region_bbox: &BoundingBox) -> Option<String> {
        let (lat_min, lon_min, lat_max, lon_max) = wgs84_bbox(region_bbox)?;
        Some(match self.format {
            TreeFormat::OdsGeoJsonExport { geo_field } => format!(
                "{endpoint}?where=in_bbox({geo_field},{lat_min:.6},{lon_min:.6},{lat_max:.6},{lon_max:.6})"
            ),
        })
    }
}

/// One answer: the stored features (points in the map's CRS) and the ones that were left out.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TreeAnswer {
    pub features: Vec<Value>,
    /// Features that were no usable tree point, as `<id>: <why>`.
    pub skipped: Vec<String>,
}

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
    let mut answer = TreeAnswer::default();
    for feature in features {
        match dataset.format {
            TreeFormat::OdsGeoJsonExport { .. } => normalise(feature, dataset, crs, &mut answer),
        }
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

/// Adds the stored form of one WGS84 point feature to `answer`.
fn normalise(feature: &Value, dataset: &TreeDataset, crs: Crs, answer: &mut TreeAnswer) {
    let empty = Map::new();
    let properties = feature["properties"].as_object().unwrap_or(&empty);
    let fields = &dataset.fields;
    let kind = text(properties, fields.kind);
    if kind
        .as_deref()
        .is_some_and(|k| dataset.skipped_kinds.contains(&k))
    {
        return;
    }
    let Some(id) = text(properties, fields.id) else {
        answer.skipped.push(format!(
            "{}: no tree number",
            text(properties, "objectid").unwrap_or_default()
        ));
        return;
    };
    let point = (feature["geometry"]["type"] == "Point")
        .then(|| &feature["geometry"]["coordinates"])
        .and_then(|c| Some((c[0].as_f64()?, c[1].as_f64()?)))
        .filter(|(lon, lat)| lon.is_finite() && lat.is_finite());
    let Some((lon, lat)) = point else {
        answer.skipped.push(format!("{id}: no point"));
        return;
    };
    let p = crs.forward(LatLon { lat, lon });
    // Centimetres are plenty for a tree.
    let round = |v: f64| (v * 100.0).round() / 100.0;
    let mut stored = Map::new();
    stored.insert("tree_dataset".into(), dataset.id.into());
    stored.insert("tree_id".into(), id.as_ref().into());
    if let Some(kind) = kind {
        stored.insert("tree_kind".into(), kind.as_ref().into());
    }
    // A date that does not parse leaves the fetch date as the currency.
    let date = text(properties, fields.date)
        .and_then(|d| NaiveDate::parse_from_str(d.get(..10)?, "%Y-%m-%d").ok());
    if let Some(date) = date {
        stored.insert("tree_date".into(), date.to_string().into());
    }
    if let Some(species) = fields.species.and_then(|f| text(properties, f)) {
        stored.insert("tree_species".into(), species.as_ref().into());
    }
    let crown = fields
        .crown
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
    pub regions: Vec<Region>,
}

/// One dataset as fetched; an empty `features` is a dataset without trees in the area.
#[derive(Debug)]
pub struct FetchedTreeLayer {
    pub dataset: &'static TreeDataset,
    pub features: Result<TreeFeatures, FetchError>,
}

/// The tree datasets for `area`; none, and no request, when the area is outside every envelope.
/// A dataset whose endpoint fails fails as a whole (a half-filled file would pass for
/// complete); the others go on. Each answer streams into a file `part` names right before the
/// download (removed afterwards), so one over the dataset's `max_bytes` is stopped while it is
/// read.
pub fn fetch_tree_layers(
    fetcher: &dyn Fetcher,
    area: &FetchArea,
    part: PartFile,
) -> Vec<FetchedTreeLayer> {
    tree_datasets_for(&area.region_bbox)
        .into_iter()
        .map(|dataset| FetchedTreeLayer {
            dataset,
            features: fetch_dataset(fetcher, dataset, area, part),
        })
        .collect()
}

/// All endpoints of `dataset` for `area`, each point once (by tree number and position).
fn fetch_dataset(
    fetcher: &dyn Fetcher,
    dataset: &TreeDataset,
    area: &FetchArea,
    part: PartFile,
) -> Result<TreeFeatures, FetchError> {
    let mut seen = HashSet::new();
    let mut fetched = TreeFeatures {
        features: Vec::new(),
        skipped: Vec::new(),
        regions: vec![dataset.region],
    };
    for endpoint in dataset.endpoints {
        let url = dataset.url(endpoint, &area.region_bbox).ok_or_else(|| {
            FetchError::Alkis(invalid(format!("{}: area out of range", dataset.name)))
        })?;
        // An error page with status 200 is not a FeatureCollection and is asked again; an
        // answer over the cap is not.
        let answer =
            get_limited_with_retries(fetcher, &url, &part()?, dataset.max_bytes, |bytes| {
                Ok(parse_trees(
                    &String::from_utf8_lossy(bytes),
                    dataset,
                    area.crs,
                )?)
            })?;
        for feature in answer.features {
            let c = &feature["geometry"]["coordinates"];
            let key = format!("{}@{},{}", feature["id"], c[0], c[1]);
            if seen.insert(key) {
                fetched.features.push(feature);
            }
        }
        fetched.skipped.extend(answer.skipped);
    }
    Ok(fetched)
}

#[cfg(test)]
pub(crate) mod test_support {
    /// A Potsdam export answer: three trees of the Stadtkarte (Sanssouci area, 2009) and a stump.
    pub const POTSDAM: &str = include_str!("../../../../tests/fixtures/potsdam-trees.geojson");
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::fetch::test_support::MapFetcher;

    /// Sanssouci, Potsdam (EPSG:25833), where the fixture trees stand.
    fn sanssouci() -> BoundingBox {
        BoundingBox {
            min_e: 366_200.0,
            min_n: 5_807_100.0,
            max_e: 367_100.0,
            max_n: 5_807_700.0,
        }
    }

    /// The numbers of `in_bbox(geo_point_2d,a,b,c,d)`.
    fn in_bbox(url: &str) -> Vec<f64> {
        let start = url.find("in_bbox(geo_point_2d,").expect(url) + "in_bbox(geo_point_2d,".len();
        let end = start + url[start..].find(')').expect(url);
        url[start..end]
            .split(',')
            .map(|n| n.parse().unwrap())
            .collect()
    }

    #[test]
    fn potsdam_url_has_wgs84_bbox() {
        let urls: Vec<String> = POTSDAM
            .endpoints
            .iter()
            .map(|e| POTSDAM.url(e, &sanssouci()).unwrap())
            .collect();
        assert_eq!(urls.len(), 3);
        for (i, url) in urls.iter().enumerate() {
            let id = format!(
                "baeume-{}-stadtkarte-potsdam/exports/geojson?where=in_bbox(",
                i + 1
            );
            assert!(url.contains(&id), "{url}");
            // lat_min, lon_min, lat_max, lon_max: latitude first.
            let [lat_min, lon_min, lat_max, lon_max] = in_bbox(url)[..] else {
                panic!("{url}")
            };
            assert!(
                (52.39..52.41).contains(&lat_min) && lat_min < lat_max,
                "{url}"
            );
            assert!(
                (13.03..13.06).contains(&lon_min) && lon_min < lon_max,
                "{url}"
            );
            // The box is the area plus 1 m (about 9e-6 degrees of latitude) and the grid
            // is turned against north by half a degree here: it holds the fixture tree at
            // 13.038386 E, 52.400320 N, and the height is the area's, give or take the turn.
            assert!(lat_min < 52.400320 && 52.400320 < lat_max, "{url}");
            assert!(lon_min < 13.038386 && 13.038386 < lon_max, "{url}");
            assert!(
                (lat_max - lat_min - 600.0 / 111_200.0).abs() < 3e-4,
                "{url}"
            );
        }
    }

    #[test]
    fn trees_only_inside_the_dataset_envelope() {
        let names = |bbox: &BoundingBox| -> Vec<&str> {
            tree_datasets_for(bbox).iter().map(|d| d.id).collect()
        };
        assert_eq!(names(&sanssouci()), ["potsdam"]);
        // Straddling the envelope edge still needs the dataset.
        let edge = BoundingBox {
            min_e: 358_000.0,
            min_n: 5_808_000.0,
            max_e: 359_500.0,
            max_n: 5_809_000.0,
        };
        assert_eq!(names(&edge), ["potsdam"]);
        let strausberg = BoundingBox {
            min_e: 421_000.0,
            min_n: 5_821_000.0,
            max_e: 422_500.0,
            max_n: 5_822_000.0,
        };
        let mitte = BoundingBox {
            min_e: 392_000.0,
            min_n: 5_820_000.0,
            max_e: 393_500.0,
            max_n: 5_821_000.0,
        };
        // Kladow lies inside the envelope but in Berlin.
        let kladow = BoundingBox {
            min_e: 374_000.0,
            min_n: 5_813_000.0,
            max_e: 374_500.0,
            max_n: 5_813_500.0,
        };
        let dresden = BoundingBox {
            min_e: 415_800.0,
            min_n: 5_655_600.0,
            max_e: 416_150.0,
            max_n: 5_655_900.0,
        };
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("answer.geojson.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        let fetcher = MapFetcher::default();
        for outside in [strausberg, mitte, kladow, dresden] {
            assert!(names(&outside).is_empty(), "{outside:?}");
            assert!(
                fetch_tree_layers(&fetcher, &FetchArea::new(outside, Crs::Utm33), &part).is_empty()
            );
        }
        assert_eq!(fetcher.calls.get(), 0);
        // Inside: one request per endpoint.
        fetcher.insert_prefix(
            POTSDAM.endpoints[0],
            test_support::POTSDAM.as_bytes().to_vec(),
        );
        fetcher.insert_prefix(
            POTSDAM.endpoints[1],
            br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
        );
        fetcher.insert_prefix(
            POTSDAM.endpoints[2],
            br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
        );
        let layers = fetch_tree_layers(&fetcher, &FetchArea::new(sanssouci(), Crs::Utm33), &part);
        assert_eq!(fetcher.calls.get(), 3);
        assert_eq!(layers.len(), 1);
        let trees = layers[0].features.as_ref().unwrap();
        assert_eq!(trees.features.len(), 3);
        assert_eq!(trees.regions, [Region::Brandenburg]);
    }

    #[test]
    fn tree_answer_over_the_size_cap_fails_the_dataset() {
        assert_eq!(POTSDAM.max_bytes, 256 << 20);
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("answer.geojson.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        let area = FetchArea::new(sanssouci(), Crs::Utm33);
        let size = test_support::POTSDAM.len() as u64;
        let fetcher = MapFetcher::default();
        for endpoint in POTSDAM.endpoints {
            fetcher.insert_prefix(endpoint, test_support::POTSDAM.as_bytes().to_vec());
        }
        // One byte over the cap: stopped while it is read, not asked again.
        let small = TreeDataset {
            max_bytes: size - 1,
            ..POTSDAM
        };
        let err = fetch_dataset(&fetcher, &small, &area, &part).unwrap_err();
        assert!(matches!(err, FetchError::TooLarge { .. }), "{err}");
        assert_eq!(fetcher.calls.get(), 1);
        assert!(!part_path.exists(), "the part file is removed");
        // At the cap it is read.
        let exact = TreeDataset {
            max_bytes: size,
            ..POTSDAM
        };
        let trees = fetch_dataset(&fetcher, &exact, &area, &part).unwrap();
        assert_eq!(trees.features.len(), 3);
        assert!(!part_path.exists());
    }

    #[test]
    fn dataset_needs_its_state_inside_the_envelope() {
        // The Brandenburg/Berlin border near Kladow: Brandenburg west of E 373 750, Berlin east
        // of E 374 000 (the border slants), at N 5 817 000–5 817 250.
        let area = BoundingBox {
            min_e: 373_000.0,
            min_n: 5_817_000.0,
            max_e: 374_800.0,
            max_n: 5_817_250.0,
        };
        let row = |region, min_e, max_e| TreeDataset {
            region,
            envelope: BoundingBox {
                min_e,
                min_n: 5_816_000.0,
                max_e,
                max_n: 5_819_000.0,
            },
            ..POTSDAM
        };
        // The area touches both states, but the envelope covers only the Berlin side.
        let berlin_side = |region| row(region, 374_300.0, 374_700.0);
        assert!(needs(&berlin_side(Region::Berlin), &area));
        assert!(!needs(&berlin_side(Region::Brandenburg), &area));
        // And only the Brandenburg side.
        let brandenburg_side = |region| row(region, 373_000.0, 373_400.0);
        assert!(needs(&brandenburg_side(Region::Brandenburg), &area));
        assert!(!needs(&brandenburg_side(Region::Berlin), &area));
    }

    #[test]
    fn unparsable_date_leaves_the_fetch_date_as_currency() {
        let text = test_support::POTSDAM.replace("2009-06-10T00:00:00+00:00", "unbekannt");
        let answer = parse_trees(&text, &POTSDAM, Crs::Utm33).unwrap();
        assert_eq!(answer.features.len(), 3);
        for tree in &answer.features {
            assert!(tree["properties"].get("tree_date").is_none(), "{tree}");
        }
    }

    #[test]
    fn tree_point_keeps_kind_id_and_date() {
        let answer = parse_trees(test_support::POTSDAM, &POTSDAM, Crs::Utm33).unwrap();
        assert!(answer.skipped.is_empty(), "{:?}", answer.skipped);
        let tree = answer
            .features
            .iter()
            .find(|f| f["id"] == "2005108M-30600")
            .expect("the first tree");
        let p = &tree["properties"];
        assert_eq!(p["tree_dataset"], "potsdam");
        assert_eq!(p["tree_id"], "2005108M-30600");
        assert_eq!(p["tree_kind"], "Laubbaum");
        assert_eq!(p["tree_date"], "2009-06-10");
        assert!(p.get("tree_crown_m").is_none() && p.get("tree_species").is_none());
        // The WGS84 point is in the map's CRS now: the dataset's own pktx/pkty.
        assert_eq!(tree["geometry"]["type"], "Point");
        let [e, n] = [0, 1].map(|i| tree["geometry"]["coordinates"][i].as_f64().unwrap());
        assert!(
            (e - 366_545.3).abs() < 0.05 && (n - 5_807_374.74).abs() < 0.05,
            "{e} {n}"
        );
        // Another CRS moves the point.
        let utm32 = parse_trees(test_support::POTSDAM, &POTSDAM, Crs::Utm32).unwrap();
        assert!(
            (utm32.features[0]["geometry"]["coordinates"][0]
                .as_f64()
                .unwrap()
                - e)
                .abs()
                > 1000.0
        );
    }

    #[test]
    fn stump_is_no_candidate() {
        // The fixture holds a stump next to the three trees: the reader does not store it.
        let answer = parse_trees(test_support::POTSDAM, &POTSDAM, Crs::Utm33).unwrap();
        assert_eq!(answer.features.len(), 3);
        assert!(
            answer
                .features
                .iter()
                .all(|f| f["properties"]["tree_kind"] != "Baumstumpf")
        );
        assert!(answer.skipped.is_empty());
    }
}
