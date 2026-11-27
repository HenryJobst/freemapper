// SPDX-License-Identifier: AGPL-3.0-or-later

//! `[terrain]` without `[orthophoto]`: a region that has a terrain model and no orthophoto
//! (Sweden), and the addresses its backend asks.

use std::path::Path;

use freemapper_manifest::model::HeightModel;
use freemapper_manifest::{ManifestError, parse};

/// The sample without its raster tables.
fn head() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/invalid/unknown-key.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .replace("colour = \"red\"\n", "");
    text[..text
        .find("[orthophoto]")
        .expect("the sample has an orthophoto")]
        .to_string()
}

fn with(rest: &str) -> Result<freemapper_manifest::Manifest, Vec<ManifestError>> {
    parse(&format!("{}{rest}", head()), Path::new("sample"))
}

fn keys(findings: &[ManifestError]) -> Vec<&str> {
    findings
        .iter()
        .map(|finding| finding.key.as_str())
        .collect()
}

const TERRAIN: &str = "[terrain]\nbackend = \"lantmateriet\"\n\
    endpoints = { stac-hojd = \"https://geo.example.org/stac-hojd\", stac-vektor = \"https://geo.example.org/stac-vektor\" }\n";

#[test]
fn a_terrain_model_alone_is_a_valid_raster_set() {
    let manifest = with(TERRAIN).unwrap_or_else(|findings| panic!("{findings:?}"));
    assert_eq!(manifest.orthophoto, None);
    let Some(HeightModel::Backend { name, config }) = manifest.terrain else {
        panic!("a backend terrain");
    };
    assert_eq!(name, "lantmateriet");
    assert_eq!(
        config.endpoint("stac-hojd"),
        Some("https://geo.example.org/stac-hojd")
    );
    assert_eq!(
        config.endpoint("stac-vektor"),
        Some("https://geo.example.org/stac-vektor")
    );
}

#[test]
fn a_surface_model_without_a_terrain_model_is_still_refused() {
    let findings = with("[surface]\nbackend = \"lantmateriet\"\n").unwrap_err();
    assert_eq!(keys(&findings), ["terrain"]);
}

#[test]
fn a_surface_model_of_another_backend_beside_the_terrain_is_refused() {
    let findings = with(&format!("{TERRAIN}[surface]\nbackend = \"lgb\"\n")).unwrap_err();
    assert_eq!(keys(&findings), ["terrain"]);
}

#[test]
fn the_endpoints_of_a_terrain_are_addresses_on_declared_hosts() {
    let findings = with(
        "[terrain]\nbackend = \"lantmateriet\"\nendpoints = { stac-hojd = \"https://elsewhere.example.net/x\" }\n",
    )
    .unwrap_err();
    assert_eq!(keys(&findings), ["terrain.endpoints.stac-hojd"]);
}

#[test]
fn a_name_the_backend_does_not_ask_is_refused() {
    let findings = with(
        "[terrain]\nbackend = \"lantmateriet\"\nendpoints = { other = \"https://geo.example.org/x\" }\n",
    )
    .unwrap_err();
    assert_eq!(keys(&findings), ["terrain.endpoints.other"]);
}

#[test]
fn terrain_endpoints_beside_an_orthophoto_are_refused() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/invalid/unknown-key.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap()
        .replace("colour = \"red\"\n", "");
    let findings = parse(
        &format!(
            "{}[orthophoto]\nbackend = \"lantmateriet\"\n{TERRAIN}",
            &text[..text.find("[orthophoto]").unwrap()]
        ),
        Path::new("sample"),
    )
    .unwrap_err();
    assert_eq!(keys(&findings), ["terrain.endpoints"]);
}
