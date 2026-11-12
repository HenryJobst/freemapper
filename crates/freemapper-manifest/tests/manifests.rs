//! The parser and validator of the adapter manifest against the sample manifests of the design
//! and against manifests with one defect each.

use std::path::{Path, PathBuf};

use freemapper_georef::crs::Crs;
use freemapper_manifest::model::{
    Alkis, BoundaryCrs, DateFormat, DateSource, HeightModel, Licence, Normalise, Orthophoto,
    Version,
};
use freemapper_manifest::{ManifestError, parse};

fn folder(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(name)
}

fn valid(id: &str) -> String {
    std::fs::read_to_string(folder("valid").join(format!("{id}.toml"))).unwrap()
}

/// The codes of the findings, with the whole finding in the failure message.
fn codes(findings: &[ManifestError]) -> Vec<&'static str> {
    findings.iter().map(|finding| finding.code).collect()
}

/// A manifest with one defect, built from the smallest valid one.
fn sample() -> String {
    let text = std::fs::read_to_string(folder("invalid").join("unknown-key.toml")).unwrap();
    text.replace("colour = \"red\"\n", "")
}

#[test]
fn valid_manifests_parse() {
    let hamburg = parse(&valid("hamburg"), Path::new("adapters/hamburg")).unwrap();
    assert_eq!(hamburg.adapter.id, "hamburg");
    assert_eq!(hamburg.adapter.provider, "lgv");
    assert_eq!(hamburg.adapter.crs, Crs::Utm32);
    assert_eq!(hamburg.adapter.min_freemapper, Version(0, 13, 0));
    assert_eq!(hamburg.adapter.object_id_prefix.as_deref(), Some("DEHH"));
    assert_eq!(hamburg.adapter.maintainers, ["github:HenryJobst"]);
    assert_eq!(
        hamburg.adapter.last_verified.unwrap().to_string(),
        "2026-11-12"
    );
    assert_eq!(
        hamburg.hosts,
        [
            "geodienste.hamburg.de",
            "api.hamburg.de",
            "www.daten-hamburg.de"
        ]
    );
    assert_eq!(hamburg.boundary.file, "boundary.geojson");
    assert_eq!(hamburg.boundary.crs, BoundaryCrs::Utm33);
    assert_eq!(hamburg.boundary.simplify_m, 25.0);
    assert_eq!(hamburg.osm.len(), 1);
    assert_eq!(hamburg.osm[0].geofabrik, "europe/germany/hamburg");
    assert_eq!(hamburg.osm[0].cache_prefix, "hamburg");
    assert_eq!(hamburg.osm[0].megabytes, 54.0);
    assert_eq!(hamburg.attribution.licence, Licence::DlDeBy20);
    assert!(
        hamburg
            .attribution
            .text
            .contains("(LGV) {year}, dl-de/by-2-0")
    );
    let Some(Alkis::OgcApi {
        buildings,
        crs,
        normalise,
        fallback: Some(fallback),
        ..
    }) = &hamburg.alkis
    else {
        panic!("{:?}", hamburg.alkis);
    };
    assert!(buildings.ends_with("/collections/GebaeudeBauwerk/items"));
    assert_eq!((*crs, *normalise), (Crs::Utm32, Normalise::HamburgAscii));
    // The fallback answers in the CRS of the API and has no square name.
    assert_eq!(fallback.name, "LGV ALKIS (WFS)");
    assert_eq!(fallback.crs, Crs::Utm32);
    assert_eq!(fallback.square_bez, None);
    let Some(Orthophoto::Wms {
        layer,
        params,
        max_px,
        date,
        ..
    }) = &hamburg.orthophoto
    else {
        panic!("{:?}", hamburg.orthophoto);
    };
    assert_eq!(layer, "dop_zeitreihe_unbelaubt");
    assert_eq!(params, &[("TIME".to_string(), "2026".to_string())]);
    assert_eq!(*max_px, 4000);
    assert_eq!(
        date,
        &DateSource::Backend {
            name: "lgv".to_string()
        }
    );
    let backend = HeightModel::Backend {
        name: "lgv".to_string(),
    };
    assert_eq!(hamburg.terrain.as_ref(), Some(&backend));
    assert_eq!(hamburg.surface.as_ref(), Some(&backend));
    let preview = hamburg.preview.unwrap();
    assert_eq!(preview.format, "image/jpeg");
    assert_eq!(preview.params, [("time".to_string(), "2026".to_string())]);
    assert_eq!(preview.agency, "LGV");

    let nrw = parse(
        &valid("nordrheinwestfalen"),
        Path::new("nordrheinwestfalen"),
    )
    .unwrap();
    assert_eq!(nrw.adapter.names.de, "Nordrhein-Westfalen");
    assert_eq!(nrw.osm.len(), 5);
    assert_eq!(nrw.osm[2].label, "North Rhine-Westphalia, Düsseldorf");
    assert_eq!(
        nrw.osm[2].boundary.as_deref(),
        Some("osm-duesseldorf.geojson")
    );
    assert_eq!(nrw.osm[2].cache_prefix, "nrw-duesseldorf");
    assert_eq!(nrw.osm[0].megabytes, 195.2);
    assert_eq!(nrw.attribution.licence, Licence::DlDeZero20);
    let Some(Alkis::Wfs(wfs)) = &nrw.alkis else {
        panic!("{:?}", nrw.alkis);
    };
    assert_eq!(wfs.name, "Geobasis NRW ALKIS");
    assert_eq!(
        wfs.square_bez.as_deref(),
        Some("ohne Unterart (Nordrhein-Westfalen)")
    );
    let Some(Orthophoto::Wms { date, .. }) = &nrw.orthophoto else {
        panic!("{:?}", nrw.orthophoto);
    };
    assert_eq!(
        date,
        &DateSource::WmsFeatureInfo {
            layer: "nw_dop_utm_info".to_string(),
            field: "Bildflugdatum".to_string()
        }
    );
    // `kind = "backend"` and a bare `backend` say the same.
    assert_eq!(nrw.terrain, nrw.surface);

    let st = parse(&valid("saxonyanhalt"), Path::new("saxonyanhalt")).unwrap();
    assert_eq!(
        st.adapter.licence_of_manifest.as_deref(),
        Some("GPL-3.0-or-later")
    );
    assert_eq!(st.attribution.short.as_deref(), Some("LVermGeo ST"));
    let Some(Orthophoto::Wms { date, params, .. }) = &st.orthophoto else {
        panic!("{:?}", st.orthophoto);
    };
    assert!(params.is_empty());
    assert_eq!(
        date,
        &DateSource::WfsGeoJson {
            url: "https://geodatenportal.sachsen-anhalt.de/ows_lvermgeo_wfs_st_geobasis"
                .to_string(),
            layer: "atkis_geobasis_bestand:truedop_kacheln".to_string(),
            field: "BFDATUM".to_string(),
            format: DateFormat::DayMonthYear,
        }
    );
    let Some(HeightModel::Wcs {
        coverage,
        multipart,
        envelope,
        max_m,
        date,
        source_label,
        ..
    }) = &st.terrain
    else {
        panic!("{:?}", st.terrain);
    };
    assert_eq!(coverage, "Coverage1");
    assert!(*multipart);
    assert_eq!(*envelope, [606_000.0, 5_646_000.0, 790_000.0, 5_882_000.0]);
    assert_eq!(*max_m, 4000.0);
    assert_eq!(date, &DateSource::None);
    assert_eq!(source_label.as_deref(), Some("lvermgeo-st-dgm1"));
    assert!(matches!(
        &st.surface,
        Some(HeightModel::Wcs {
            date: DateSource::WfsGeoJson { .. },
            source_label: None,
            ..
        })
    ));
    assert_eq!(st.estimate.terrain_mb_per_km2, Some(4.0));
    assert_eq!(hamburg.estimate.terrain_mb_per_km2, None);
}

