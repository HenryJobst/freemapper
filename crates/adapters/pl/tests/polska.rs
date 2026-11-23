// SPDX-License-Identifier: AGPL-3.0-or-later

//! The adapter `polska`: where Poland lies and which OSM extracts a map area reads.

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::geometry::{contains_point_in, map_crs_in, within_margin};
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
fn polska_is_a_polish_region_with_rasters_and_a_cadastre() {
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    let info = adapter.info();
    assert_eq!(info.country, Country::Pl);
    // Rasters and the cadastre (EGiB) of GUGiK.
    assert_eq!(info.coverage, Coverage::Full);
    assert_eq!(info.names.en, "Poland");
    assert_eq!(info.names.de, "Polen");
    // The rasters are those of GUGiK, and the region names their provider.
    let rasters = adapter.rasters().expect("the rasters of GUGiK");
    assert_eq!(rasters.provider().id.as_str(), "gugik");
    assert_eq!(info.provider.as_str(), "gugik");
    assert!(adapter.alkis().is_some());
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

/// The boundary is the state border, not the clipping polygon of the extracts, which reaches
/// kilometres into Germany and Czechia.
#[test]
fn border_towns_of_the_neighbours_are_outside() {
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    for (name, lat, lon) in [
        ("Frankfurt (Oder)", 52.340, 14.540),
        ("Görlitz", 51.150, 14.990),
        ("Guben", 51.950, 14.720),
        ("Bad Muskau", 51.550, 14.720),
        ("Český Těšín", 49.750, 18.620),
    ] {
        let (e, n) = point(lat, lon);
        assert!(!contains_point_in(adapter, e, n), "{name}");
    }
    for (name, lat, lon) in [("Słubice", 52.350, 14.560), ("Cieszyn", 49.750, 18.630)] {
        let (e, n) = point(lat, lon);
        assert!(contains_point_in(adapter, e, n), "{name}");
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

/// A map area around Opole that reaches over the zone border at 18° E has one zone, that of its
/// centre, and the real adapter says so (not only a stand-in region).
#[test]
fn an_area_over_the_zone_border_has_the_zone_of_its_centre() {
    let registry = polska();
    // 17.7° to 18.4° E: the border lies inside. The centre is 17.95° E, 18.05° E.
    let area = |lon: f64| {
        let a = Crs::Utm33.forward(LatLon {
            lat: 50.6,
            lon: lon - 0.25,
        });
        let b = Crs::Utm33.forward(LatLon {
            lat: 50.8,
            lon: lon + 0.25,
        });
        BoundingBox {
            min_e: a.easting,
            min_n: a.northing,
            max_e: b.easting,
            max_n: b.northing,
        }
    };
    assert_eq!(map_crs_in(&registry, &area(17.95)), Crs::Utm33);
    assert_eq!(map_crs_in(&registry, &area(18.05)), Crs::Utm34);
}

/// The adapter declares itself (revision and day of its check) and has a policy that names the
/// two hosts of GUGiK, the host of the GDOŚ and no other: the OSM extracts come through the core hosts.
#[test]
fn the_adapter_declares_itself_and_owns_a_policy_of_the_hosts_of_gugik() {
    let registry = polska();
    let adapter = registry.adapters().next().unwrap();
    let declared = adapter.declared().expect("the adapter declares itself");
    assert_eq!(declared.revision, 8);
    assert!(declared.last_verified.is_some());
    let hosts = [
        "mapy.geoportal.gov.pl",
        "opendata.geoportal.gov.pl",
        "sdi.gdos.gov.pl",
        "usluga.zabytek.gov.pl",
    ];
    // What it declares (`adapters list`) is what its policy lets through.
    assert_eq!(declared.hosts, hosts);
    let policy = adapter.policy().expect("the adapter has a policy");
    assert_eq!(policy.hosts().collect::<Vec<_>>(), hosts);
    for url in [
        "https://mapy.geoportal.gov.pl/wss/service/PZGIK/ORTO/WFS/Skorowidze?SERVICE=WFS",
        "https://opendata.geoportal.gov.pl/NumDaneWys/NMT/73426/x.asc",
        "https://sdi.gdos.gov.pl/wfs?service=WFS",
    ] {
        assert!(policy.check_url(url).is_ok(), "{url}");
    }
    for url in [
        "https://download.geofabrik.de/europe/poland-latest.osm.pbf",
        "https://geoportal.gov.pl/x",
        "https://evil.example.com/opendata.geoportal.gov.pl",
        "http://opendata.geoportal.gov.pl/x.asc",
    ] {
        assert!(policy.check_url(url).is_err(), "{url}");
    }
}

/// The credit line of GUGiK stands in the attribution of a project whose rasters came from it,
/// and in none that holds no raster of it.
#[test]
fn the_credit_of_gugik_follows_the_rasters() {
    use freemapper_adapter_api::credit::UsedSources;
    use freemapper_adapter_api::records::{AdapterId, WcsRecord};
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    let terrain = WcsRecord {
        fetched: chrono::NaiveDate::from_ymd_opt(2026, 11, 21).unwrap(),
        bbox: [0.0; 4],
        date: Some("2020".into()),
    };
    let mut used = UsedSources::default();
    assert!(adapter.attribution(&used).is_empty());
    used.raster_provider = Some(AdapterId::built_in("gugik"));
    assert!(adapter.attribution(&used).is_empty());
    used.terrain = Some(&terrain);
    let lines: Vec<_> = adapter
        .attribution(&used)
        .into_iter()
        .map(|c| c.text)
        .collect();
    assert_eq!(lines, ["© GUGiK, geoportal.gov.pl"]);
    used.raster_provider = Some(AdapterId::built_in("lgb"));
    assert!(adapter.attribution(&used).is_empty());
}

/// The credit of GUGiK also stands in the attribution of a project that holds objects of its
/// cadastre, whatever its rasters were.
#[test]
fn the_credit_of_gugik_follows_the_cadastre() {
    use freemapper_adapter_api::credit::{AlkisUse, UsedSources};
    use freemapper_adapter_api::records::AdapterId;
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    let fetched = chrono::NaiveDate::from_ymd_opt(2026, 11, 21).unwrap();
    let mut used = UsedSources::default();
    used.alkis.push(AlkisUse {
        regions: vec![AdapterId::built_in("lgb")],
        fetched,
    });
    assert!(adapter.attribution(&used).is_empty());
    used.alkis.push(AlkisUse {
        regions: vec![AdapterId::built_in("polska")],
        fetched,
    });
    let lines: Vec<_> = adapter
        .attribution(&used)
        .into_iter()
        .map(|c| c.text)
        .collect();
    assert_eq!(lines, ["© GUGiK, geoportal.gov.pl"]);
}

/// A host a person allowed for `polska` is in its policy, and no other.
#[test]
fn an_allowed_host_is_in_the_policy() {
    let mut registry = Registry::new();
    let extra =
        std::collections::BTreeMap::from([("polska".to_string(), vec!["example.pl".to_string()])]);
    freemapper_adapters_pl::register_with_hosts(&mut registry, &extra).unwrap();
    let policy = registry.adapters().next().unwrap().policy().unwrap();
    assert!(policy.allows_host("example.pl"));
    assert!(!policy.allows_host("example.com"));
}

#[test]
fn polska_brings_the_permit_sources_of_the_gdos_and_the_prg() {
    let registry = polska();
    let adapter = registry.get("polska").unwrap();
    let keys: Vec<&str> = adapter
        .permit_sources()
        .iter()
        .map(|s| s.info().key)
        .collect();
    assert_eq!(
        keys,
        [
            "gdos",
            "prg",
            "prg-granice",
            "nid-zabytki",
            "nid-archeologia"
        ]
    );
}
