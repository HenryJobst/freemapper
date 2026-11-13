//! The manifests of the German states against the constants they were written from: the
//! services a manifest builds are the services the code names.

use std::borrow::Cow;
use std::path::Path;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::registry::Registry;
use freemapper_adapters_de::states::alkis::{
    BUILDINGS_ENDPOINT, LAND_USE_ENDPOINT, MV_BUILDINGS_ENDPOINT, MV_LAND_USE_ENDPOINT,
};
use freemapper_adapters_de::states::{State, StateAdapter, StateFiles};
use freemapper_adapters_de::{geobremen, lgl_bw, lgv, lvgl, states};
use freemapper_georef::crs::Crs;
use freemapper_manifest::adapter::{ManifestAdapter, Parse};
use freemapper_manifest::model::Alkis;
use freemapper_manifest::{Manifest, adapter, alkis, extras, parse};
use freemapper_sources::alkis_source::{Normaliser, OgcApiAlkis};
use freemapper_sources::nas::LGB_NAS;
use freemapper_sources::trees::POTSDAM;
use freemapper_sources::wfs::{self, WfsService};

fn manifest(id: &str) -> Manifest {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("manifests")
        .join(id)
        .join("manifest.toml");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    parse(&text, Path::new(id)).unwrap_or_else(|findings| {
        let lines: Vec<String> = findings.iter().map(ToString::to_string).collect();
        panic!("{}", lines.join("\n"))
    })
}

/// The WFS of a manifest: its ALKIS service, or the fallback of its OGC API.
fn wfs_of(manifest: &Manifest) -> WfsService {
    match manifest.alkis.as_ref().unwrap() {
        Alkis::Wfs(wfs) => alkis::wfs_service(wfs),
        Alkis::OgcApi {
            fallback: Some(wfs),
            ..
        } => alkis::wfs_service(wfs),
        other => panic!("{}: no WFS in {other:?}", manifest.adapter.id),
    }
}

#[test]
fn manifest_wfs_rows_equal_the_ten_constants() {
    let rows: [(&str, WfsService); 10] = [
        ("saxony", wfs::GEOSN),
        ("saxonyanhalt", wfs::LVERMGEO),
        ("hamburg", wfs::LGV),
        ("hessen", wfs::HVBG),
        ("schleswigholstein", wfs::LVERMGEO_SH),
        ("mecklenburgvorpommern", wfs::LAIV_MV),
        ("nordrheinwestfalen", wfs::GEOBASIS_NRW),
        ("niedersachsen", wfs::LGLN),
        ("rheinlandpfalz", wfs::LVERMGEO_RP),
        ("saarland", lvgl::ALKIS),
    ];
    for (id, constant) in rows {
        assert_eq!(wfs_of(&manifest(id)), constant, "{id}");
    }
}

