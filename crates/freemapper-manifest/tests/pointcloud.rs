// SPDX-License-Identifier: AGPL-3.0-or-later

//! `[point_cloud]`: the addresses of a point-cloud backend (issue #273).

use std::path::Path;

use freemapper_manifest::{ManifestError, PointCloudKind, parse};

fn sample() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/invalid/unknown-key.toml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .replace("colour = \"red\"\n", "")
}

fn with(table: &str) -> Result<freemapper_manifest::Manifest, Vec<ManifestError>> {
    parse(&format!("{}\n{table}", sample()), Path::new("sample"))
}

fn codes(findings: &[ManifestError]) -> Vec<&'static str> {
    findings.iter().map(|finding| finding.code).collect()
}

fn keys(findings: &[ManifestError]) -> Vec<&str> {
    findings
        .iter()
        .map(|finding| finding.key.as_str())
        .collect()
}

#[test]
fn a_manifest_without_the_table_has_no_point_cloud() {
    assert_eq!(
        parse(&sample(), Path::new("sample")).unwrap().point_cloud,
        None
    );
}

#[test]
fn every_kind_is_read_with_its_addresses() {
    for (kind, index) in [
        ("laz-tiles", false),
        ("zip-archives", false),
        ("index-json", true),
        ("meta4", true),
        ("currency-csv", true),
    ] {
        let extra = if index {
            "index_url = \"https://geo.example.org/laz/index\"\n"
        } else {
            ""
        };
        let manifest = with(&format!(
            "[point_cloud]\nkind = \"{kind}\"\nurl = \"https://geo.example.org/laz\"\n{extra}"
        ))
        .unwrap_or_else(|findings| panic!("{kind}: {findings:?}"));
        let cloud = manifest.point_cloud.expect("the table");
        assert_eq!(cloud.kind.name(), kind);
        assert_eq!(cloud.kind.has_index(), index, "{kind}");
        assert_eq!(cloud.url, "https://geo.example.org/laz");
        assert_eq!(
            cloud.index_url.as_deref(),
            index.then_some("https://geo.example.org/laz/index"),
            "{kind}"
        );
    }
}

#[test]
fn the_kinds_are_the_five_the_manifest_names() {
    let names: Vec<&str> = PointCloudKind::ALL.map(PointCloudKind::name).to_vec();
    assert_eq!(
        names,
        [
            "laz-tiles",
            "index-json",
            "meta4",
            "currency-csv",
            "zip-archives"
        ]
    );
}

#[test]
fn a_kind_that_reads_an_index_needs_its_address() {
    for kind in ["index-json", "meta4", "currency-csv"] {
        let findings = with(&format!(
            "[point_cloud]\nkind = \"{kind}\"\nurl = \"https://geo.example.org/laz\"\n"
        ))
        .unwrap_err();
        assert_eq!(codes(&findings), ["manifest.missing-key"], "{kind}");
        assert_eq!(keys(&findings), ["point_cloud.index_url"], "{kind}");
    }
}

#[test]
fn a_kind_without_an_index_refuses_one() {
    for kind in ["laz-tiles", "zip-archives"] {
        let findings = with(&format!(
            "[point_cloud]\nkind = \"{kind}\"\nurl = \"https://geo.example.org/laz\"\n\
             index_url = \"https://geo.example.org/laz/index\"\n"
        ))
        .unwrap_err();
        assert_eq!(codes(&findings), ["manifest.unknown-key"], "{kind}");
        assert_eq!(keys(&findings), ["point_cloud.index_url"], "{kind}");
    }
}

#[test]
fn an_unknown_kind_names_the_known_ones() {
    let findings = with("[point_cloud]\nkind = \"tiles\"\nurl = \"https://geo.example.org/laz\"\n")
        .unwrap_err();
    assert_eq!(codes(&findings), ["manifest.value"]);
    let message = findings[0].to_string();
    for kind in PointCloudKind::ALL {
        assert!(message.contains(kind.name()), "{message}");
    }
}

#[test]
fn an_unknown_kind_with_an_index_reports_only_the_kind() {
    let findings = with(
        "[point_cloud]\nkind = \"tiles\"\nurl = \"https://geo.example.org/laz\"\n\
         index_url = \"https://geo.example.org/laz/index\"\n",
    )
    .unwrap_err();
    assert_eq!(codes(&findings), ["manifest.value"]);
}

#[test]
fn the_folder_has_no_trailing_slash() {
    let findings =
        with("[point_cloud]\nkind = \"laz-tiles\"\nurl = \"https://geo.example.org/laz/\"\n")
            .unwrap_err();
    assert_eq!(codes(&findings), ["manifest.value"]);
    assert_eq!(keys(&findings), ["point_cloud.url"]);
}

#[test]
fn the_addresses_obey_the_url_rules() {
    let query =
        with("[point_cloud]\nkind = \"laz-tiles\"\nurl = \"https://geo.example.org/laz?x=1\"\n")
            .unwrap_err();
    assert_eq!(codes(&query), ["policy.url"]);
    let host =
        with("[point_cloud]\nkind = \"laz-tiles\"\nurl = \"https://other.example.org/laz\"\n")
            .unwrap_err();
    assert_eq!(codes(&host), ["policy.host-not-declared"]);
    let index = with(
        "[point_cloud]\nkind = \"meta4\"\nurl = \"https://geo.example.org/laz\"\n\
         index_url = \"https://other.example.org/laz.meta4\"\n",
    )
    .unwrap_err();
    assert_eq!(codes(&index), ["policy.host-not-declared"]);
    assert_eq!(keys(&index), ["point_cloud.index_url"]);
}

#[test]
fn an_unknown_key_of_the_table_is_refused() {
    let findings = with(
        "[point_cloud]\nkind = \"laz-tiles\"\nurl = \"https://geo.example.org/laz\"\ncolour = 1\n",
    )
    .unwrap_err();
    assert_eq!(codes(&findings), ["manifest.unknown-key"]);
}

#[test]
fn the_table_needs_its_kind_and_folder() {
    assert!(with("[point_cloud]\nurl = \"https://geo.example.org/laz\"\n").is_err());
    assert!(with("[point_cloud]\nkind = \"laz-tiles\"\n").is_err());
}
