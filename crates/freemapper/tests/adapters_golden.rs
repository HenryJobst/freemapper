//! The golden net of the region adapter migration (ADR 0082, plan stage A). Five layers, all
//! offline, recorded from the v0.12.0 behaviour. A refactor must leave every file under
//! `tests/golden/adapters/` as it is; `FREEMAPPER_BLESS=1` rewrites them on purpose, see
//! `tests/golden/adapters/README.md`.

use std::path::{Path, PathBuf};

use freemapper::crs::{BoundingBox, Crs, LatLon, ProjectedPoint, utm33_forward};
use freemapper::project::ProjectFile;
use freemapper::sources::osm::extracts_for;
use freemapper::sources::region::golden::legacy_facts;
use freemapper::sources::region::{
    Provider, Region, contains_point, provider_tiles, raster_region, reaches_beyond_coverage,
    regions_for, tile_centre_in, tile_touches,
};
use freemapper::tiles::{TileId, tiles_covering};
use serde::{Deserialize, Serialize};

/// `tests/golden/adapters/` at the repository root.
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/adapters")
}

fn blessing() -> bool {
    std::env::var_os("FREEMAPPER_BLESS").is_some()
}

/// Compares `actual` with the golden file `name`, or writes it when blessing.
fn check_golden(name: &str, actual: &str) {
    let path = golden_dir().join(name);
    if blessing() {
        std::fs::create_dir_all(path.parent().expect("golden file has a folder")).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; record it with FREEMAPPER_BLESS=1", path.display()));
    // Line endings are not part of the record (a Windows checkout may turn LF into CRLF).
    assert!(
        expected.replace("\r\n", "\n") == actual,
        "{name} differs from the recorded behaviour; a refactor must not change it"
    );
}

#[test]
fn g1_registry_data_is_unchanged() {
    let facts = legacy_facts();
    assert_eq!(facts.len(), 11);
    let mut text = serde_json::to_string_pretty(&facts).unwrap();
    text.push('\n');
    check_golden("g1-regions.json", &text);
}

// Layer G2: geometry and dispatch. What the region functions answer for 211 boxes: the eleven
// example areas and 200 boxes on borders and coasts, where the answers change.

/// `examples/` at the repository root.
fn examples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// One box of G2 in EPSG:25833, in whole metres, so the file reads the same on every system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GoldenBox {
    name: String,
    min_e: i64,
    min_n: i64,
    max_e: i64,
    max_n: i64,
}

impl GoldenBox {
    fn bbox(&self) -> BoundingBox {
        BoundingBox {
            min_e: self.min_e as f64,
            min_n: self.min_n as f64,
            max_e: self.max_e as f64,
            max_n: self.max_n as f64,
        }
    }
}

/// Ten borders with four points each (longitude, latitude), roughly on the line. The generator
/// moves each point to the nearest change of region before it places boxes around it.
const BORDERS: [(&str, [(f64, f64); 4]); 10] = [
    (
        "bb-be",
        [
            (13.090, 52.413),
            (13.500, 52.660),
            (13.740, 52.430),
            (13.410, 52.375),
        ],
    ),
    (
        "st-ni",
        [
            (11.070, 52.210),
            (10.600, 51.800),
            (10.980, 52.430),
            (11.270, 52.870),
        ],
    ),
    (
        "ni-hb",
        [
            (8.780, 53.010),
            (8.980, 53.060),
            (8.610, 53.165),
            (8.650, 53.550),
        ],
    ),
    (
        "he-rp",
        [
            (8.270, 50.020),
            (8.030, 50.380),
            (7.910, 49.975),
            (8.380, 49.630),
        ],
    ),
    (
        "nw-ni",
        [
            (7.930, 52.280),
            (9.000, 52.270),
            (7.390, 52.310),
            (9.420, 51.800),
        ],
    ),
    (
        "mv-sh",
        [
            (10.820, 53.900),
            (10.900, 53.955),
            (10.830, 53.720),
            (10.930, 53.580),
        ],
    ),
    (
        "mv-bb",
        [
            (12.500, 53.250),
            (13.150, 53.220),
            (13.900, 53.430),
            (11.350, 53.120),
        ],
    ),
    (
        "sn-bb",
        [
            (13.500, 51.400),
            (14.350, 51.520),
            (14.700, 51.550),
            (13.220, 51.430),
        ],
    ),
    (
        "hh-sh",
        [
            (10.000, 53.680),
            (9.730, 53.590),
            (10.200, 53.660),
            (10.250, 53.520),
        ],
    ),
    (
        "hh-ni",
        [
            (9.950, 53.420),
            (9.800, 53.460),
            (10.300, 53.420),
            (9.750, 53.530),
        ],
    ),
];

/// Boxes placed around each border point.
const BOXES_PER_POINT: usize = 5;

/// SplitMix64: a fixed sequence, so the boxes are the same on every run.
struct Sequence(u64);

impl Sequence {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A whole number from `low` to `high`, both included.
    fn between(&mut self, low: i64, high: i64) -> i64 {
        low + (self.next() % (high - low + 1) as u64) as i64
    }
}

