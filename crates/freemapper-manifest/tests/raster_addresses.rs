// SPDX-License-Identifier: AGPL-3.0-or-later

//! `[orthophoto]` of a backend with `url`, `layer` and `max_px`: where the backend asks for the
//! orthophoto (issue #273).

use std::path::Path;

use freemapper_adapter_api::rasters::RasterConfig;
use freemapper_manifest::model::Orthophoto;
use freemapper_manifest::{ManifestError, TAKES_ADDRESSES, TAKES_ENDPOINTS, parse};

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

fn with(orthophoto: &str, rest: &str) -> Result<freemapper_manifest::Manifest, Vec<ManifestError>> {
    parse(
        &format!("{}[orthophoto]\n{orthophoto}\n{rest}", head()),
        Path::new("sample"),
    )
}

/// A manifest whose three raster tables name `backend`, with `orthophoto` besides in the
/// first.
fn backend(
    name: &str,
    orthophoto: &str,
) -> Result<freemapper_manifest::Manifest, Vec<ManifestError>> {
    with(
        &format!("backend = \"{name}\"\n{orthophoto}"),
        &format!("[terrain]\nbackend = \"{name}\"\n[surface]\nbackend = \"{name}\"\n"),
    )
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

fn config_of(manifest: &freemapper_manifest::Manifest) -> RasterConfig {
    match manifest.orthophoto.as_ref().expect("an orthophoto") {
        Orthophoto::Backend { config, .. } => config.clone(),
        other => panic!("not a backend: {other:?}"),
    }
}

const ALL: &str = "url = \"https://geo.example.org/dop/wms\"\nlayer = \"dop_20\"\nmax_px = 3000\n";

#[test]
fn the_backends_that_take_addresses_are_those_of_the_states() {
    assert_eq!(
        TAKES_ADDRESSES,
        [
            "geosn",
            "hvbg",
            "lvermgeo-rp",
            "lgl-bw",
            "lvermgeo-sh",
            "lvgl",
            "geobasis-nrw",
            "ldbv",
            "lgln",
            "lgv",
        ]
    );
}

#[test]
fn a_backend_that_takes_addresses_reads_all_three_keys() {
    for name in TAKES_ADDRESSES {
        let manifest = backend(name, ALL).unwrap_or_else(|findings| panic!("{name}: {findings:?}"));
        assert_eq!(
            config_of(&manifest),
            RasterConfig {
                url: Some("https://geo.example.org/dop/wms".to_string()),
                layer: Some("dop_20".to_string()),
                max_px: Some(3000),
                ..RasterConfig::default()
            },
            "{name}"
        );
    }
}

#[test]
fn each_key_stands_alone_and_a_missing_one_stays_none() {
    let url = config_of(&backend("geosn", "url = \"https://geo.example.org/wms\"\n").unwrap());
    assert_eq!(url.url.as_deref(), Some("https://geo.example.org/wms"));
    assert_eq!((url.layer, url.max_px), (None, None));
    let layer = config_of(&backend("geosn", "layer = \"dop\"\n").unwrap());
    assert_eq!(layer.layer.as_deref(), Some("dop"));
    assert_eq!((layer.url, layer.max_px), (None, None));
    let size = config_of(&backend("geosn", "max_px = 2048\n").unwrap());
    assert_eq!(size.max_px, Some(2048));
    assert_eq!((size.url, size.layer), (None, None));
    assert!(config_of(&backend("geosn", "").unwrap()).is_empty());
}

#[test]
fn the_edges_of_max_px_are_256_and_10000() {
    for (value, fine) in [(255, false), (256, true), (10_000, true), (10_001, false)] {
        let result = backend("geosn", &format!("max_px = {value}\n"));
        assert_eq!(result.is_ok(), fine, "{value}: {result:?}");
        if let Err(findings) = result {
            assert_eq!(keys(&findings), ["orthophoto.max_px"], "{value}");
        }
    }
}

#[test]
fn a_url_off_the_declared_hosts_or_with_a_query_is_refused() {
    for url in [
        "https://elsewhere.example.net/wms",
        "https://geo.example.org/wms?SERVICE=WMS",
        "http://geo.example.org/wms",
    ] {
        let findings = backend("geosn", &format!("url = \"{url}\"\n")).unwrap_err();
        assert_eq!(keys(&findings), ["orthophoto.url"], "{url}");
    }
}

#[test]
fn a_layer_is_a_plain_name() {
    for layer in ["", "a b", "x&y"] {
        let findings = backend("geosn", &format!("layer = \"{layer}\"\n")).unwrap_err();
        assert_eq!(keys(&findings), ["orthophoto.layer"], "{layer:?}");
        assert_eq!(codes(&findings), ["manifest.value"], "{layer:?}");
    }
}

#[test]
fn a_backend_that_keeps_its_addresses_in_code_takes_none() {
    for name in ["lgb", "laiv-mv", "geobremen"] {
        for line in [
            "url = \"https://geo.example.org/wms\"\n",
            "layer = \"dop\"\n",
            "max_px = 2048\n",
        ] {
            let findings = backend(name, line).unwrap_err();
            let key = line.split(' ').next().unwrap();
            assert_eq!(
                keys(&findings),
                [format!("orthophoto.{key}")],
                "{name} {line}"
            );
        }
        assert!(backend(name, "").is_ok(), "{name}");
    }
}

#[test]
fn the_other_tables_of_a_backend_take_no_address() {
    for key in [
        "url = \"https://geo.example.org/wms\"",
        "layer = \"dop\"",
        "max_px = 2048",
    ] {
        let name = key.split(' ').next().unwrap();
        for table in ["terrain", "surface"] {
            let (terrain, surface) = match table {
                "terrain" => (
                    format!("backend = \"geosn\"\n{key}\n"),
                    "backend = \"geosn\"\n".to_string(),
                ),
                _ => (
                    "backend = \"geosn\"\n".to_string(),
                    format!("backend = \"geosn\"\n{key}\n"),
                ),
            };
            let findings = with(
                "backend = \"geosn\"\n",
                &format!("[terrain]\n{terrain}[surface]\n{surface}"),
            )
            .unwrap_err();
            assert_eq!(
                keys(&findings),
                [format!("{table}.{name}")],
                "{table} {key}"
            );
        }
    }
}

#[test]
fn a_backend_table_still_refuses_the_keys_of_a_service() {
    for line in [
        "coverage = \"dop\"",
        "crs = 25832",
        "label = \"x\"",
        "multipart = true",
    ] {
        let findings = backend("geosn", &format!("{line}\n")).unwrap_err();
        assert_eq!(findings.len(), 1, "{line}: {findings:?}");
    }
}

/// One `endpoints` line of a backend that names `names`, each at its own address.
fn endpoints_line(names: &[&str]) -> String {
    let pairs: Vec<String> = names
        .iter()
        .map(|name| format!("{name} = \"https://geo.example.org/{name}\""))
        .collect();
    format!("endpoints = {{ {} }}\n", pairs.join(", "))
}

#[test]
fn the_backends_that_take_endpoints_and_the_names_they_ask() {
    assert_eq!(
        TAKES_ENDPOINTS.map(|(name, names)| (name, names.to_vec())),
        [
            ("lgv", vec!["metadata", "dgm1", "bdom"]),
            ("geobasis-nrw", vec!["hm"]),
            (
                "lgln",
                vec!["dop-stac", "dgm-stac", "dom-stac", "dgm-wcs", "dom-wcs"]
            ),
            ("laiv-mv", vec!["overview", "dop-wcs", "dgm-wcs", "dom-wcs"]),
            ("geobremen", vec!["wms", "files"]),
            ("lgb", vec!["data", "currency"]),
            ("lantmateriet", vec!["stac-hojd", "stac-vektor"]),
        ]
    );
}

#[test]
fn a_backend_reads_the_endpoints_it_asks_and_no_other() {
    for (name, names) in TAKES_ENDPOINTS {
        let manifest = backend(name, &endpoints_line(names))
            .unwrap_or_else(|findings| panic!("{name}: {findings:?}"));
        let config = config_of(&manifest);
        assert_eq!(config.endpoints.len(), names.len(), "{name}");
        for endpoint in names {
            assert_eq!(
                config.endpoint(endpoint),
                Some(format!("https://geo.example.org/{endpoint}").as_str()),
                "{name} {endpoint}"
            );
        }
        // One name alone is fine, and none leaves the map empty.
        let one = config_of(&backend(name, &endpoints_line(&names[..1])).unwrap());
        assert_eq!(one.endpoints.len(), 1, "{name}");
        assert!(config_of(&backend(name, "").unwrap()).endpoints.is_empty());
        // A name the backend does not ask is refused, at its own key.
        let findings = backend(name, &endpoints_line(&["elsewhere"])).unwrap_err();
        assert_eq!(
            keys(&findings),
            ["orthophoto.endpoints.elsewhere"],
            "{name}"
        );
        assert_eq!(codes(&findings), ["manifest.unknown-key"], "{name}");
    }
}

#[test]
fn an_endpoint_is_a_plain_address_on_a_declared_host() {
    for url in [
        "https://elsewhere.example.net/f.zip",
        "https://geo.example.org/f.zip?x=1",
        "http://geo.example.org/f.zip",
    ] {
        let findings =
            backend("lgv", &format!("endpoints = {{ dgm1 = \"{url}\" }}\n")).unwrap_err();
        assert_eq!(keys(&findings), ["orthophoto.endpoints.dgm1"], "{url}");
    }
}

#[test]
fn a_backend_without_endpoints_and_the_other_tables_take_none() {
    let line = endpoints_line(&["metadata"]);
    for name in ["geosn", "ldbv"] {
        let findings = backend(name, &line).unwrap_err();
        assert_eq!(keys(&findings), ["orthophoto.endpoints"], "{name}");
    }
    for table in ["terrain", "surface"] {
        let (terrain, surface) = match table {
            "terrain" => (
                format!("backend = \"lgv\"\n{line}"),
                "backend = \"lgv\"\n".to_string(),
            ),
            _ => (
                "backend = \"lgv\"\n".to_string(),
                format!("backend = \"lgv\"\n{line}"),
            ),
        };
        let findings = with(
            "backend = \"lgv\"\n",
            &format!("[terrain]\n{terrain}[surface]\n{surface}"),
        )
        .unwrap_err();
        assert_eq!(keys(&findings), [format!("{table}.endpoints")], "{table}");
    }
    let service = with(
        &format!(
            "kind = \"wms\"\nurl = \"https://geo.example.org/wms\"\nlayer = \"l\"\ncrs = 25832\nmax_px = 2000\n{line}"
        ),
        "[terrain]\nbackend = \"lgv\"\n",
    );
    let findings = service.unwrap_err();
    assert!(
        keys(&findings).contains(&"orthophoto.endpoints"),
        "{findings:?}"
    );
}
