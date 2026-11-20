//! The adapter `polska`: where Poland lies and which OSM extracts a map area reads.

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::geometry::{contains_point_in, within_margin};
use freemapper_adapter_api::info::{Country, Coverage};
use freemapper_adapter_api::registry::Registry;
use freemapper_georef::crs::{BoundingBox, Crs, LatLon};

fn polska() -> Registry {
    let mut registry = Registry::new();
    freemapper_adapters_pl::register(&mut registry).unwrap();
    registry
}

fn point(lat: f64, lon: f64) -> (f64, f64) {
    let p = Crs::Utm33.forward(LatLon { lat, lon });
    (p.easting, p.northing)
}

fn square(lat: f64, lon: f64, half_m: f64) -> BoundingBox {
    let (e, n) = point(lat, lon);
    BoundingBox {
        min_e: e - half_m,
        min_n: n - half_m,
        max_e: e + half_m,
        max_n: n + half_m,
    }
}

/// The extracts (by id) a map area reads, as the engine asks.
fn extracts_of(adapter: &dyn RegionAdapter, bbox: &BoundingBox) -> Vec<String> {
    adapter
        .info()
        .osm
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            adapter
                .extract_boundary(*i)
                .is_none_or(|boundary| within_margin(boundary, bbox))
        })
        .map(|(_, extract)| extract.prefix.clone())
        .collect()
}

#[test]
fn polska_is_an_osm_only_polish_region() {
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    let info = adapter.info();
    assert_eq!(info.country, Country::Pl);
    assert_eq!(info.coverage, Coverage::OsmOnly);
    assert_eq!(info.names.en, "Poland");
    assert_eq!(info.names.de, "Polen");
    assert!(adapter.rasters().is_none() && adapter.alkis().is_none());
}

#[test]
fn warsaw_is_inside_the_boundary() {
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    for (lat, lon) in [(52.2297, 21.0122), (54.352, 18.6466), (50.0647, 19.945)] {
        let (e, n) = point(lat, lon);
        assert!(contains_point_in(adapter, e, n), "{lat} {lon}");
    }
}

#[test]
fn berlin_and_prague_are_outside() {
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    for (lat, lon) in [(52.52, 13.405), (50.0755, 14.4378), (53.55, 9.99)] {
        let (e, n) = point(lat, lon);
        assert!(!contains_point_in(adapter, e, n), "{lat} {lon}");
    }
}

#[test]
fn sixteen_extracts_with_unique_prefixes() {
    let registry = polska();
    let info = registry.get("polska").unwrap().info();
    assert_eq!(info.osm.len(), 16);
    let mut prefixes: Vec<_> = info.osm.iter().map(|e| e.prefix.clone()).collect();
    prefixes.sort();
    prefixes.dedup();
    assert_eq!(prefixes.len(), 16);
    for extract in &info.osm {
        assert!(extract.geofabrik_path.starts_with("europe/poland/"));
        assert!(extract.megabytes > 30.0, "{}", extract.label);
    }
    assert_eq!(
        info.osm[0].url(),
        format!(
            "https://download.geofabrik.de/{}-latest.osm.pbf",
            info.osm[0].geofabrik_path
        )
    );
}

#[test]
fn a_box_in_lodz_reads_only_lodzkie() {
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    let reads = extracts_of(adapter, &square(51.7592, 19.456, 1_000.0));
    assert_eq!(reads, vec!["lodzkie_".to_string()]);
}

#[test]
fn a_box_on_a_voivodeship_border_reads_both_sides() {
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    // Two cities a few km apart across the border of Łódź and Masovia: Żyrardów (Masovia) lies at
    // the border; a 12 km box around Skierniewice/Żyrardów reaches both.
    let reads = extracts_of(adapter, &square(52.0, 20.2, 12_000.0));
    assert!(reads.contains(&"lodzkie_".to_string()), "{reads:?}");
    assert!(reads.contains(&"mazowieckie_".to_string()), "{reads:?}");
}

#[test]
fn the_zone_follows_the_centre() {
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    assert_eq!(
        adapter.crs_for(LatLon {
            lat: 52.2297,
            lon: 21.0122
        }),
        Crs::Utm34
    );
    assert_eq!(
        adapter.crs_for(LatLon {
            lat: 52.349,
            lon: 14.556
        }),
        Crs::Utm33
    );
}