fn region_at(easting: f64, northing: f64) -> Option<Region> {
    Region::ALL
        .into_iter()
        .find(|region| contains_point(*region, easting, northing))
}

/// The nearest point east, north, west or south of the given one (EPSG:25833, 100 m steps, at
/// most 10 km away) where the region changes: a border between two regions, or the edge of the
/// supported area.
fn nearest_border(easting: f64, northing: f64) -> (i64, i64) {
    let home = region_at(easting, northing);
    for step in 1..=100 {
        for (de, dn) in [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)] {
            let e = (easting + de * 100.0 * f64::from(step)).round();
            let n = (northing + dn * 100.0 * f64::from(step)).round();
            let here = match home {
                Some(region) => contains_point(region, e, n).then_some(region),
                None => region_at(e, n),
            };
            if here != home {
                return (e as i64, n as i64);
            }
        }
    }
    panic!("no border within 10 km of {easting} {northing}");
}

/// The 211 boxes: the region boxes of the example projects, then the border boxes (150 m to
/// 1.5 km wide and high, their centre at most 700 m from the border point in each direction).
fn generated_boxes() -> Vec<GoldenBox> {
    let mut names: Vec<String> = std::fs::read_dir(examples_dir())
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.path().join("freemapper.toml").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut boxes: Vec<GoldenBox> = names
        .iter()
        .map(|name| {
            let project = ProjectFile::load(&examples_dir().join(name)).unwrap();
            let bbox = project.region_bbox().unwrap();
            GoldenBox {
                name: format!("example:{name}"),
                min_e: bbox.min_e.floor() as i64,
                min_n: bbox.min_n.floor() as i64,
                max_e: bbox.max_e.ceil() as i64,
                max_n: bbox.max_n.ceil() as i64,
            }
        })
        .collect();
    let mut sequence = Sequence(0x0082);
    for (border, points) in BORDERS {
        for (point, (lon, lat)) in points.into_iter().enumerate() {
            let near = utm33_forward(LatLon { lat, lon });
            let (e, n) = nearest_border(near.easting, near.northing);
            for index in 0..BOXES_PER_POINT {
                let centre_e = e + sequence.between(-700, 700);
                let centre_n = n + sequence.between(-700, 700);
                let half_width = sequence.between(75, 750);
                let half_height = sequence.between(75, 750);
                boxes.push(GoldenBox {
                    name: format!("border:{border}:{}", point * BOXES_PER_POINT + index),
                    min_e: centre_e - half_width,
                    min_n: centre_n - half_height,
                    max_e: centre_e + half_width,
                    max_n: centre_n + half_height,
                });
            }
        }
    }
    boxes
}

/// The boxes of `g2-boxes.json`. They are the input of G2, not its result: read from the file,
/// they are the same numbers on every system, whatever its maths library rounds.
fn recorded_boxes() -> Vec<GoldenBox> {
    let path = golden_dir().join("g2-boxes.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; record it with FREEMAPPER_BLESS=1", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn id_of<T: Serialize>(value: T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(id)) => id,
        other => panic!("an enum serialises to a string, not {other:?}"),
    }
}

/// The tiles of one provider's grid over a box.
#[derive(Debug, Serialize)]
struct ProviderTiles {
    provider: String,
    km: u32,
    tiles: Vec<String>,
}

/// What a zone-32 region says about the 1 km zone-32 tiles a box touches.
#[derive(Debug, Serialize)]
struct Zone32Tiles {
    region: String,
    touched: Vec<String>,
    centre_in: Vec<String>,
}

/// Everything the region functions answer for one box.
#[derive(Debug, Serialize)]
struct Geometry {
    name: String,
    regions: Vec<String>,
    raster_region: Option<String>,
    provider_tiles: Vec<ProviderTiles>,
    zone_32: Vec<Zone32Tiles>,
    beyond_coverage: bool,
    extracts: Vec<String>,
    warnings: Vec<String>,
}

/// The 1 km tiles of EPSG:25832 a box in EPSG:25833 touches, as `layers.rs` finds them for the
/// providers of zone 32.
fn zone_32_tiles(bbox: &BoundingBox) -> Vec<TileId> {
    let corners: Vec<ProjectedPoint> = [
        (bbox.min_e, bbox.min_n),
        (bbox.max_e, bbox.min_n),
        (bbox.max_e, bbox.max_n),
        (bbox.min_e, bbox.max_n),
    ]
    .into_iter()
    .filter_map(|(easting, northing)| {
        Crs::Utm32.reproject(Crs::Utm33, ProjectedPoint { easting, northing })
    })
    .collect();
    let fold = |pick: fn(&ProjectedPoint) -> f64, init: f64, f: fn(f64, f64) -> f64| {
        corners.iter().map(pick).fold(init, f)
    };
    tiles_covering(&BoundingBox {
        min_e: fold(|p| p.easting, f64::INFINITY, f64::min),
        min_n: fold(|p| p.northing, f64::INFINITY, f64::min),
        max_e: fold(|p| p.easting, f64::NEG_INFINITY, f64::max),
        max_n: fold(|p| p.northing, f64::NEG_INFINITY, f64::max),
    })
}

