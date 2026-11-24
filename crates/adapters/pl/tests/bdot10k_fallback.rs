// SPDX-License-Identifier: AGPL-3.0-or-later

//! The fallback of the cadastre of Poland to BDOT10k (ADR 0178), through the adapter as the
//! engine calls it: EGiB comes first, BDOT10k fills only where EGiB gives nothing.

use std::io::Write;

use chrono::NaiveDate;
use freemapper_adapter_api::alkis::{AlkisPart, FetchArea};
use freemapper_adapter_api::cache::CacheDir;
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::fetch::FetchMode;
use freemapper_adapter_api::registry::Registry;
use freemapper_adapter_api::testing::MapFetcher;
use freemapper_georef::crs::{BoundingBox, Crs, LatLon};

const EGIB: &str = "https://mapy.geoportal.gov.pl/wss/service/PZGIK/EGIB/WFS/UslugaZbiorcza";
const INDEX: &str = "https://mapy.geoportal.gov.pl/wss/service/PZGIK/BDOT/WFS/PobieranieBDOT10k";
const ARCHIVE: &str = "https://opendata.geoportal.gov.pl/bdot10k/schemat2021/08/0805_GML.zip";
const POWIATY: &str = include_str!("fixtures/bdot10k/powiaty-slubice.xml");
const EGIB_BUILDINGS: &str = include_str!("fixtures/egib/budynki-slubice.xml");
const EMPTY: &str = r#"<wfs:FeatureCollection xmlns:ms="m" xmlns:gml="g" xmlns:wfs="w" numberReturned="0"></wfs:FeatureCollection>"#;

/// A 500 m box at Słubice in UTM 33, the CRS of the map there.
fn area() -> FetchArea {
    let at = |lat: f64, lon: f64| Crs::Utm33.forward(LatLon { lat, lon });
    let c = at(52.3515, 14.56);
    FetchArea::new(
        BoundingBox {
            min_e: c.easting - 250.0,
            min_n: c.northing - 250.0,
            max_e: c.easting + 250.0,
            max_n: c.northing + 250.0,
        },
        Crs::Utm33,
    )
}

fn archive() -> Vec<u8> {
    archive_with(true)
}

