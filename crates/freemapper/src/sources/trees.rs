//! Tree cadastres of cities. The reader moved to `freemapper-sources` as `trees` (ADR 0082);
//! this module keeps the old paths working and holds what picks a dataset by state.

pub use freemapper_sources::trees::*;

use crate::adapters::{RegionAdapter, Registry, asking, registry};
use crate::crs::{BoundingBox, Crs};
use crate::fetch::Fetcher;
use crate::sources::alkis::FetchArea;
use crate::sources::alkis_nas::PartFile;
use crate::sources::region::touches_region_in;

/// The tree cadastres of every adapter, in the order of the registry.
pub fn all_tree_datasets() -> impl Iterator<Item = &'static TreeDataset> {
    registry()
        .adapters()
        .flat_map(|adapter| adapter.tree_datasets())
}

/// The datasets a map area needs: those whose envelope the part of the area inside their state
/// intersects (the envelope of Potsdam reaches into Berlin, which has its own register).
/// `region_bbox` is the area in EPSG:25833.
pub fn tree_datasets_for(region_bbox: &BoundingBox) -> Vec<&'static TreeDataset> {
    needed_in(registry(), region_bbox)
        .map(|(_, dataset)| dataset)
        .collect()
}

/// The datasets of `registry` the area in `region_bbox` (EPSG:25833) needs, each with the
/// adapter that brings it, in the order of the registry.
fn needed_in<'a, 'b>(
    registry: &'a Registry,
    region_bbox: &'b BoundingBox,
) -> impl Iterator<Item = (&'a dyn RegionAdapter, &'a TreeDataset)> + use<'a, 'b> {
    registry
        .adapters()
        .flat_map(|adapter| {
            adapter
                .tree_datasets()
                .iter()
                .map(move |dataset| (adapter, dataset))
        })
        .filter(move |(adapter, dataset)| needs(*adapter, dataset, region_bbox))
}

/// Whether the area in `region_bbox` (EPSG:25833) needs `dataset` of `adapter`. The state is
/// the adapter that brings the dataset, not what its row names.
fn needs(adapter: &dyn RegionAdapter, dataset: &TreeDataset, region_bbox: &BoundingBox) -> bool {
    let area = FetchArea::new(*region_bbox, Crs::Utm33).bbox_for(dataset.crs);
    if !intersects(&area, &dataset.envelope) {
        return false;
    }
    // The state's boundary is in EPSG:25833: clip the area to the envelope there.
    if dataset.crs != Crs::Utm33 {
        return touches_region_in(adapter, region_bbox);
    }
    let e = &dataset.envelope;
    let clip = BoundingBox {
        min_e: area.min_e.max(e.min_e),
        min_n: area.min_n.max(e.min_n),
        max_e: area.max_e.min(e.max_e),
        max_n: area.max_n.min(e.max_n),
    };
    touches_region_in(adapter, &clip)
}

fn intersects(a: &BoundingBox, b: &BoundingBox) -> bool {
    a.min_e <= b.max_e && b.min_e <= a.max_e && a.min_n <= b.max_n && b.min_n <= a.max_n
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
    fetch_tree_layers_in(registry(), fetcher, area, part)
}

/// [`fetch_tree_layers`] with the adapters of `registry`.
fn fetch_tree_layers_in(
    registry: &'static Registry,
    fetcher: &dyn Fetcher,
    area: &FetchArea,
    part: PartFile,
) -> Vec<FetchedTreeLayer> {
    needed_in(registry, &area.region_bbox)
        .map(|(adapter, dataset)| fetch_tree_layer(adapter, fetcher, dataset, area, part))
        .collect()
}

/// `dataset` of `adapter` for `area`, asked as the adapter asks: one that is held to its
/// declared hosts ([`crate::adapters::RegionAdapter::policy`]) through that policy. The adapter
/// is the one that brings the dataset; the region its row names decides nothing here.
fn fetch_tree_layer(
    adapter: &dyn RegionAdapter,
    fetcher: &dyn Fetcher,
    dataset: &'static TreeDataset,
    area: &FetchArea,
    part: PartFile,
) -> FetchedTreeLayer {
    let features = asking(adapter, fetcher, |fetcher| {
        fetch_dataset(fetcher, dataset, area, part)
    });
    FetchedTreeLayer { dataset, features }
}