fn names(tiles: impl IntoIterator<Item = TileId>) -> Vec<String> {
    tiles.into_iter().map(|tile| tile.name()).collect()
}

fn geometry_of(golden: &GoldenBox) -> Geometry {
    let bbox = golden.bbox();
    let mut providers: Vec<Provider> = Vec::new();
    for region in Region::ALL {
        if !providers.contains(&region.provider()) {
            providers.push(region.provider());
        }
    }
    let mut tiles = Vec::new();
    for provider in providers {
        for km in [1, 2] {
            let found = provider_tiles(&bbox, provider, km);
            if !found.is_empty() {
                tiles.push(ProviderTiles {
                    provider: id_of(provider),
                    km,
                    tiles: names(found),
                });
            }
        }
    }
    let zone_32_tiles = zone_32_tiles(&bbox);
    // Only the regions of the box are asked, as the fetch does; `tile_centre_in` is slow.
    let zone_32 = regions_for(&bbox)
        .into_iter()
        .filter(|region| region.crs() == Crs::Utm32)
        .map(|region| Zone32Tiles {
            region: id_of(region),
            touched: names(
                zone_32_tiles
                    .iter()
                    .copied()
                    .filter(|t| tile_touches(*t, region)),
            ),
            centre_in: names(
                zone_32_tiles
                    .iter()
                    .copied()
                    .filter(|t| tile_centre_in(*t, region)),
            ),
        })
        .filter(|tiles| !tiles.touched.is_empty() || !tiles.centre_in.is_empty())
        .collect();
    let mut warnings = Vec::new();
    let extracts = extracts_for(&bbox, &mut |note| warnings.push(note.to_string()))
        .into_iter()
        .map(|source| source.label().to_string())
        .collect();
    Geometry {
        name: golden.name.clone(),
        regions: regions_for(&bbox).into_iter().map(id_of).collect(),
        raster_region: raster_region(&bbox).map(id_of),
        provider_tiles: tiles,
        zone_32,
        beyond_coverage: reaches_beyond_coverage(&bbox),
        extracts,
        warnings,
    }
}

fn pretty<T: Serialize>(value: &T) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap();
    text.push('\n');
    text
}

#[test]
fn g2_geometry_is_unchanged() {
    if blessing() {
        check_golden("g2-boxes.json", &pretty(&generated_boxes()));
    }
    let boxes = recorded_boxes();
    assert_eq!(boxes.len(), 211);
    let geometry: Vec<Geometry> = boxes.iter().map(geometry_of).collect();
    // The border boxes are worth their name only if many of them see two regions or the edge.
    let on_a_border = geometry
        .iter()
        .filter(|g| g.regions.len() > 1 || g.beyond_coverage)
        .count();
    assert!(
        on_a_border >= 120,
        "only {on_a_border} of 211 boxes lie on a border"
    );
    check_golden("g2-geometry.json", &pretty(&geometry));
}

/// The generator still gives the recorded boxes. A metre of difference is allowed: the border
/// points are projected with the maths library of the system.
#[test]
fn g2_boxes_come_from_the_generator() {
    if blessing() {
        // The file is being written by `g2_geometry_is_unchanged`.
        return;
    }
    let (generated, recorded) = (generated_boxes(), recorded_boxes());
    assert_eq!(generated.len(), recorded.len());
    for (new, old) in generated.iter().zip(&recorded) {
        assert_eq!(new.name, old.name);
        let moved = [
            new.min_e - old.min_e,
            new.min_n - old.min_n,
            new.max_e - old.max_e,
            new.max_n - old.max_n,
        ];
        assert!(moved.iter().all(|d| d.abs() <= 1), "{new:?} is not {old:?}");
    }
}

/// What the 211 boxes took in a release build of v0.12.0 on the maintainer's machine (Apple M
/// series, 2026-11-11), in milliseconds: the best of five runs.
const G2_MEASURED_MS: u128 = 1100;

/// The registry must not make the dispatch slower than the enums were: the 211 boxes finish
/// within twice the measured time. Run by hand after the registry takes over (plan C2, C14), on
/// the machine the measurement comes from, never on a shared CI runner:
/// `cargo test --release -p freemapper --test adapters_golden g2_dispatch -- --ignored --nocapture`
#[test]
#[ignore = "a timing, run by hand in a release build"]
fn g2_dispatch_is_fast_enough() {
    let boxes = recorded_boxes();
    // Once unmeasured, so the boundaries are parsed before the clock starts.
    boxes
        .iter()
        .take(11)
        .for_each(|golden| drop(geometry_of(golden)));
    // The best of five runs: other work on the machine only ever adds time.
    let took = (0..5)
        .map(|_| {
            let start = std::time::Instant::now();
            assert_eq!(boxes.iter().map(geometry_of).count(), 211);
            start.elapsed().as_millis()
        })
        .min()
        .expect("five runs");
    eprintln!("211 boxes in {took} ms (measured: {G2_MEASURED_MS} ms)");
    assert!(
        took <= 2 * G2_MEASURED_MS,
        "{took} ms is more than twice the {G2_MEASURED_MS} ms of v0.12.0"
    );
}