/// Every file under `tests/invalid` has one defect and names the code of its finding in the
/// first line. The parser reports that finding and no other, with the file and the line.
#[test]
fn invalid_manifests_fail_with_their_code() {
    let mut files: Vec<PathBuf> = std::fs::read_dir(folder("invalid"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    files.sort();
    assert!(files.len() >= 30, "{} files", files.len());
    let adapter = Path::new("adapters/sample");
    for file in files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&file).unwrap();
        let expected = text
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("# expect: "))
            .unwrap_or_else(|| panic!("{name} names no code"));
        let findings = parse(&text, adapter).expect_err(&name);
        assert_eq!(codes(&findings), [expected], "{name}: {findings:?}");
        let finding = &findings[0];
        assert_eq!(finding.path, adapter.join("manifest.toml"), "{name}");
        assert!(!finding.message.is_empty(), "{name}");
        assert!(finding.line <= text.lines().count(), "{name}: {finding:?}");
    }
}

/// The place of a finding: the line of the value, and the key with its tables.
#[test]
fn a_finding_names_the_line_and_the_key() {
    let read = |name: &str| {
        let text = std::fs::read_to_string(folder("invalid").join(name)).unwrap();
        let mut findings = parse(&text, Path::new("sample")).unwrap_err();
        let finding = findings.remove(0);
        let line = text
            .lines()
            .nth(finding.line.wrapping_sub(1))
            .unwrap_or("")
            .to_string();
        (finding, line)
    };
    let (finding, line) = read("url-http.toml");
    assert_eq!(finding.key, "orthophoto.url");
    assert_eq!(line, "url = \"http://geo.example.org/dop/wms\"");
    assert_eq!(
        finding.to_string(),
        format!(
            "{}:{}: orthophoto.url: http://geo.example.org/dop/wms: not https [policy.url]",
            Path::new("sample").join("manifest.toml").display(),
            finding.line
        )
    );
    let (finding, line) = read("id-pattern.toml");
    assert_eq!(
        (finding.key.as_str(), line.as_str()),
        ("adapter.id", "id = \"Sample_1\"")
    );
    let (finding, line) = read("unknown-key.toml");
    assert_eq!(
        (finding.key.as_str(), line.as_str()),
        ("colour", "colour = \"red\"")
    );
    let (finding, line) = read("extracts-without-boundary.toml");
    assert_eq!(finding.key, "osm.extract.boundary");
    assert_eq!(line, "geofabrik = \"europe/germany/hamburg\"");
    let (finding, line) = read("host-wildcard.toml");
    assert_eq!(
        (finding.key.as_str(), line.as_str()),
        ("net.hosts", "hosts = [\"*.example.org\"]")
    );
    let (finding, _) = read("empty.toml");
    assert_eq!((finding.key.as_str(), finding.line), ("", 0));
}

