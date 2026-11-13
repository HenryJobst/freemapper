//! The golden net of the region adapter migration (ADR 0082, plan stage A). Five layers, all
//! offline, recorded from the v0.12.0 behaviour. A refactor must leave every file under
//! `tests/golden/adapters/` as it is; `FREEMAPPER_BLESS=1` rewrites them on purpose, see
//! `tests/golden/adapters/README.md`.

use std::path::{Path, PathBuf};

use freemapper::area::MapArea;
use freemapper::clock;
use freemapper::crs::{BoundingBox, Crs, LatLon, ProjectedPoint, utm33_forward};
use freemapper::fetch::{ByteRange, Cache, FetchError, FetchMode, Fetcher, HttpFetcher};
use freemapper::layers::estimate_download;
use freemapper::project::{
    MapKind, PdfSources, ProjectFile, derive_project, export_candidates_pdf_from, export_ocad_from,
    export_pdf_from, export_starter_project_from, fetch_project, init_project, starter_documents,
};
use freemapper::sources::osm::extracts_for;
use freemapper::sources::region::golden::{inside, legacy_facts};
use freemapper::sources::region::ids::{IdFacts as _, regions};
use freemapper::sources::region::{
    contains_point, provider_tiles, raster_region, reaches_beyond_coverage, regions_for,
    tile_centre_in, tile_touches,
};
use freemapper::tiles::{TileId, tiles_covering};
use freemapper_adapter_api::records::AdapterId;
use serde::{Deserialize, Serialize};

#[path = "support/replay.rs"]
mod replay;