#[test]
fn hamburg_manifest_equals_the_ogc_api_plus_wfs_fallback() {
    let api = |id: &str| alkis::ogc_api(manifest(id).alkis.as_ref().unwrap()).unwrap();
    assert_eq!(
        api("hamburg"),
        OgcApiAlkis {
            buildings: Cow::Borrowed(lgv::BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(lgv::LAND_USE_ENDPOINT),
            crs: Crs::Utm32,
            fallback: Some(wfs::LGV),
            normalise: Normaliser::HamburgAscii,
        }
    );
    assert_eq!(
        api("mecklenburgvorpommern"),
        OgcApiAlkis {
            buildings: Cow::Borrowed(MV_BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(MV_LAND_USE_ENDPOINT),
            crs: Crs::Utm33,
            fallback: Some(wfs::LAIV_MV),
            normalise: Normaliser::Lgb,
        }
    );
    assert_eq!(
        api("brandenburg"),
        OgcApiAlkis {
            buildings: Cow::Borrowed(BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(LAND_USE_ENDPOINT),
            crs: Crs::Utm33,
            fallback: None,
            normalise: Normaliser::Lgb,
        }
    );
}

#[test]
fn potsdam_tree_dataset_from_manifest_equals_the_constant() {
    let brandenburg = manifest("brandenburg");
    let region = "brandenburg".parse().unwrap();
    assert_eq!(brandenburg.trees.len(), 1);
    assert_eq!(
        extras::tree_dataset(&brandenburg.trees[0], &region),
        POTSDAM
    );
    assert_eq!(brandenburg.nas.len(), 1);
    assert_eq!(extras::nas_service(&brandenburg.nas[0], &region), LGB_NAS);
    // Bremen brings the layers of its full ALKIS as well, from the service its buildings and
    // land use come from. No other state has such sources.
    let bremen = manifest("bremen");
    assert_eq!(bremen.nas.len(), 1);
    let service = extras::nas_service(&bremen.nas[0], &"bremen".parse().unwrap());
    assert_eq!(service.url, geobremen::ALKIS_URL);
    assert_eq!(service.crs, Crs::Utm32);
    assert_eq!(
        (service.max_count, service.max_bytes),
        (geobremen::ALKIS.max_count, geobremen::ALKIS.max_bytes)
    );
    assert!(bremen.trees.is_empty());
    for state in State::ALL {
        if !matches!(state, State::Brandenburg | State::Bremen) {
            let manifest = manifest(state.id());
            assert!(
                manifest.nas.is_empty() && manifest.trees.is_empty(),
                "{}",
                state.id()
            );
        }
    }
}

/// The golden net of the adapter work, at the root of the repository.
fn golden_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/golden/adapters")
}

/// The hosts in the trace of a recorded fetch, without Geofabrik's (the OSM extracts belong
/// to no adapter).
fn hosts_in(trace: &str) -> Vec<String> {
    let mut hosts: Vec<String> = trace
        .lines()
        .filter_map(|line| line.strip_prefix("GET https://"))
        .filter_map(|rest| rest.split(['/', '?', ' ']).next())
        .filter(|host| *host != "download.geofabrik.de")
        .map(str::to_string)
        .collect();
    hosts.sort();
    hosts.dedup();
    hosts
}

/// The hosts a fetch of `state` asked when the golden net recorded it: for the eleven states
/// of the net `g1-hosts.txt`, which keeps them for the states that are no longer recorded.
/// Saarland, Baden-Württemberg and Bremen came after the net: their hosts are those of the addresses
/// their backend and their WFS are written with.
fn recorded_hosts(state: State) -> Vec<String> {
    let written: &[&str] = match state {
        State::Saarland => &[
            lvgl::WMS.url.as_ref(),
            lvgl::SHARE,
            lvgl::ALKIS.url.as_ref(),
        ],
        State::BadenWuerttemberg => &[
            lgl_bw::WMS.url.as_ref(),
            lgl_bw::FLIGHTS,
            lgl_bw::FILES,
            lgl_bw::ALKIS.url.as_ref(),
        ],
        State::Bremen => &[
            geobremen::ORTHOPHOTOS,
            geobremen::FILES,
            geobremen::ALKIS_URL,
        ],
        _ => &[],
    };
    if !written.is_empty() {
        let mut hosts: Vec<String> = written
            .iter()
            .filter_map(|url| url.strip_prefix("https://")?.split('/').next())
            .map(str::to_string)
            .collect();
        hosts.sort();
        hosts.dedup();
        return hosts;
    }
    let path = golden_dir().join("g1-hosts.txt");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let line = text
        .lines()
        .find_map(|line| line.strip_prefix(state.id())?.strip_prefix(": "))
        .unwrap_or_else(|| panic!("{} names no hosts of {}", path.display(), state.id()));
    line.split(' ').map(str::to_string).collect()
}

/// A state the net still records asks the hosts `g1-hosts.txt` names, no more and no fewer: a
/// new recording that asks another host changes the file with it.
#[test]
fn the_recorded_hosts_are_those_of_the_traces() {
    let mut recorded = 0;
    for state in State::ALL {
        let trace = golden_dir().join(state.id()).join("trace.txt");
        if let Ok(trace) = std::fs::read_to_string(&trace) {
            assert_eq!(hosts_in(&trace), recorded_hosts(state), "{}", state.id());
            recorded += 1;
        }
    }
    assert!(recorded > 0, "the net records no state");
}

/// No two states share ground. The borders of neighbours were simplified apart and overlap by
/// strips; the measure of the loader must not take those for shared ground. The mouth of the
/// Elbe, which the cadastres of Lower Saxony and Schleswig-Holstein both hold (89 km² of water),
/// is split between the two boundary files at the line of equal distance from the two claims.
#[test]
fn no_two_states_share_ground() {
    use geo::{Area, BooleanOps};
    let mut registry = Registry::new();
    states::register(&mut registry).unwrap();
    let mut largest: f64 = 0.0;
    for state in State::ALL {
        let adapter = registry.get(state.id()).unwrap();
        let others: Vec<String> = freemapper_manifest::loader::sharing_ground(&registry, adapter)
            .into_iter()
            .filter(|other| other != state.id())
            .collect();
        assert!(
            others.is_empty(),
            "{} shares ground with {others:?}",
            state.id()
        );
        for other in State::ALL {
            let theirs = registry.get(other.id()).unwrap().boundary();
            let mine = adapter.boundary();
            if other != state {
                let smaller = mine.unsigned_area().min(theirs.unsigned_area());
                largest = largest.max(mine.intersection(theirs).unsigned_area() / smaller);
            }
        }
    }
    // A tenth of the hundredth the loader allows: a boundary that changes this is looked at.
    assert!(largest < 0.001, "{largest}");
}

/// Every built-in manifest passes the validator, its boundary files are valid geometries, its
/// folder holds the files it names and no other, and it declares every host its state asks.
#[test]
fn builtin_manifests_pass_the_validator_and_policy() {
    let mut backends = Registry::new();
    states::register_backends(&mut backends).unwrap();
    for state in State::ALL {
        let id = state.id();
        let manifest = manifest(id);
        assert_eq!(manifest.adapter.id, id);
        assert_eq!(manifest.adapter.provider, state.agency().id(), "{id}");
        // The text built into the program is the file.
        let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("manifests")
            .join(id);
        let file = std::fs::read_to_string(folder.join("manifest.toml")).unwrap();
        assert_eq!(states::manifest_text(state), file, "{id}");
        let asked = recorded_hosts(state);
        assert!(!asked.is_empty(), "{id}");
        for host in &asked {
            assert!(
                manifest.hosts.contains(host),
                "{id} does not declare {host}"
            );
        }
        if matches!(
            state,
            State::Saarland | State::BadenWuerttemberg | State::Bremen
        ) {
            // No more than it asks, either: nothing recorded could hide a host here.
            assert_eq!(manifest.hosts.len(), asked.len(), "{id}");
        }
        let mut expected: Vec<String> = std::iter::once(manifest.boundary.file.clone())
            .chain(
                manifest
                    .osm
                    .iter()
                    .filter_map(|extract| extract.boundary.clone()),
            )
            .chain(std::iter::once("manifest.toml".to_string()))
            .collect();
        expected.sort();
        let mut present: Vec<String> = std::fs::read_dir(&folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        present.sort();
        assert_eq!(present, expected, "{id}");
        // Parsed now, a boundary that is no valid geometry is a finding.
        let adapter = ManifestAdapter::new(manifest, &StateFiles(state), Parse::Now, &backends)
            .unwrap_or_else(|findings| panic!("{id}: {findings:?}"));
        assert!(!adapter.boundary().0.is_empty(), "{id}");
        assert!(adapter.rasters().is_some(), "{id}");
        // Berlin, Baden-Württemberg and Bremen read their ALKIS with code.
        let own_alkis = matches!(
            state,
            State::Berlin | State::BadenWuerttemberg | State::Bremen
        );
        assert_eq!(adapter.alkis().is_some(), !own_alkis, "{id}");
    }
}

/// The adapter of a state answers the facts of its manifest, and the boundary the modules of
/// the agencies ask is the boundary of the manifest's file.
#[test]
fn every_manifest_names_its_state() {
    let mut backends = Registry::new();
    states::register_backends(&mut backends).unwrap();
    for state in State::ALL {
        let built_in = StateAdapter::new(state, &backends);
        let manifest = manifest(state.id());
        assert_eq!(
            &adapter::info_of(&manifest),
            built_in.info(),
            "{}",
            state.id()
        );
        let from_file =
            ManifestAdapter::new(manifest, &StateFiles(state), Parse::Now, &backends).unwrap();
        assert_eq!(from_file.boundary(), built_in.boundary(), "{}", state.id());
        assert_eq!(from_file.bounds(), built_in.bounds(), "{}", state.id());
        assert!(built_in.alkis().is_some(), "{}", state.id());
    }
    assert_eq!(State::ALL.len(), 14);
    assert_eq!(states::adapters().len(), 14);
}

/// Saarland asks the WFS of the geoportal with the type names of its ArcGIS server and
/// without a sort, which that server refuses.
#[test]
fn saarland_asks_the_wfs_of_the_geoportal_with_its_own_type_names() {
    let service = wfs_of(&manifest("saarland"));
    let saarbruecken = freemapper_georef::crs::BoundingBox {
        min_e: 353_875.0,
        min_n: 5_455_271.0,
        max_e: 354_075.0,
        max_n: 5_455_471.0,
    };
    let url = service.wfs_url(&service.buildings_type, &saarbruecken, 0);
    assert!(
        url.starts_with(
            "https://geoportal.saarland.de/registry/wfs/414?SERVICE=WFS&VERSION=2.0.0\
             &REQUEST=GetFeature&TYPENAMES=ALKIS_ALKIS_WFS_ohne_Eig:GebaeudeBauwerk\
             &SRSNAME=urn:ogc:def:crs:EPSG::25832&BBOX=353875,5455271,354075,5455471,"
        ),
        "{url}"
    );
    assert!(!url.contains("SORTBY"), "{url}");
    assert_eq!(service.land_use_type, "ALKIS_ALKIS_WFS_ohne_Eig:Nutzung");
}

/// A built-in state asks the hosts of its own manifest and no other: its policy lets every
/// URL of its manifest pass and refuses the hosts of the program's own requests and of the
/// other states.
#[test]
fn every_state_is_held_to_the_hosts_of_its_manifest() {
    let mut backends = Registry::new();
    states::register_backends(&mut backends).unwrap();
    for state in State::ALL {
        let built_in = StateAdapter::new(state, &backends);
        let policy = built_in
            .policy()
            .unwrap_or_else(|| panic!("{} has no policy", state.id()));
        let hosts = &built_in.declared().unwrap().hosts;
        assert!(!hosts.is_empty(), "{}", state.id());
        for host in hosts {
            let url = format!("https://{host}/x");
            assert!(policy.check_url(&url).is_ok(), "{}: {url}", state.id());
        }
        for other in [
            "https://download.geofabrik.de/x",
            "https://geo.example.org/x",
        ] {
            assert!(policy.check_url(other).is_err(), "{}: {other}", state.id());
        }
    }
    // Hamburg does not ask the servers of Brandenburg.
    let hamburg = StateAdapter::new(State::Hamburg, &backends);
    let policy = hamburg.policy().unwrap();
    assert!(policy.check_url("https://isk.geobasis-bb.de/x").is_err());
}

/// A host a person allowed for a state is one more host of that state and of no other; an
/// entry that is no public DNS name is left out. A host nobody allowed is refused with the
/// command that allows it.
#[test]
fn a_host_allowed_for_a_state_widens_that_state_only() {
    let mut extra = std::collections::BTreeMap::new();
    extra.insert(
        "hessen".to_string(),
        vec![
            "cdn.example.net".to_string(),
            "127.0.0.1".to_string(),
            "*".to_string(),
            "localhost".to_string(),
            "user@evil.example.net".to_string(),
            "evil.example.net:8443".to_string(),
        ],
    );
    let mut registry = Registry::new();
    states::register_with_hosts(&mut registry, &extra).unwrap();
    let policy = |id: &str| registry.get(id).unwrap().policy().unwrap();
    assert!(
        policy("hessen")
            .check_url("https://cdn.example.net/x")
            .is_ok()
    );
    assert!(
        policy("hamburg")
            .check_url("https://cdn.example.net/x")
            .is_err()
    );
    let hosts: Vec<&str> = policy("hessen").hosts().collect();
    let declared = &registry.get("hessen").unwrap().declared().unwrap().hosts;
    assert_eq!(hosts.len(), declared.len() + 1, "{hosts:?}");
    // What nobody allowed is refused, and the message has the command.
    let refused = policy("hessen")
        .check_url("https://other.example.net/x")
        .unwrap_err();
    let message = policy("hessen").refusal(refused, None).to_string();
    assert!(
        message.contains(
            "the adapter of Hesse does not name the host other.example.net. If the service \
             has moved there, allow the host with `freemapper adapters allow-host hessen \
             other.example.net` and report it at"
        ),
        "{message}"
    );
}