/// All findings of the validator come back at once.
#[test]
fn the_validator_reports_every_finding() {
    let text = sample()
        .replace("id = \"sample\"", "id = \"9\"")
        .replace(
            "https://geo.example.org/dop/wms",
            "http://geo.example.org/dop/wms",
        )
        .replace("licence = \"dl-de/by-2-0\"", "licence = \"mine\"");
    let findings = parse(&text, Path::new("sample")).unwrap_err();
    assert_eq!(
        codes(&findings),
        ["manifest.id", "manifest.licence", "policy.url"],
        "{findings:?}"
    );
}

/// Editors on Windows write a byte order mark and CRLF line ends; neither is a defect.
#[test]
fn a_byte_order_mark_and_crlf_line_ends_are_read() {
    let plain = parse(&sample(), Path::new("sample")).unwrap();
    let crlf = sample().replace('\n', "\r\n");
    assert_eq!(parse(&crlf, Path::new("sample")).unwrap(), plain);
    let marked = format!("\u{feff}{}", sample());
    assert_eq!(parse(&marked, Path::new("sample")).unwrap(), plain);
    // The line of a finding counts from the first line all the same.
    let broken = format!("\u{feff}{}", sample()).replace("crs = 25832", "crs = 4326");
    let finding = parse(&broken, Path::new("sample")).unwrap_err().remove(0);
    assert_eq!(broken.lines().nth(finding.line - 1), Some("crs = 4326"));
}