use replay::CutOff;

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
    let expected = expected.replace("\r\n", "\n");
    if expected == actual {
        return;
    }
    let (mut recorded, mut now) = (expected.lines(), actual.lines());
    let mut line = 1;
    let (was, is) = loop {
        match (recorded.next(), now.next()) {
            (was, is) if was != is => break (was, is),
            _ => line += 1,
        }
    };
    panic!(
        "{name} differs from the recorded behaviour in line {line}; a refactor must not change \
         it\n  recorded: {}\n  now:      {}",
        was.unwrap_or("(the file ends)"),
        is.unwrap_or("(the output ends)")
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

/// Neighbouring states whose ALKIS objects one map can hold: the first is the state of the map
/// area, the others delivered objects too.
const CREDIT_NEIGHBOURS: [&[&str]; 21] = [
    &["brandenburg", "berlin"],
    &["berlin", "brandenburg"],
    &["brandenburg", "saxony"],
    &["brandenburg", "saxonyanhalt"],
    &["brandenburg", "mecklenburgvorpommern"],
    &["brandenburg", "niedersachsen"],
    &["saxony", "saxonyanhalt"],
    &["saxonyanhalt", "niedersachsen"],
    &["hamburg", "schleswigholstein"],
    &["hamburg", "niedersachsen"],
    &["hamburg", "schleswigholstein", "niedersachsen"],
    &["schleswigholstein", "mecklenburgvorpommern"],
    &["schleswigholstein", "niedersachsen"],
    &["mecklenburgvorpommern", "niedersachsen"],
    &[
        "niedersachsen",
        "schleswigholstein",
        "mecklenburgvorpommern",
    ],
    &["hessen", "nordrheinwestfalen"],
    &["hessen", "niedersachsen"],
    &["hessen", "rheinlandpfalz"],
    &["nordrheinwestfalen", "niedersachsen"],
    &["nordrheinwestfalen", "rheinlandpfalz"],
    &["rheinlandpfalz", "hessen", "nordrheinwestfalen"],
];

/// The credit lines of the project notes and of the map sheet: per state for two years and the
/// three record combinations (rasters only, ALKIS only, both), and for neighbouring states in
/// one map. The lines move from the ten attribution functions into the adapters (plan C4); the
/// texts, the year, the order and the licence notes must stay.
#[test]
fn g1_credit_lines_match_for_all_year_and_record_combinations() {
    use freemapper::project::golden::{Held, credits};
    let mut rows = Vec::new();
    let mut row = |regions: &[AdapterId], year: i32, held: Held| {
        let (lon, lat) = golden_centre(&regions[0]);
        let lines = credits(&regions[0], lon, lat, year, held, &regions[1..]);
        rows.push(serde_json::json!({
            "regions": regions.iter().map(id_of).collect::<Vec<_>>(),
            "year": year,
            "records": match held {
                Held::Rasters => "rasters",
                Held::Alkis => "alkis",
                Held::Both => "both",
            },
            "notes": lines.notes,
            "sheet": lines.sheet,
        }));
    };
    for region in regions::LEGACY {
        for year in [2025, 2026] {
            for held in [Held::Rasters, Held::Alkis, Held::Both] {
                row(std::slice::from_ref(&region), year, held);
            }
        }
    }
    for ids in CREDIT_NEIGHBOURS {
        let regions: Vec<AdapterId> = ids.iter().map(|id| region_of(id)).collect();
        for held in [Held::Alkis, Held::Both] {
            row(&regions, 2026, held);
        }
    }
    assert_eq!(rows.len(), 11 * 6 + 21 * 2);
    check_golden("g1-credits.json", &pretty(&rows));
}

/// The service constants of the states, as they print and as the URLs they ask for three
/// sample areas. The service structs change how they hold their texts (plan B4: owned fields,
/// so a manifest can build a service at run time); what they ask must stay the same.
#[test]
fn g1_service_constants_are_unchanged() {
    use freemapper::raster::GridSpec;
    use freemapper::sources::{alkis_nas, trees};
    use freemapper_adapters_de::{
        geobasis_nrw, geosn, hvbg, laiv_mv, lgln, lgv, lvermgeo, lvermgeo_rp, lvermgeo_sh,
    };
    use freemapper_sources::wfs as alkis_wfs;
    use std::fmt::Write as _;
    let boxes = [
        (400_000.0, 5_800_000.0, 400_300.0, 5_800_200.0),
        (366_200.4, 5_807_100.6, 367_100.2, 5_807_700.9),
        (600_000.0, 5_600_000.0, 601_500.0, 5_601_000.0),
    ]
    .map(|(min_e, min_n, max_e, max_n)| BoundingBox {
        min_e,
        min_n,
        max_e,
        max_n,
    });
    let grids = boxes.map(|b| GridSpec {
        west: b.min_e,
        north: b.max_n,
        pixel_size: 0.2,
        width: 1000,
        height: 800,
    });
    let mut text = String::new();
    let mut add = |name: &str, service: String, urls: Vec<String>| {
        writeln!(text, "{name}: {service}").unwrap();
        for url in urls {
            writeln!(text, "  {url}").unwrap();
        }
    };
    for (name, service) in [
        ("lgv::WMS", lgv::WMS),
        ("lvermgeo_rp::WMS", lvermgeo_rp::WMS),
        ("lvermgeo_sh::WMS", lvermgeo_sh::WMS),
        ("hvbg::WMS", hvbg::WMS),
        ("lgln::WMS", lgln::WMS),
        ("geosn::WMS", geosn::WMS),
        ("geobasis_nrw::WMS", geobasis_nrw::WMS),
        ("lvermgeo::WMS", lvermgeo::WMS),
        ("wms::K5_WMS", freemapper_adapters_de::berlin::K5_WMS),
    ] {
        let urls = grids.iter().map(|g| service.piece_url(g)).collect();
        add(name, format!("{service:?}"), urls);
    }
    for (name, service) in [
        ("lgln::DGM1_WCS", lgln::DGM1_WCS),
        ("lgln::DOM1_WCS", lgln::DOM1_WCS),
    ] {
        let urls = boxes.iter().map(|b| service.get_coverage_url(b)).collect();
        add(name, format!("{service:?}"), urls);
    }
    let dop = laiv_mv::DOP;
    let urls = grids.iter().map(|g| dop.piece_url(g)).collect();
    add("laiv_mv::DOP", format!("{dop:?}"), urls);
    for (name, service) in [
        ("alkis_wfs::GEOSN", alkis_wfs::GEOSN),
        ("alkis_wfs::LVERMGEO", alkis_wfs::LVERMGEO),
        ("alkis_wfs::HVBG", alkis_wfs::HVBG),
        ("alkis_wfs::GEOBASIS_NRW", alkis_wfs::GEOBASIS_NRW),
        ("alkis_wfs::LGLN", alkis_wfs::LGLN),
        ("alkis_wfs::LVERMGEO_RP", alkis_wfs::LVERMGEO_RP),
        ("alkis_wfs::LVERMGEO_SH", alkis_wfs::LVERMGEO_SH),
        ("alkis_wfs::LGV", alkis_wfs::LGV),
        ("alkis_wfs::LAIV_MV", alkis_wfs::LAIV_MV),
    ] {
        let mut urls = Vec::new();
        for b in &boxes {
            urls.push(service.wfs_url(&service.buildings_type, b, 0));
            urls.push(service.wfs_url(&service.land_use_type, b, 10_000));
        }
        add(name, format!("{service:?}"), urls);
    }
    let nas = alkis_nas::LGB_NAS;
    let layer = alkis_nas::NAS_LAYERS[0].type_name;
    let urls = boxes
        .iter()
        .map(|b| nas.get_feature_url(layer, b))
        .collect();
    add("alkis_nas::LGB_NAS", format!("{nas:?}"), urls);
    let potsdam = trees::POTSDAM;
    let urls = boxes
        .iter()
        .flat_map(|b| potsdam.endpoints.iter().map(|e| potsdam.url(e, b)))
        .map(|url| url.unwrap_or_else(|| "(outside)".to_string()))
        .collect();
    add("trees::POTSDAM", format!("{potsdam:?}"), urls);
    check_golden("g1-services.txt", &text);
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

fn region_at(easting: f64, northing: f64) -> Option<AdapterId> {
    regions::LEGACY
        .into_iter()
        .find(|region| contains_point(region, easting, northing))
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
            let here = match &home {
                Some(region) => contains_point(region, e, n).then(|| region.clone()),
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
        .filter_map(|name| {
            let project = ProjectFile::load(&examples_dir().join(name)).unwrap();
            let bbox = project.region_bbox().unwrap();
            // The net covers the states that were code before: the example of a state added
            // since (Saarland) is no box of it.
            let legacy =
                raster_region(&bbox).is_some_and(|region| regions::LEGACY.contains(&region));
            legacy.then(|| GoldenBox {
                name: format!("example:{name}"),
                min_e: bbox.min_e.floor() as i64,
                min_n: bbox.min_n.floor() as i64,
                max_e: bbox.max_e.ceil() as i64,
                max_n: bbox.max_n.ceil() as i64,
            })
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
    let mut providers: Vec<AdapterId> = Vec::new();
    for region in regions::LEGACY {
        if !providers.contains(&region.provider()) {
            providers.push(region.provider());
        }
    }
    let mut tiles = Vec::new();
    for provider in providers {
        for km in [1, 2] {
            let found = provider_tiles(&bbox, &provider, km);
            if !found.is_empty() {
                tiles.push(ProviderTiles {
                    provider: id_of(&provider),
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
            region: id_of(&region),
            touched: names(
                zone_32_tiles
                    .iter()
                    .copied()
                    .filter(|t| tile_touches(*t, &region)),
            ),
            centre_in: names(
                zone_32_tiles
                    .iter()
                    .copied()
                    .filter(|t| tile_centre_in(*t, &region)),
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

/// The registry must not make the dispatch slower than the enums of v0.12.0 were: the 211 boxes finish
/// within twice the measured time. Run by hand after the registry takes over (plan C2, C14), on
/// the machine the measurement comes from, never on a shared CI runner:
/// `cargo test --release -p freemapper --test adapters_golden g2_dispatch -- --ignored --nocapture`
#[test]
#[ignore = "a timing, run by hand in a release build"]
fn g2_dispatch_is_fast_enough() {
    if cfg!(debug_assertions) {
        // `scripts/golden.sh test` runs the ignored tests too, in a debug build, where the
        // time says nothing.
        eprintln!("not timed: a debug build");
        return;
    }
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

// Layer G3: the requests of an online fetch per state, replayed from the fixture store.

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn replay_returns_recorded_bytes_and_logs_requests() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("store");
    std::fs::create_dir_all(store.join("responses")).unwrap();
    let keep = |bytes: &[u8]| {
        let hash = sha256_hex(bytes);
        std::fs::write(store.join("responses").join(format!("{hash}.bin")), bytes).unwrap();
        hash
    };
    let (whole, part) = (keep(b"the whole answer"), keep(b"wh"));
    let index = dir.path().join("index.toml");
    std::fs::write(
        &index,
        format!(
            "[[response]]\nmethod = \"GET\"\nurl = \"https://example.invalid/a\"\n\
             body = \"{whole}\"\nbytes = 16\n\n\
             [[response]]\nmethod = \"GET\"\nurl = \"https://example.invalid/b\"\n\
             range = \"bytes=4-5\"\nbody = \"{part}\"\nbytes = 2\n\n\
             [[response]]\nmethod = \"HEAD\"\nurl = \"https://example.invalid/a\"\n\
             last_modified = \"2025-03-03\"\n\n\
             [[response]]\nmethod = \"GET\"\nurl = \"https://example.invalid/gone\"\n\
             error = \"not-found\"\n"
        ),
    )
    .unwrap();
    let fetcher = replay::replay(&index, &store);
    assert_eq!(
        fetcher.get("https://example.invalid/a").unwrap(),
        b"the whole answer"
    );
    let range = ByteRange::From { start: 4, len: 2 };
    assert_eq!(
        fetcher
            .get_range("https://example.invalid/b", range)
            .unwrap(),
        b"wh"
    );
    assert_eq!(
        fetcher.last_modified("https://example.invalid/a").unwrap(),
        chrono::NaiveDate::from_ymd_opt(2025, 3, 3)
    );
    let gone = fetcher.get("https://example.invalid/gone").unwrap_err();
    assert!(matches!(gone, FetchError::NotFound(_)), "{gone:?}");
    // A request the recording does not know fails like a server error and is logged.
    let unknown = fetcher.get("https://example.invalid/new").unwrap_err();
    assert!(
        matches!(&unknown, FetchError::Http { message, .. } if message.contains("404")),
        "{unknown:?}"
    );
    // An OSM extract is answered with the empty one, whatever the index says.
    let extract = dir.path().join("x.osm.pbf.part");
    let url = "https://example.invalid/x-latest.osm.pbf";
    fetcher
        .download_to(url, &extract, 1 << 20, &mut |_| {})
        .unwrap();
    let bytes = std::fs::read(&extract).unwrap();
    assert!(
        !bytes.is_empty() && bytes.len() < 300,
        "{} bytes",
        bytes.len()
    );
    assert_eq!(
        fetcher.trace(),
        [
            "GET https://example.invalid/a",
            "GET https://example.invalid/b [bytes=4-5]",
            "HEAD https://example.invalid/a",
            "GET https://example.invalid/gone",
            "GET https://example.invalid/new",
            "GET https://example.invalid/x-latest.osm.pbf [to file]",
        ]
    );
}

/// The version of the fixture store this tree needs (`store.lock`).
fn store_version() -> u32 {
    #[derive(Deserialize)]
    struct Lock {
        version: u32,
    }
    let path = golden_dir().join("store.lock");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    toml::from_str::<Lock>(&text).unwrap().version
}

/// Where the fixture store lies: `FREEMAPPER_GOLDEN_STORE`, or `target/golden-adapters/<version>/`
/// at the repository root, where `scripts/golden.sh fetch` unpacks it.
fn store_dir() -> PathBuf {
    match std::env::var_os("FREEMAPPER_GOLDEN_STORE") {
        Some(dir) => PathBuf::from(dir),
        None => Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/golden-adapters")
            .join(store_version().to_string()),
    }
}

/// The store, fetched. Without it G3 and G4 fail; they never pass by skipping.
fn fetched_store() -> PathBuf {
    let dir = store_dir();
    assert!(
        dir.join("responses").is_dir(),
        "needs the fixture store: scripts/golden.sh test ({} is missing)",
        dir.display()
    );
    dir
}

/// The instant every golden run happens at.
fn golden_noon() -> chrono::NaiveDateTime {
    chrono::NaiveDate::from_ymd_opt(2026, 11, 11)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap()
}

fn region_of(id: &str) -> AdapterId {
    regions::LEGACY
        .into_iter()
        .find(|region| id_of(region) == id)
        .unwrap_or_else(|| panic!("no region {id:?}"))
}

/// The centre of the golden area of a region (longitude, latitude): the centre of its example
/// area. In four states that centre lies so close to a line of the 1 km grid that the area
/// would need two or four tiles (Schleswig-Holstein: four surface model tiles of 105 MB each);
/// there the centre is moved by 90 to 190 m, so that the area with its buffer stays in one tile.
/// The numbers are written out, not computed: the area must be the same on every system.
fn golden_centre(region: &AdapterId) -> (f64, f64) {
    match region.as_str() {
        "berlin" => (13.3829, 52.488),
        "schleswigholstein" => (10.6849, 53.8648),
        "nordrheinwestfalen" => (6.2401, 50.5534),
        "rheinlandpfalz" => (7.1638, 50.1443),
        _ => inside(region),
    }
}

/// The golden area of a region: about 150 m square around [`golden_centre`].
fn golden_area(region: &AdapterId) -> MapArea {
    let (lon, lat) = golden_centre(region);
    let (dlon, dlat) = (0.001_1, 0.000_7);
    let ring = [
        [lon - dlon, lat - dlat],
        [lon + dlon, lat - dlat],
        [lon + dlon, lat + dlat],
        [lon - dlon, lat + dlat],
        [lon - dlon, lat - dlat],
    ];
    let polygon = serde_json::json!({"type": "Polygon", "coordinates": [ring]});
    MapArea::from_geojson(&polygon.to_string()).unwrap()
}

/// A new sprint project over the golden area of `region` in the empty folder `dir`.
fn init_golden_project(dir: &Path, region: &AdapterId) {
    std::fs::create_dir_all(dir).unwrap();
    let date = golden_noon().date();
    init_project(dir, &golden_area(region), MapKind::Sprint, None, date, None).unwrap();
}

/// `line` without the folders of this run, which have another name every time.
fn without_run_folders(line: &str, dir: &Path, cache: &Path) -> String {
    let mut scrubbed = line.to_string();
    for (folder, name) in [(dir, "<project>"), (cache, "<cache>")] {
        let resolved = std::fs::canonicalize(folder).unwrap_or_else(|_| folder.to_path_buf());
        for spelling in [resolved.as_path(), folder] {
            scrubbed = scrubbed.replace(&spelling.display().to_string(), name);
        }
    }
    if scrubbed != line {
        scrubbed = scrubbed.replace('\\', "/");
    }
    scrubbed
}

/// A project fetched online from the recording of one state.
struct Replayed {
    /// Holds the project folder and the cache; removed when dropped.
    _root: tempfile::TempDir,
    /// The project folder, named after the state, so the map files are too.
    dir: PathBuf,
    cache: Cache,
    /// The estimate, then the requests and warnings in their order, then how the fetch ended.
    lines: Vec<String>,
}

/// Fetches the golden project of `id` into an empty folder with an empty cache, under the fixed
/// clock, every request answered from the store.
fn replayed_fetch(id: &str) -> Replayed {
    replayed_fetch_until(id, None)
}

/// [`replayed_fetch`] that loses the network at the first request whose URL contains `cut`.
fn replayed_fetch_until(id: &str, cut: Option<&str>) -> Replayed {
    let store = fetched_store();
    let root = tempfile::tempdir().unwrap();
    let (dir, cache_dir) = (root.path().join(id), root.path().join("cache"));
    let cache = Cache::new(cache_dir.clone());
    let fetcher = replay::replay(&golden_dir().join(id).join("index.toml"), &store);
    let log = fetcher.log();
    let add = |line: String| log.lock().unwrap().push(line);
    clock::with_fixed(golden_noon(), || {
        init_golden_project(&dir, &region_of(id));
        let estimate = estimate_download(&dir, &cache, clock::today()).unwrap();
        for item in &estimate.items {
            add(format!(
                "estimate: {} | {:?} | {:.3} MB | cached: {}",
                item.what, item.part, item.megabytes, item.cached
            ));
        }
        let cut_off = cut.map(|marker| CutOff::new(&fetcher, marker));
        let through: &dyn Fetcher = match &cut_off {
            Some(cut_off) => cut_off,
            None => &fetcher,
        };
        let fetched = fetch_project(&dir, through, &cache, FetchMode::Online, &mut |note| {
            if note.is_warning() {
                // The text of a warning starts with "warning:".
                add(note.to_string());
            }
        });
        match fetched {
            Ok(_) => add("result: fetched".to_string()),
            Err(e) => add(format!("result: {e}")),
        }
    });
    let lines = fetcher
        .trace()
        .iter()
        .map(|line| without_run_folders(line, &dir, &cache_dir))
        .collect();
    Replayed {
        _root: root,
        dir,
        cache,
        lines,
    }
}

fn g3_trace_of(id: &str) {
    let replayed = replayed_fetch(id);
    let mut text = replayed.lines.join("\n");
    text.push('\n');
    check_golden(&format!("{id}/trace.txt"), &text);
    // Hesse and Schleswig-Holstein pick the newest kept list by modification time. A replay
    // writes its files within one second, so the pick must never be between two lists.
    let lists: Vec<String> = match id {
        "hessen" => file_names_under(&replayed.cache.provider_dir("hvbg"))
            .into_iter()
            .filter(|name| name.ends_with(".xlsx"))
            .map(|_| "metadata list".to_string())
            .collect(),
        "schleswigholstein" => {
            file_names_under(&replayed.cache.provider_dir("lvermgeo_sh").join("overview"))
                .into_iter()
                .filter_map(|name| name.split_once('_').map(|(kind, _)| kind.to_string()))
                .collect()
        }
        _ => return,
    };
    assert!(!lists.is_empty(), "{id} kept no list");
    let mut kinds = lists.clone();
    kinds.sort();
    kinds.dedup();
    assert_eq!(
        kinds.len(),
        lists.len(),
        "{id} keeps two lists of one kind: {lists:?}"
    );
}

/// The names of the files in `dir` and its subfolders.
fn file_names_under(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        if entry.path().is_dir() {
            names.extend(file_names_under(&entry.path()));
        } else {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names
}

/// The states the net still records (G3, G4), one of each kind of fetch: Brandenburg (tiles in
/// the cache), Berlin (the code adapter), Saxony-Anhalt (per-area services, declared by its
/// manifest alone) and Hesse (archives read by range requests). The other seven left the net
/// after the migration to adapters, which they were recorded to prove: every later change to a
/// request would have paid for them with a new recording.
const RECORDED: [&str; 4] = ["brandenburg", "berlin", "saxonyanhalt", "hessen"];

/// The folders of the net are those of [`RECORDED`], and each has its three files.
#[test]
fn g3_the_net_holds_the_recorded_states() {
    let mut folders: Vec<String> = std::fs::read_dir(golden_dir())
        .unwrap()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    folders.sort();
    let mut recorded = RECORDED.map(str::to_string).to_vec();
    recorded.sort();
    assert_eq!(folders, recorded);
    for id in RECORDED {
        for file in ["index.toml", "trace.txt", "expected.sha256"] {
            assert!(golden_dir().join(id).join(file).is_file(), "{id}/{file}");
        }
    }
}

/// One test per state in the module `$layer`, each calling `$run` with the state's id.
macro_rules! per_state {
    ($layer:ident, $run:path, $($id:ident),+) => {
        mod $layer {
            $(
                #[test]
                #[ignore = "needs the fixture store: scripts/golden.sh test"]
                fn $id() {
                    $run(stringify!($id));
                }
            )+
        }
    };
}

per_state!(
    g3_trace,
    crate::g3_trace_of,
    brandenburg,
    berlin,
    saxonyanhalt,
    hessen
);

/// The download estimate of a project as a golden row: the tiles, every item (text, megabytes,
/// cached flag, part, store), the megabytes still to download and the parts stored in place.
fn estimate_row(id: &str, dir: &Path, cache: &Cache) -> serde_json::Value {
    let estimate = estimate_download(dir, cache, clock::today()).unwrap();
    serde_json::json!({
        "region": id,
        "tiles": estimate.tiles,
        "items": estimate
            .items
            .iter()
            .map(|item| format!(
                "{} | {:.3} MB | cached: {} | {:?} | {:?}",
                item.what, item.megabytes, item.cached, item.part, item.store
            ))
            .collect::<Vec<_>>(),
        "total_megabytes": format!("{:.3}", estimate.total_megabytes),
        "in_place": estimate
            .in_place
            .iter()
            .map(|part| format!("{part:?}"))
            .collect::<Vec<_>>(),
    })
}

/// The estimate of a new project in every state, with an empty cache, under the fixed clock.
/// The ten `*_items` functions of `layers.rs` move into the adapters (plan C5); the items must
/// stay.
#[test]
fn g2_estimate_items_equal_the_old_ones_for_the_example_areas() {
    let rows: Vec<serde_json::Value> = regions::LEGACY
        .into_iter()
        .map(|region| {
            let root = tempfile::tempdir().unwrap();
            let (dir, cache) = (root.path().join("project"), root.path().join("cache"));
            let cache = Cache::new(cache);
            clock::with_fixed(golden_noon(), || {
                init_golden_project(&dir, &region);
                estimate_row(&id_of(&region), &dir, &cache)
            })
        })
        .collect();
    assert_eq!(rows.len(), 11);
    check_golden("g2-estimates.json", &pretty(&rows));
}

/// The estimate of every recorded state after its fetch: the tiles and lists are in the cache and
/// the rasters in the project folder, so the items say what is cached and what is replaced in place.
/// Each provider finds its files in the cache under its own names; the move of plan C5 must
/// keep them.
#[test]
#[ignore = "needs the fixture store: scripts/golden.sh test"]
fn g3_estimates_after_the_fetch_are_unchanged() {
    let rows: Vec<serde_json::Value> = RECORDED
        .into_iter()
        .map(|id| {
            let replayed = replayed_fetch(id);
            assert_eq!(
                replayed.lines.last().map(String::as_str),
                Some("result: fetched"),
                "{id}"
            );
            clock::with_fixed(golden_noon(), || {
                estimate_row(id, &replayed.dir, &replayed.cache)
            })
        })
        .collect();
    check_golden("g3-estimates.json", &pretty(&rows));
}

/// One interrupted fetch: what was asked, what was said, and the project file left behind.
#[derive(Debug, Serialize)]
struct Interrupted {
    case: String,
    /// The requests and warnings in their order, then how the fetch ended.
    lines: Vec<String>,
    /// `freemapper.toml` after the fetch, line by line.
    project_file: Vec<String>,
}

/// A fetch that loses the network halfway must leave the project file with exactly the parts
/// that finished, so that the next fetch goes on from there. Three states (per area, tiles, zip
/// ranges), each cut at three places: after the orthophoto, at the first request for the surface
/// model, and at the last request. The raster fetch gets other parameters in plan task B5; what
/// it saves and when must stay.
#[test]
#[ignore = "needs the fixture store: scripts/golden.sh test"]
fn g3_interrupted_fetches_keep_what_finished() {
    // Per state: the part of a URL that names the first request after the orthophoto, and the
    // first request for the surface model.
    let states = [
        ("saxonyanhalt", "ows_WCS_ST_DGM1", "ows_WCS_ST_DOM1"),
        ("brandenburg", "bb_dgm_aktualitaet.csv", "/ndom/NDOM_"),
        (
            "hessen",
            "Digitales+Gel%C3%A4ndemodell",
            "Digitales+Oberfl%C3%A4chenmodell",
        ),
    ];
    let mut cases = Vec::new();
    for (id, after_orthophoto, first_surface) in states {
        for (place, marker) in [
            ("after-orthophoto", after_orthophoto),
            ("first-surface", first_surface),
            ("last", ".osm.pbf"),
        ] {
            let replayed = replayed_fetch_until(id, Some(marker));
            let lines: Vec<String> = replayed
                .lines
                .iter()
                .filter(|line| !line.starts_with("estimate:"))
                .cloned()
                .collect();
            assert!(
                lines.iter().any(|line| line.ends_with("(no network)")),
                "{id}/{place}: no request names {marker:?}"
            );
            let project_file = std::fs::read_to_string(replayed.dir.join("freemapper.toml"))
                .unwrap()
                .lines()
                .map(str::to_string)
                .collect();
            cases.push(Interrupted {
                case: format!("{id}/{place}"),
                lines,
                project_file,
            });
        }
    }
    check_golden("g3-interrupted.json", &pretty(&cases));
}

// Layer G4: the whole pipeline per state. It goes on from the replayed fetch of G3, so every
// file the run reads was written by the run, and hashes every file the project folder holds
// afterwards: the stored data, the templates, the two maps, the OCAD project and the PDFs.

/// Every file under `dir` with its path relative to `dir` (`/` on every system), sorted.
fn files_under(dir: &Path) -> Vec<(String, PathBuf)> {
    fn walk(dir: &Path, root: &Path, found: &mut Vec<(String, PathBuf)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, root, found);
            } else {
                let relative = path.strip_prefix(root).unwrap();
                let name: Vec<String> = relative
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned())
                    .collect();
                found.push((name.join("/"), path));
            }
        }
    }
    let mut found = Vec::new();
    walk(dir, dir, &mut found);
    found.sort();
    found
}

/// Runs the pipeline on the golden project of `id` and returns its folder (kept alive by the
/// first value) with the hash of every file in it, as lines `<sha256>  <path>`.
fn golden_pipeline(id: &str) -> (Replayed, Vec<String>) {
    let quiet = &mut |_: freemapper::note::Note| {};
    let replayed = replayed_fetch(id);
    let dir = &replayed.dir;
    clock::with_fixed(golden_noon(), || {
        // An offline fetch on the result asks for nothing and leaves the project file alone.
        let project_file = dir.join("freemapper.toml");
        let before = std::fs::read(&project_file).unwrap();
        let store = fetched_store();
        let offline = replay::replay(&golden_dir().join(id).join("index.toml"), &store);
        fetch_project(dir, &offline, &replayed.cache, FetchMode::Offline, quiet).unwrap();
        assert_eq!(
            offline.trace(),
            Vec::<String>::new(),
            "an offline fetch asked the network"
        );
        assert!(
            std::fs::read(&project_file).unwrap() == before,
            "an offline fetch changed freemapper.toml"
        );
        // The replay answered the extract with the empty one. The candidates of the real
        // extract keep the OSM objects in the outputs.
        std::fs::copy(
            store.join(id).join("osm-candidates.geojson"),
            dir.join("data/osm-candidates.geojson"),
        )
        .unwrap();
        derive_project(dir, &replayed.cache, quiet).unwrap();
        let documents = starter_documents(dir).unwrap();
        export_starter_project_from(dir, &documents, true).unwrap();
        export_ocad_from(dir, &documents, true).unwrap();
        export_pdf_from(dir, &documents, false, PdfSources::Auto).unwrap();
        export_candidates_pdf_from(dir, &documents, false, PdfSources::Auto).unwrap();
    });
    let hashes = files_under(dir)
        .into_iter()
        .map(|(name, path)| format!("{}  {name}", sha256_hex(&std::fs::read(path).unwrap())))
        .collect();
    (replayed, hashes)
}

fn g4_pipeline_of(id: &str) {
    let (replayed, hashes) = golden_pipeline(id);
    // `FREEMAPPER_GOLDEN_KEEP=1` keeps every output in `target/golden-outputs/<id>/`, to look at
    // or to compare with those of another system.
    if std::env::var_os("FREEMAPPER_GOLDEN_KEEP").is_some() {
        let kept = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/golden-outputs")
            .join(id);
        for (file, path) in files_under(&replayed.dir) {
            let copy = kept.join(&file);
            std::fs::create_dir_all(copy.parent().expect("a folder")).unwrap();
            std::fs::copy(&path, &copy).unwrap();
        }
    }
    let name = format!("{id}/expected.sha256");
    let mut text = hashes.join("\n");
    text.push('\n');
    if blessing() {
        check_golden(&name, &text);
        return;
    }
    let path = golden_dir().join(&name);
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; record it with FREEMAPPER_BLESS=1", path.display()));
    let expected: Vec<&str> = expected.lines().collect();
    // Every file that differs, not only the first: the list says which kind of output moved.
    let mut differing: Vec<String> = Vec::new();
    for line in &hashes {
        if !expected.contains(&line.as_str()) {
            differing.push(format!("now:      {line}"));
        }
    }
    for line in &expected {
        if !hashes.iter().any(|now| now == line) {
            differing.push(format!("recorded: {line}"));
        }
    }
    if differing.is_empty() {
        return;
    }
    // The files that differ are kept, so they can be compared with those of another system
    // (CI uploads the folder as an artifact).
    let kept = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/golden-diff")
        .join(id);
    for (file, path) in files_under(&replayed.dir) {
        let recorded_hash = expected
            .iter()
            .find_map(|line| line.split_once("  ").filter(|(_, name)| *name == file));
        let now = sha256_hex(&std::fs::read(&path).unwrap());
        if recorded_hash.is_none_or(|(hash, _)| hash != now) {
            let copy = kept.join(&file);
            std::fs::create_dir_all(copy.parent().expect("a folder")).unwrap();
            std::fs::copy(&path, &copy).unwrap();
        }
    }
    panic!(
        "{name} differs from the recorded behaviour; a refactor must not change it\n  {}\n\
         the files are kept in {}",
        differing.join("\n  "),
        kept.display()
    );
}

per_state!(
    g4_pipeline,
    crate::g4_pipeline_of,
    brandenburg,
    berlin,
    saxonyanhalt,
    hessen
);

/// Copies the maintainer's cached OSM extracts of `region` into `cache`, so the recording does
/// not download 50 to 500 MB that the replay answers with the empty extract anyway.
fn seed_extracts(region: &AdapterId, cache: &Cache) {
    let Some(kept) = Cache::default_location() else {
        return;
    };
    std::fs::create_dir_all(cache.osm_dir()).unwrap();
    for extract in freemapper::sources::osm::golden::extracts_of(region) {
        for entry in std::fs::read_dir(kept.osm_dir())
            .into_iter()
            .flatten()
            .flatten()
        {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(&extract.prefix) && name.ends_with(".osm.pbf") {
                std::fs::copy(entry.path(), cache.osm_dir().join(&name)).unwrap();
                eprintln!("using the cached extract {name}");
            }
        }
    }
}

/// Records the online fetch of the state `FREEMAPPER_GOLDEN_RECORD` names: the index into the
/// repository, the answers into the store folder. Run through `scripts/golden.sh record <id>`.
/// No request carries personal data: the fetcher refuses a URL or a user agent with an `@`.
#[test]
#[ignore = "records from the network: scripts/golden.sh record <id>"]
fn record_state() {
    let Ok(id) = std::env::var("FREEMAPPER_GOLDEN_RECORD") else {
        eprintln!("nothing to record: FREEMAPPER_GOLDEN_RECORD names no state");
        return;
    };
    let region = region_of(&id);
    let store = store_dir();
    std::fs::create_dir_all(store.join(&id)).unwrap();
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join(&id);
    let cache = Cache::new(root.path().join("cache"));
    let index = golden_dir().join(&id).join("index.toml");
    let recorder = replay::record(HttpFetcher::new(), &index, &store);
    clock::with_fixed(golden_noon(), || {
        init_golden_project(&dir, &region);
        seed_extracts(&region, &cache);
        let fetched = fetch_project(&dir, &recorder, &cache, FetchMode::Online, &mut |note| {
            eprintln!("{note}");
        });
        if let Err(e) = fetched {
            eprintln!("the fetch failed and is recorded as it is: {e}");
        }
    });
    recorder.save().unwrap();
    // The candidates of the real extract: G4 puts them in place of the empty extract's.
    let candidates = dir.join("data/osm-candidates.geojson");
    if candidates.is_file() {
        std::fs::copy(&candidates, store.join(&id).join("osm-candidates.geojson")).unwrap();
    }
    // The store republishes data of the state and of OpenStreetMap.
    let (lon, lat) = inside(&region);
    let credit = freemapper::project::golden::credit_lines(&region, lon, lat, 2026, true);
    let mut attribution = format!(
        "Recorded answers of the open data services for the golden area of {id}, and OpenStreetMap \
         objects of the same area (osm-candidates.geojson, ODbL 1.0), for the tests of freemapper \
         (https://github.com/HenryJobst/freemapper).\n\n"
    );
    for line in credit {
        attribution.push_str(&line);
        attribution.push('\n');
    }
    std::fs::write(store.join(&id).join("ATTRIBUTION.txt"), attribution).unwrap();
    eprintln!(
        "recorded {} requests of {id}",
        recorder.log().lock().unwrap().len()
    );
}

// Layer G5: the persisted format. A project file written by v0.12.0 is read and written back
// byte for byte, and a provider freemapper does not know is named in the error.

#[test]
fn g5_examples_roundtrip() {
    let mut checked = 0;
    for entry in std::fs::read_dir(examples_dir()).unwrap() {
        let example = entry.unwrap().path();
        let Ok(original) = std::fs::read_to_string(example.join("freemapper.toml")) else {
            continue;
        };
        let project = ProjectFile::load(&example).unwrap();
        let copy = tempfile::tempdir().unwrap();
        project.save(copy.path()).unwrap();
        let saved = std::fs::read_to_string(copy.path().join("freemapper.toml")).unwrap();
        assert!(
            saved == original.replace("\r\n", "\n"),
            "{}: freemapper.toml is not written back as it was read",
            example.display()
        );
        checked += 1;
    }
    assert_eq!(checked, 13, "one example per state and the forest map");
}

/// A project of a colleague with an adapter that is not installed here: the error names the
/// adapter instead of a parse error.
#[test]
fn g5_unknown_provider_is_a_clear_error() {
    let example = examples_dir().join("brandenburg-sanssouci/freemapper.toml");
    let text = std::fs::read_to_string(example).unwrap();
    assert!(text.contains("provider = \"lgb\""));
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("freemapper.toml"),
        text.replace("provider = \"lgb\"", "provider = \"czfoo\""),
    )
    .unwrap();
    let error = ProjectFile::load(dir.path()).unwrap_err().to_string();
    assert!(
        error.contains("czfoo") && error.contains("adapter"),
        "{error}"
    );
}
