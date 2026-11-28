// SPDX-License-Identifier: AGPL-3.0-or-later

//! `[auth]`: the host that takes HTTP Basic credentials and the variables they come from.

use std::path::Path;

use freemapper_manifest::{ManifestError, parse};

fn sample() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/invalid/unknown-key.toml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .replace("colour = \"red\"\n", "")
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

const GOOD: &str = "[auth]\nhost = \"geo.example.org\"\nuser_env = \"FREEMAPPER_X_USER\"\n\
    password_env = \"FREEMAPPER_X_PASSWORD\"\nproducts = [\"Product A\", \"Product B\"]\n";

#[test]
fn a_manifest_without_the_table_has_no_auth() {
    assert_eq!(parse(&sample(), Path::new("sample")).unwrap().auth, None);
}

#[test]
fn the_table_is_read_with_its_host_variables_and_products() {
    let auth = with(GOOD).unwrap().auth.expect("the table");
    assert_eq!(auth.host, "geo.example.org");
    assert_eq!(auth.user_env, "FREEMAPPER_X_USER");
    assert_eq!(auth.password_env, "FREEMAPPER_X_PASSWORD");
    assert_eq!(auth.products, ["Product A", "Product B"]);
}

#[test]
fn a_host_outside_the_declared_hosts_is_refused() {
    let findings = with(&GOOD.replace("geo.example.org", "elsewhere.example.net")).unwrap_err();
    assert_eq!(keys(&findings), ["auth.host"]);
}

#[test]
fn a_variable_must_start_with_freemapper() {
    for name in [
        "HOME",
        "PATH",
        "freemapper_x",
        "FREEMAPPER_",
        "FREEMAPPER_a b",
    ] {
        let findings = with(&GOOD.replace("FREEMAPPER_X_USER", name)).unwrap_err();
        assert_eq!(keys(&findings), ["auth.user_env"], "{name}");
    }
}

#[test]
fn one_to_six_products_are_named() {
    assert_eq!(
        keys(
            &with(&GOOD.replace("products = [\"Product A\", \"Product B\"]", "products = []"))
                .unwrap_err()
        ),
        ["auth.products"]
    );
    let seven = (1..=7)
        .map(|n| format!("\"P{n}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let findings = with(&GOOD.replace("\"Product A\", \"Product B\"", &seven)).unwrap_err();
    assert_eq!(keys(&findings), ["auth.products"]);
}