/// The archive of the powiat, with or without the file of the cemeteries.
fn archive_with(cemeteries: bool) -> Vec<u8> {
    let name = |c: &str| format!("PL.PZGiK.333.0805/BDOT10k/PL.PZGiK.333.0805__OT_{c}.xml");
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (class, text) in [
        ("BUBD_A", include_str!("fixtures/bdot10k/BUBD_A.xml")),
        ("PTTR_A", include_str!("fixtures/bdot10k/PTTR_A.xml")),
        ("PTLZ_A", include_str!("fixtures/bdot10k/PTLZ_A.xml")),
        ("PTPL_A", include_str!("fixtures/bdot10k/PTPL_A.xml")),
        ("PTZB_A", include_str!("fixtures/bdot10k/PTZB_A.xml")),
        ("KUSC_A", include_str!("fixtures/bdot10k/KUSC_A.xml")),
    ]
    .into_iter()
    .filter(|(class, _)| cemeteries || *class != "KUSC_A")
    {
        writer
            .start_file(name(class), zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(text.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// The service of BDOT10k answering, and EGiB answering `buildings` and `parcels` as given.
fn served(buildings: &str, parcels: &str) -> MapFetcher {
    let fetcher = MapFetcher::default();
    fetcher.insert_prefix(
        &format!("{EGIB}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ms:budynki"),
        buildings.as_bytes().to_vec(),
    );
    fetcher.insert_prefix(
        &format!("{EGIB}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ms:dzialki"),
        parcels.as_bytes().to_vec(),
    );
    fetcher.insert_prefix(INDEX, POWIATY.as_bytes().to_vec());
    fetcher.insert(ARCHIVE, archive());
    fetcher
}

/// A page of parcels: `(class, [(north, east)])` each, rings closed.
fn parcels(list: &[(&str, [(f64, f64); 5])]) -> String {
    let members: String = list
        .iter()
        .enumerate()
        .map(|(i, (class, ring))| {
            let pos: Vec<String> = ring.iter().map(|(n, e)| format!("{n} {e}")).collect();
            format!(
                "<wfs:member><ms:dzialki><ms:geom><gml:Polygon><gml:exterior><gml:LinearRing>\
                 <gml:posList>{}</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></ms:geom>\
                 <ms:ID_DZIALKI>P{i}</ms:ID_DZIALKI><ms:KLASOUZYTKI_EGIB>{class}</ms:KLASOUZYTKI_EGIB>\
                 </ms:dzialki></wfs:member>",
                pos.join(" ")
            )
        })
        .collect();
    format!(
        "<wfs:FeatureCollection xmlns:ms=\"m\" xmlns:gml=\"g\" xmlns:wfs=\"w\">{members}</wfs:FeatureCollection>"
    )
}

/// A square of `side` metres with its south-west corner at `(north, east)` of EPSG:2180.
fn square(north: f64, east: f64, side: f64) -> [(f64, f64); 5] {
    [
        (north, east),
        (north, east + side),
        (north + side, east + side),
        (north + side, east),
        (north, east),
    ]
}

struct Run {
    _scratch: tempfile::TempDir,
    cache: CacheDir,
    mode: FetchMode,
    notes: Vec<(String, bool)>,
}

impl Run {
    fn new() -> Self {
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        Self {
            _scratch: scratch,
            cache,
            mode: FetchMode::Online,
            notes: Vec::new(),
        }
    }

    fn with<T>(
        &mut self,
        fetcher: &MapFetcher,
        run: impl FnOnce(&dyn freemapper_adapter_api::alkis::AlkisSource, &mut AdapterEnv<'_>) -> T,
    ) -> T {
        let mut registry = Registry::new();
        freemapper_adapters_pl::register(&mut registry).unwrap();
        let adapter = registry.get("polska").unwrap();
        let notes = &mut self.notes;
        let mut progress = |n: freemapper_adapter_api::note::Note| {
            notes.push((n.to_string(), n.is_warning()));
        };
        let mut env = AdapterEnv {
            fetcher,
            regions: &registry,
            cache: &self.cache,
            mode: self.mode,
            today: NaiveDate::from_ymd_opt(2026, 11, 22).unwrap(),
            skip_surface: false,
            progress: &mut progress,
        };
        run(adapter.alkis().unwrap(), &mut env)
    }

    fn buildings(&mut self, fetcher: &MapFetcher) -> Result<AlkisPart, AdapterError> {
        self.with(fetcher, |source, env| source.buildings(env, &area()))
    }

    fn land_use(&mut self, fetcher: &MapFetcher) -> Result<AlkisPart, AdapterError> {
        self.with(fetcher, |source, env| source.land_use(env, &area()))
    }

    fn has_note(&self, text: &str, warning: bool) -> bool {
        self.notes
            .iter()
            .any(|(t, w)| t.contains(text) && *w == warning)
    }
}

fn asked_bdot(fetcher: &MapFetcher) -> bool {
    fetcher
        .log
        .borrow()
        .iter()
        .any(|u| u.starts_with(INDEX) || u == ARCHIVE)
}

fn oids(part: &AlkisPart) -> Vec<String> {
    part.features
        .iter()
        .map(|f| f["properties"]["oid"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn where_egib_has_buildings_bdot10k_is_never_asked() {
    let fetcher = served(EGIB_BUILDINGS, EMPTY);
    let mut run = Run::new();
    let part = run.buildings(&fetcher).unwrap();
    assert_eq!(part.features.len(), 2);
    assert!(oids(&part).iter().all(|o| o.ends_with("_BUD")));
    assert!(!asked_bdot(&fetcher));
    assert!(run.notes.is_empty());
}

#[test]
fn where_egib_has_no_buildings_bdot10k_gives_them_and_the_run_says_so() {
    let fetcher = served(EMPTY, EMPTY);
    let mut run = Run::new();
    let part = run.buildings(&fetcher).unwrap();
    // The fixture holds six buildings in the box; the box of the area is the one of the fixture
    // within a few metres, so most of them touch it.
    assert!(part.features.len() >= 4, "{}", part.features.len());
    assert!(oids(&part).contains(&"080505_4.0001.629/11.1_BUD".to_string()));
    assert_eq!(part.features[0]["properties"]["gebnutzbez"], "Gebäude");
    assert!(run.has_note("BDOT10k buildings filled in where EGiB gives none", false));
}

#[test]
fn a_failing_egib_is_stood_in_for_by_bdot10k_with_a_warning() {
    let fetcher = served(EMPTY, EMPTY);
    fetcher.refusing.borrow_mut().push(EGIB.to_string());
    let mut run = Run::new();
    let part = run.buildings(&fetcher).unwrap();
    assert!(part.features.len() >= 4);
    assert!(run.has_note("EGiB is unavailable", true));
}

#[test]
fn when_egib_and_bdot10k_both_fail_the_error_is_that_of_egib() {
    let fetcher = served(EMPTY, EMPTY);
    fetcher.refusing.borrow_mut().push(EGIB.to_string());
    fetcher.refusing.borrow_mut().push(INDEX.to_string());
    let mut run = Run::new();
    let error = run.buildings(&fetcher).unwrap_err();
    assert!(error.to_string().contains("EGIB"), "{error}");
    assert!(run.has_note("could not fill the gap", true));
}

#[test]
fn an_empty_egib_and_a_failing_bdot10k_give_the_empty_map_and_a_warning_not_an_error() {
    let fetcher = served(EMPTY, EMPTY);
    fetcher.refusing.borrow_mut().push(INDEX.to_string());
    let mut run = Run::new();
    let part = run.buildings(&fetcher).unwrap();
    assert!(part.features.is_empty());
    assert!(run.has_note("could not fill the gap", true));
}

#[test]
fn a_broken_archive_does_not_fail_the_run_either() {
    let fetcher = served(EMPTY, EMPTY);
    fetcher.insert(ARCHIVE, b"not an archive".to_vec());
    let mut run = Run::new();
    let part = run.buildings(&fetcher).unwrap();
    assert!(part.features.is_empty());
    assert!(run.has_note("no archive", true));
}

#[test]
fn a_box_in_a_gap_that_bdot10k_does_not_list_stays_empty_without_a_warning() {
    let fetcher = served(EMPTY, EMPTY);
    fetcher.insert_prefix(INDEX, EMPTY.as_bytes().to_vec());
    let mut run = Run::new();
    assert!(run.buildings(&fetcher).unwrap().features.is_empty());
    assert!(run.notes.is_empty());
}

#[test]
fn parcels_that_cover_the_area_keep_bdot10k_away_from_the_land_use() {
    let wood = parcels(&[("Ls", square(507_200.0, 197_400.0, 800.0))]);
    let fetcher = served(EMPTY, &wood);
    let mut run = Run::new();
    let part = run.land_use(&fetcher).unwrap();
    assert_eq!(oids(&part), ["P0"]);
    // Only the cemeteries are asked, by byte ranges: no archive in the cache, no land use read.
    assert!(!cached_archive(&run));
    assert!(run.notes.is_empty());
}

#[test]
fn parcels_without_a_class_leave_the_land_use_to_bdot10k() {
    // The cities send the parcels and no class.
    let bare = parcels(&[("", square(507_200.0, 197_400.0, 800.0))]);
    let fetcher = served(EMPTY, &bare);
    let mut run = Run::new();
    let part = run.land_use(&fetcher).unwrap();
    assert!(!part.features.is_empty());
    assert!(oids(&part).iter().all(|o| o.starts_with("BDOT10k.")));
    let uses: Vec<&str> = part
        .features
        .iter()
        .map(|f| f["properties"]["nutzart"].as_str().unwrap())
        .collect();
    assert!(uses.contains(&"Landwirtschaft"), "{uses:?}");
    assert!(uses.contains(&"Platz"), "{uses:?}");
    assert!(run.has_note("BDOT10k land use filled in", false));
}

#[test]
fn bdot10k_land_use_stays_out_of_what_egib_does_cover() {
    let baseline = {
        let fetcher = served(EMPTY, EMPTY);
        Run::new().land_use(&fetcher).unwrap().features.len()
    };
    // A small parcel of farmland around the one tree stand of the fixture (at 197 831 east, 507 759
    // north): too small to count as coverage, big enough to hold the stand's middle.
    let small = parcels(&[("RIV", square(507_700.0, 197_780.0, 120.0))]);
    let fetcher = served(EMPTY, &small);
    let mut run = Run::new();
    let part = run.land_use(&fetcher).unwrap();
    let own = oids(&part).iter().filter(|o| *o == "P0").count();
    assert_eq!(own, 1);
    // EGiB's parcel stands, and one object of BDOT10k less than without it.
    let bdot = part.features.len() - own;
    assert!(bdot < baseline, "{bdot} < {baseline}");
}

#[test]
fn a_failing_egib_parcel_layer_is_stood_in_for_by_bdot10k() {
    let fetcher = served(EMPTY, EMPTY);
    fetcher.refusing.borrow_mut().push(format!(
        "{EGIB}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ms:dzialki"
    ));
    let mut run = Run::new();
    let part = run.land_use(&fetcher).unwrap();
    assert!(!part.features.is_empty());
    assert!(run.has_note("EGiB is unavailable", true));
}

#[test]
fn every_address_the_fallback_asks_is_on_a_host_of_the_adapter() {
    let fetcher = served(EMPTY, EMPTY);
    let mut run = Run::new();
    run.buildings(&fetcher).unwrap();
    for url in fetcher.log.borrow().iter() {
        let host = url.split('/').nth(2).unwrap();
        assert!(
            ["mapy.geoportal.gov.pl", "opendata.geoportal.gov.pl"].contains(&host),
            "{url}"
        );
    }
}

/// Whether a powiat archive was downloaded whole into the cache.
fn cached_archive(run: &Run) -> bool {
    let dir = run
        .cache
        .provider_dir(freemapper_adapters_pl::rasters::PROVIDER)
        .join("bdot10k");
    dir.read_dir()
        .is_ok_and(|mut entries| entries.next().is_some())
}

/// Parcels where EGiB gives a land use for a quarter of the box (two woods in the south) and a
/// `Bi` parcel on the cemetery of the fixture (at 507 700 to 507 783 north, 197 888 to 197 970
/// east) and one on the church grounds (507 560 to 507 645 north, 197 653 to 197 780 east).
fn parcels_with(on_cemetery: &str) -> String {
    parcels(&[
        ("Ls", square(507_405.0, 197_783.0, 300.0)),
        ("Ls", square(507_405.0, 197_483.0, 100.0)),
        ("Bi", square(507_540.0, 197_640.0, 150.0)),
        (on_cemetery, square(507_650.0, 197_850.0, 200.0)),
    ])
}

fn friedhof_ids(part: &AlkisPart) -> Vec<String> {
    part.features
        .iter()
        .filter(|f| f["properties"]["nutzart"] == "Friedhof")
        .map(|f| f["properties"]["oid"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_cemetery_on_a_bi_parcel_is_a_friedhof_and_church_grounds_are_not() {
    let fetcher = served(EMPTY, &parcels_with("Bi"));
    let mut run = Run::new();
    let part = run.land_use(&fetcher).unwrap();
    // The two woods of EGiB (a `Bi` parcel gives no land use) and one cemetery.
    assert_eq!(part.features.len(), 3, "{:?}", oids(&part));
    let friedhof = friedhof_ids(&part);
    assert_eq!(friedhof.len(), 1);
    assert!(friedhof[0].starts_with("BDOT10k.KUSC_A."), "{friedhof:?}");
    assert!(
        run.has_note("1 BDOT10k cemeteries filled in", false),
        "{:?}",
        run.notes
    );
    assert!(!cached_archive(&run));
}

#[test]
fn a_cemetery_on_a_parcel_with_a_land_use_is_left_to_egib() {
    for class in ["Bz", "Lz", "B", "Ls"] {
        let fetcher = served(EMPTY, &parcels_with(class));
        let mut run = Run::new();
        let part = run.land_use(&fetcher).unwrap();
        assert!(friedhof_ids(&part).is_empty(), "{class}");
        assert!(run.notes.is_empty(), "{class}: {:?}", run.notes);
    }
}

#[test]
fn a_cemetery_on_a_parcel_of_mixed_classes_is_a_friedhof() {
    // `Bi,Bz` is a parcel without a land use of its own (ADR 0171).
    let fetcher = served(EMPTY, &parcels_with("Bi,Bz"));
    let mut run = Run::new();
    let part = run.land_use(&fetcher).unwrap();
    assert_eq!(friedhof_ids(&part).len(), 1);
}

#[test]
fn offline_no_cemetery_is_asked() {
    let fetcher = served(EMPTY, &parcels_with("Bi"));
    let mut run = Run::new();
    run.mode = FetchMode::Offline;
    let part = run.land_use(&fetcher).unwrap();
    assert!(friedhof_ids(&part).is_empty());
    assert!(!fetcher.log.borrow().iter().any(|u| u.starts_with(INDEX)));
    assert!(run.notes.is_empty());
}

#[test]
fn a_failing_index_is_a_warning_and_the_land_use_stays_that_of_egib() {
    let fetcher = served(EMPTY, &parcels_with("Bi"));
    fetcher.refusing.borrow_mut().push(INDEX.to_string());
    let mut run = Run::new();
    let part = run.land_use(&fetcher).unwrap();
    assert_eq!(oids(&part), ["P0", "P1"].map(String::from));
    assert!(run.has_note("BDOT10k could not fill the gap in EGiB (cemeteries)", true));
}

#[test]
fn an_archive_without_a_cemetery_file_adds_nothing_and_says_nothing() {
    let fetcher = served(EMPTY, &parcels_with("Bi"));
    fetcher.insert(ARCHIVE, archive_with(false));
    let mut run = Run::new();
    let part = run.land_use(&fetcher).unwrap();
    assert!(friedhof_ids(&part).is_empty());
    assert_eq!(part.features.len(), 2);
    assert!(run.notes.is_empty());
}

#[test]
fn a_box_in_no_powiat_adds_nothing_and_says_nothing() {
    let fetcher = served(EMPTY, &parcels_with("Bi"));
    fetcher.insert_prefix(INDEX, EMPTY.as_bytes().to_vec());
    let mut run = Run::new();
    let part = run.land_use(&fetcher).unwrap();
    assert_eq!(part.features.len(), 2);
    assert!(run.notes.is_empty());
}
