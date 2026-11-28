// SPDX-License-Identifier: AGPL-3.0-or-later

//! `[alkis] kind = "stac-gpkg"`: land cover as GeoPackage files a STAC API lists.

use std::path::Path;

use freemapper_manifest::model::Alkis;
use freemapper_manifest::{ManifestError, parse};

/// The sample without its `[alkis]` table.
fn sample() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/invalid/unknown-key.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .replace("colour = \"red\"\n", "");
    let from = text.find("[alkis]").expect("the sample has an alkis table");
    let to = text.find("[orthophoto]").expect("and an orthophoto");
    format!("{}{}", &text[..from], &text[to..])
}

fn with(table: &str) -> Result<freemapper_manifest::Manifest, Vec<ManifestError>> {
    parse(&format!("{}\n{table}", sample()), Path::new("sample"))
}

fn keys(findings: &[ManifestError]) -> Vec<&str> {
    findings
        .iter()
        .map(|finding| finding.key.as_str())
        .collect()
}

const GOOD: &str = "[alkis]\nkind = \"stac-gpkg\"\nstac = \"https://geo.example.org/stac-vektor\"\n\
    collection = \"marktacke\"\n";

#[test]
fn the_table_is_read_with_its_catalogue_and_collection() {
    let manifest = with(GOOD).unwrap_or_else(|findings| panic!("{findings:?}"));
    assert_eq!(
        manifest.alkis,
        Some(Alkis::StacGpkg {
            stac: "https://geo.example.org/stac-vektor".to_string(),
            collection: "marktacke".to_string(),
        })
    );
}

#[test]
fn both_keys_are_needed() {
    let findings = with("[alkis]\nkind = \"stac-gpkg\"\ncollection = \"marktacke\"\n").unwrap_err();
    assert_eq!(keys(&findings), ["alkis.stac"]);
    let findings =
        with("[alkis]\nkind = \"stac-gpkg\"\nstac = \"https://geo.example.org/stac-vektor\"\n")
            .unwrap_err();
    assert_eq!(keys(&findings), ["alkis.collection"]);
}

#[test]
fn the_catalogue_lies_on_a_declared_host_without_a_query() {
    let findings = with(&GOOD.replace("geo.example.org", "elsewhere.example.net")).unwrap_err();
    assert_eq!(keys(&findings), ["alkis.stac"]);
    let findings = with(&GOOD.replace("stac-vektor\"", "stac-vektor?x=1\"")).unwrap_err();
    assert_eq!(keys(&findings), ["alkis.stac"]);
}

#[test]
fn the_collection_is_a_plain_name() {
    let findings = with(&GOOD.replace("marktacke", "../x")).unwrap_err();
    assert_eq!(keys(&findings), ["alkis.collection"]);
}

#[test]
fn a_key_of_another_kind_is_refused() {
    let findings = with(&format!("{GOOD}url = \"https://geo.example.org/wfs\"\n")).unwrap_err();
    assert_eq!(keys(&findings), ["alkis.url"]);
}

#[test]
fn the_keys_belong_to_no_other_kind() {
    let findings = with(
        "[alkis]\nkind = \"wfs\"\nname = \"x\"\nurl = \"https://geo.example.org/wfs\"\n\
         buildings_type = \"a:b\"\nland_use_type = \"a:c\"\nstac = \"https://geo.example.org/s\"\n",
    )
    .unwrap_err();
    assert_eq!(keys(&findings), ["alkis.stac"]);
}