#[test]
fn a_manifest_over_64_kib_is_not_parsed() {
    let mut text = sample();
    while text.len() <= 64 * 1024 {
        text.push_str("# a long comment that makes the file grow line by line\n");
    }
    let findings = parse(&text, Path::new("sample")).unwrap_err();
    assert_eq!(codes(&findings), ["manifest.too-large"], "{findings:?}");
    // One byte less than the limit is read.
    let fits = format!("{}{}", sample(), "#".repeat(64 * 1024 - sample().len()));
    assert_eq!(fits.len(), 64 * 1024);
    assert!(parse(&fits, Path::new("sample")).is_ok());
}

/// A manifest holds at most 64 URLs; nine tree cadastres with eight endpoints each have more.
#[test]
fn a_manifest_holds_at_most_64_urls() {
    let trees = |count: usize| {
        let mut text = sample();
        for index in 0..count {
            let endpoints: Vec<String> = (0..8)
                .map(|part| format!("\"https://geo.example.org/trees/{index}/{part}\""))
                .collect();
            text.push_str(&format!(
                "\n[[extra.trees]]\nid = \"city{index}\"\nname = \"City {index} trees\"\n\
                 envelope = [600000, 5600000, 610000, 5610000]\nformat = \"ods-geojson-export\"\n\
                 geo_field = \"geo_point_2d\"\nendpoints = [{}]\n\
                 fields = {{ kind = \"art\", id = \"nr\", date = \"datum\" }}\n\
                 attribution = \"City {index}, dl-de/zero-2-0\"\nmax_bytes = 1000000\n",
                endpoints.join(", ")
            ));
        }
        text
    };
    // Two URLs of the sample and 56 of seven cadastres.
    let manifest = parse(&trees(7), Path::new("sample")).unwrap();
    assert_eq!(manifest.trees.len(), 7);
    assert_eq!(manifest.trees[6].endpoints.len(), 8);
    assert_eq!(manifest.trees[6].crs, Crs::Utm32);
    let findings = parse(&trees(8), Path::new("sample")).unwrap_err();
    assert_eq!(codes(&findings), ["manifest.too-many-urls"], "{findings:?}");
}

/// The full ALKIS (NAS) of a state, as Brandenburg has it.
#[test]
fn a_nas_service_is_read() {
    let text = format!(
        "{}\n[[extra.nas]]\nname = \"Sample ALKIS (NAS)\"\nurl = \"https://geo.example.org/nas\"\n\
         max_count = 100000\nmax_bytes = 268435456\nlayers = [\"gebaeude\", \"bauwerke\"]\n",
        sample()
    );
    let manifest = parse(&text, Path::new("sample")).unwrap();
    assert_eq!(manifest.nas.len(), 1);
    assert_eq!(manifest.nas[0].max_count, 100_000);
    assert_eq!(manifest.nas[0].max_bytes, 256 * 1024 * 1024);
    assert_eq!(
        manifest.nas[0].layers.as_deref(),
        Some(&["gebaeude".to_string(), "bauwerke".to_string()][..])
    );
    // An answer may not be larger than the cap of the reader.
    let over = text.replace("268435456", "268435457");
    assert_eq!(
        codes(&parse(&over, Path::new("sample")).unwrap_err()),
        ["manifest.value"]
    );
}

