//! The manifests of the German states against the constants they were written from: the
//! services a manifest builds are the services the code names.

use std::borrow::Cow;
use std::path::Path;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapters_de::states::alkis::{
    BUILDINGS_ENDPOINT, LAND_USE_ENDPOINT, MV_BUILDINGS_ENDPOINT, MV_LAND_USE_ENDPOINT,
};
use freemapper_adapters_de::states::{State, StateAdapter};
use freemapper_adapters_de::{lgv, states};
use freemapper_georef::crs::Crs;
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

/// The states that have a manifest so far (Berlin follows with its code adapter).
const WITH_MANIFEST: [State; 10] = [
    State::Brandenburg,
    State::Saxony,
    State::SaxonyAnhalt,
    State::Hamburg,
    State::Hessen,
    State::SchleswigHolstein,
    State::MecklenburgVorpommern,
    State::NordrheinWestfalen,
    State::Niedersachsen,
    State::RheinlandPfalz,
];

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
fn manifest_wfs_rows_equal_the_nine_constants() {
    let rows: [(&str, WfsService); 9] = [
        ("saxony", wfs::GEOSN),
        ("saxonyanhalt", wfs::LVERMGEO),
        ("hamburg", wfs::LGV),
        ("hessen", wfs::HVBG),
        ("schleswigholstein", wfs::LVERMGEO_SH),
        ("mecklenburgvorpommern", wfs::LAIV_MV),
        ("nordrheinwestfalen", wfs::GEOBASIS_NRW),
        ("niedersachsen", wfs::LGLN),
        ("rheinlandpfalz", wfs::LVERMGEO_RP),
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
    // No other state has such sources.
    for state in WITH_MANIFEST {
        if state != State::Brandenburg {
            let manifest = manifest(state.id());
            assert!(
                manifest.nas.is_empty() && manifest.trees.is_empty(),
                "{}",
                state.id()
            );
        }
    }
}

/// What a manifest says about its state is what the adapter of the state answers today: ids,
/// names, CRS, object-id prefix, the extracts and the preview.
#[test]
fn every_manifest_names_its_state() {
    for state in WITH_MANIFEST {
        let today = StateAdapter::new(state);
        assert_eq!(
            &adapter::info_of(&manifest(state.id())),
            today.info(),
            "{}",
            state.id()
        );
    }
    assert_eq!(states::adapters().len(), WITH_MANIFEST.len() + 1);
}