// The tests of the reader that take Potsdam and its state as their example stay here until the
// states are adapters.
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::fetch::FetchError;
    use crate::fetch::test_support::MapFetcher;
    #[allow(unused_imports)]
    use crate::sources::region::ids::{IdFacts as _, regions};
    use freemapper_adapter_api::records::AdapterId;

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
            tree_datasets_for(bbox)
                .iter()
                .map(|d| d.id.as_ref())
                .collect()
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
            &POTSDAM.endpoints[0],
            test_support::POTSDAM.as_bytes().to_vec(),
        );
        fetcher.insert_prefix(
            &POTSDAM.endpoints[1],
            br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
        );
        fetcher.insert_prefix(
            &POTSDAM.endpoints[2],
            br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
        );
        let layers = fetch_tree_layers(&fetcher, &FetchArea::new(sanssouci(), Crs::Utm33), &part);
        assert_eq!(fetcher.calls.get(), 3);
        assert_eq!(layers.len(), 1);
        let trees = layers[0].features.as_ref().unwrap();
        assert_eq!(trees.features.len(), 3);
        assert_eq!(trees.regions.len(), 1);
        assert_eq!(trees.regions[0].as_str(), regions::BRANDENBURG.id());
    }

    /// The tree cadastre of an adapter from a folder is asked through the adapter's policy:
    /// off the hosts it declares, no request leaves. One of the program is asked as before.
    #[test]
    fn the_tree_cadastre_of_a_folder_adapter_is_held_to_its_hosts() {
        use crate::adapters::test_support::{SQUARE, square};
        let dataset: &'static TreeDataset = Box::leak(Box::new(TreeDataset {
            region: SQUARE,
            ..POTSDAM
        }));
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("answer.geojson.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        let area = FetchArea::new(sanssouci(), Crs::Utm33);
        let fetched = |hosts: Option<&[&str]>| {
            let registry = square(None, hosts);
            let fetcher = MapFetcher::default();
            fetcher.insert_prefix(
                "https://",
                br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
            );
            let adapter = registry.get(SQUARE.as_str()).unwrap();
            let layer = fetch_tree_layer(adapter, &fetcher, dataset, &area, &part);
            (layer.features, fetcher.calls.get())
        };
        let (features, calls) = fetched(Some(&["geo.example.org"]));
        assert!(
            matches!(features, Err(FetchError::NotAllowed { .. })),
            "{features:?}"
        );
        assert_eq!(calls, 0);
        for hosts in [Some(&["opendata.potsdam.de"][..]), None] {
            let (features, calls) = fetched(hosts);
            assert!(features.is_ok(), "{features:?}");
            assert_eq!(calls, 3);
        }
    }

    /// One dataset is asked by the rules of the adapter it is asked for, whatever region its row
    /// names: the row is not looked up.
    #[test]
    fn a_tree_cadastre_is_held_to_the_hosts_of_the_adapter_that_brings_it() {
        use crate::adapters::test_support::{SQUARE, square};
        let dataset: &'static TreeDataset = Box::leak(Box::new(TreeDataset {
            region: AdapterId::built_in("elsewhere"),
            ..POTSDAM
        }));
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("answer.geojson.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        let area = FetchArea::new(sanssouci(), Crs::Utm33);
        let registry = square(None, Some(&["geo.example.org"]));
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            "https://",
            br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
        );
        let adapter = registry.get(SQUARE.as_str()).unwrap();
        let layer = fetch_tree_layer(adapter, &fetcher, dataset, &area, &part);
        assert!(
            matches!(layer.features, Err(FetchError::NotAllowed { .. })),
            "{:?}",
            layer.features
        );
        assert_eq!(fetcher.calls.get(), 0);
    }

    /// A row that names another adapter of the registry does not move the dataset there: the
    /// area needs it on the ground of the adapter that brings it, and that adapter's hosts
    /// decide whether the request leaves.
    #[test]
    fn a_row_that_names_another_adapter_goes_by_the_one_that_brings_it() {
        use crate::adapters::test_support::{FAR, square_and_far};
        let dataset = TreeDataset {
            region: FAR,
            envelope: BoundingBox {
                min_e: 0.0,
                min_n: 0.0,
                max_e: 30_000.0,
                max_n: 10_000.0,
            },
            ..POTSDAM
        };
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("answer.geojson.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        let from = |min_e: f64| {
            let bbox = BoundingBox {
                min_e,
                min_n: 4_000.0,
                max_e: min_e + 500.0,
                max_n: 4_500.0,
            };
            FetchArea::new(bbox, Crs::Utm33)
        };
        // The far region may ask the host of the dataset: that must not help the square.
        let fetched = |hosts: &[&str], area: &FetchArea| {
            let registry: &'static Registry = Box::leak(Box::new(square_and_far(
                vec![dataset.clone()],
                hosts,
                &["opendata.potsdam.de"],
            )));
            let fetcher = MapFetcher::default();
            fetcher.insert_prefix(
                "https://",
                br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
            );
            let layers = fetch_tree_layers_in(registry, &fetcher, area, &part);
            (layers, fetcher.calls.get())
        };
        // On the ground of the far region nothing needs the dataset of the square.
        let (layers, calls) = fetched(&["geo.example.org"], &from(24_000.0));
        assert!(layers.is_empty(), "{layers:?}");
        assert_eq!(calls, 0);
        // On the ground of the square the area needs it, and the hosts of the square decide.
        let (layers, calls) = fetched(&["geo.example.org"], &from(4_000.0));
        assert_eq!(layers.len(), 1);
        assert!(
            matches!(layers[0].features, Err(FetchError::NotAllowed { .. })),
            "{layers:?}"
        );
        assert_eq!(calls, 0);
        let (layers, calls) = fetched(&["opendata.potsdam.de"], &from(4_000.0));
        assert!(layers[0].features.is_ok(), "{layers:?}");
        assert_eq!(calls, 3);
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
        for endpoint in POTSDAM.endpoints.iter() {
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
        // A dataset of the adapter `region` with this envelope. Its row names Brandenburg
        // whichever adapter brings it: the state is the adapter's.
        let needed = |region: AdapterId, min_e, max_e| {
            let dataset = TreeDataset {
                envelope: BoundingBox {
                    min_e,
                    min_n: 5_816_000.0,
                    max_e,
                    max_n: 5_819_000.0,
                },
                ..POTSDAM
            };
            needs(registry().get(region.id()).unwrap(), &dataset, &area)
        };
        // The area touches both states, but the envelope covers only the Berlin side.
        let berlin_side = |region| needed(region, 374_300.0, 374_700.0);
        assert!(berlin_side(regions::BERLIN));
        assert!(!berlin_side(regions::BRANDENBURG));
        // And only the Brandenburg side.
        let brandenburg_side = |region| needed(region, 373_000.0, 373_400.0);
        assert!(brandenburg_side(regions::BRANDENBURG));
        assert!(!brandenburg_side(regions::BERLIN));
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