/// Defects beside those of the files: one change to the sample each, and the code it gives.
#[test]
fn further_defects_have_their_codes() {
    let wcs = "\n[terrain]\nkind = \"wcs\"\nurl = \"https://geo.example.org/dgm\"\n\
               coverage = \"dgm1\"\nenvelope = [600000, 5600000, 700000, 5700000]\nmax_m = 4000\n";
    let cases: [(&str, &str, &str); 14] = [
        // A placeholder other than {year} in the credit.
        (
            "© Sample {year}",
            "© Sample {year} {name}",
            "manifest.attribution",
        ),
        (
            "© Sample {year}",
            "© Sample {year} {year}",
            "manifest.attribution",
        ),
        (
            "max_px = 4000\n",
            "max_px = 4000\nparams = { \"a&b\" = \"1\" }\n",
            "manifest.params",
        ),
        (
            "kind = \"wms\"",
            "kind = \"backend\"",
            "manifest.missing-key",
        ),
        ("layer = \"dop20\"", "layer = \"dop 20\"", "manifest.value"),
        ("megabytes = 54", "megabytes = 0", "manifest.value"),
        ("megabytes = 54", "megabytes = nan", "manifest.value"),
        ("crs = 25833", "crs = 25832", "manifest.crs"),
        (
            "\"ave:GebaeudeBauwerk\"",
            "\"ave:Gebaeude&BBOX=1\"",
            "manifest.value",
        ),
        (
            "hosts = [\"geo.example.org\"]",
            "hosts = [\"GEO.example.org\", \"10.0.0.1\"]",
            "policy.host",
        ),
        ("revision = 1", "revision = 0", "manifest.value"),
        // The date of a height model cannot come from the feature info of a WMS.
        (
            "max_px = 4000\n",
            "max_px = 4000\n\n[terrain]\nkind = \"wcs\"\nurl = \"https://geo.example.org/dgm\"\n\
             coverage = \"dgm1\"\nenvelope = [600000, 5600000, 700000, 5700000]\nmax_m = 4000\n\
             date = { kind = \"wms-featureinfo\", layer = \"info\", field = \"date\" }\n",
            "manifest.date-kind",
        ),
        // A height model comes from a WCS or a backend, never from a WMS.
        (
            "max_px = 4000\n",
            "max_px = 4000\n\n[terrain]\nkind = \"wms\"\n",
            "manifest.kind",
        ),
        (
            "max_px = 4000\n",
            "max_px = 4000\n\n[surface]\nbackend = \"X\"\n",
            "manifest.value",
        ),
    ];
    for (old, new, code) in cases {
        assert_eq!(sample().matches(old).count(), 1, "{old}");
        let text = sample().replace(old, new);
        let findings = parse(&text, Path::new("sample")).expect_err(new);
        assert_eq!(codes(&findings), [code], "{new}: {findings:?}");
    }
    // The upper-case host of `[net]` is the lower-case host of the URLs.
    let upper = sample().replace("\"geo.example.org\"", "\"GEO.Example.ORG\"");
    assert_eq!(
        parse(&upper, Path::new("sample")).unwrap().hosts,
        ["geo.example.org"]
    );
    // A height model from a WCS, and what only the terrain has.
    let terrain = parse(&format!("{}{wcs}", sample()), Path::new("sample")).unwrap();
    assert!(matches!(
        terrain.terrain,
        Some(HeightModel::Wcs {
            multipart: false,
            ..
        })
    ));
    let surface = format!(
        "{}{}source_label = \"sample-dgm1\"\n",
        sample(),
        wcs.replace("[terrain]", "[surface]")
    );
    assert_eq!(
        codes(&parse(&surface, Path::new("sample")).unwrap_err()),
        ["manifest.unknown-key"]
    );
    // A folder without a name (the root, an empty path) is not compared with the id.
    assert!(parse(&sample(), Path::new("")).is_ok());
}
