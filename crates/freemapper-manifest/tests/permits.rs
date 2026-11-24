// SPDX-License-Identifier: AGPL-3.0-or-later

//! The permit sources a manifest declares (`[[permits.wfs]]`, issue #273): what the validator
//! accepts and refuses, and what the adapter makes of them.

use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::permits::{
    DeclaredReading, DeclaredWfs, ForestFunctionKind, PermitSection, PermitSource,
    PermitSourceFactory, ProtectedObjectKind,
};
use freemapper_adapter_api::registry::Registry;
use freemapper_manifest::adapter::{AdapterFiles, ManifestAdapter, Parse};
use freemapper_manifest::{ManifestError, parse};

fn sample() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/invalid/unknown-key.toml");
    std::fs::read_to_string(path)
        .unwrap()
        .replace("colour = \"red\"\n", "")
}

/// The sample manifest with `table` appended.
fn with(table: &str) -> String {
    format!("{}\n{table}", sample())
}

const HEAD: &str = "[[permits.wfs]]\nkey = \"natur\"\n\
    label = { en = \"Geoportal\", de = \"Geoportal\" }\n\
    what = { en = \"protected objects\", de = \"geschützte Objekte\" }\n\
    credit = \"Datenlizenz Deutschland Namensnennung 2.0\"\n\
    sections = [\"protected-objects\"]\n\
    url = \"https://geo.example.org/wfs\"\n\
    output_format = \"application/geo%2Bjson\"\n";

fn codes(findings: &[ManifestError]) -> Vec<(&'static str, String)> {
    findings
        .iter()
        .map(|finding| (finding.code, finding.key.clone()))
        .collect()
}

fn refused(table: &str) -> Vec<(&'static str, String)> {
    codes(&parse(&with(table), Path::new("sample")).unwrap_err())
}

#[test]
fn every_reading_is_declared_and_read() {
    let table = format!(
        "{HEAD}layers = [\n\
         {{ key = \"zones\", type_name = \"x:zones\", reading = \"water\", name = \"NAME\", zone = \"ZONE\", citation = \"VO\", planned_attribute = \"STATUS\", planned_value = \"geplant\" }},\n\
         {{ key = \"function\", type_name = \"x:fn\", reading = \"function\", function = \"recreation\", name = \"NAME\" }},\n\
         {{ key = \"units\", type_name = \"x:units\", reading = \"unit\", administration = \"Landesforst\", prefix = \"Revier \", office = \"REVIER\" }},\n\
         {{ key = \"state\", type_name = \"x:state\", reading = \"state-forest\", owner = \"Land\", prefix = \"Amt \", office = \"AMT\" }},\n\
         {{ key = \"objects\", type_name = \"ns:obj\", reading = \"object\", object = \"biotope\", name = \"ART\", only_attribute = \"SCHUTZ\", only_values = [\"1\", \"2\"] }},\n\
         {{ key = \"monuments\", type_name = \"x:mon\", reading = \"monument\", monument = \"Baudenkmal\", kind_from = \"ART\", link_attribute = \"LISTE\", link_prefix = \"https://example.org/list\" }},\n\
         ]\n"
    );
    let manifest = parse(&with(&table), Path::new("sample")).unwrap();
    assert_eq!(manifest.permits.len(), 1);
    let source: &DeclaredWfs = &manifest.permits[0];
    assert_eq!(source.key, "natur");
    assert_eq!(source.sections, [PermitSection::ProtectedObjects]);
    assert_eq!(source.url, "https://geo.example.org/wfs");
    assert!(!source.gml && source.native_crs.is_none() && source.gaps.is_none());
    let readings: Vec<&DeclaredReading> = source.layers.iter().map(|l| &l.reading).collect();
    assert_eq!(
        readings[0],
        &DeclaredReading::Water {
            name: "NAME".into(),
            zone: "ZONE".into(),
            citation: Some("VO".into()),
            planned: Some(("STATUS".into(), "geplant".into())),
        }
    );
    assert_eq!(
        readings[1],
        &DeclaredReading::Function {
            kind: ForestFunctionKind::Recreation,
            name: Some("NAME".into())
        }
    );
    assert!(matches!(readings[2], DeclaredReading::Unit { office, .. } if office == "REVIER"));
    assert!(matches!(readings[3], DeclaredReading::StateForest { owner, .. } if owner == "Land"));
    assert_eq!(
        readings[4],
        &DeclaredReading::Object {
            kind: ProtectedObjectKind::Biotope,
            name: Some("ART".into()),
            citation: None,
            only: Some(("SCHUTZ".into(), vec!["1".into(), "2".into()])),
        }
    );
    assert!(matches!(
        readings[5],
        DeclaredReading::Monument { link: Some((a, p)), .. } if a == "LISTE" && p == "https://example.org/list"
    ));
}

#[test]
fn a_native_crs_and_gaps_and_gml_are_read() {
    let table = format!(
        "{HEAD}gml = true\nnative_crs = 25832\ngaps = {{ en = \"later data are missing\", de = \"spätere fehlen\" }}\n\
         layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"object\", object = \"natural-monument\" }}]\n"
    );
    let manifest = parse(&with(&table), Path::new("sample")).unwrap();
    let source = &manifest.permits[0];
    assert!(source.gml);
    assert_eq!(source.native_crs, Some(25832));
    assert_eq!(source.gaps.as_ref().unwrap().de, "spätere fehlen");
}

#[test]
fn what_is_wrong_is_named_with_its_key() {
    let layer =
        |extra: &str| format!("{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", {extra} }}]\n");
    // A reading nobody knows.
    assert_eq!(
        refused(&layer("reading = \"swamp\"")),
        [("manifest.value", "permits.wfs.layers.reading".to_string())]
    );
    // A key that does not belong to the reading, and one it needs.
    assert_eq!(
        refused(&layer(
            "reading = \"object\", object = \"biotope\", zone = \"Z\""
        )),
        [("manifest.unknown-key", "zone".to_string())]
    );
    assert_eq!(
        refused(&layer("reading = \"object\"")),
        [("manifest.missing-key", "object".to_string())]
    );
    // A kind the program does not know.
    assert_eq!(
        refused(&layer("reading = \"object\", object = \"dragon\"")),
        [("manifest.value", "permits.wfs.layers.object".to_string())]
    );
    // A pair that comes together.
    assert_eq!(
        refused(&layer(
            "reading = \"object\", object = \"biotope\", only_attribute = \"A\""
        )),
        [("manifest.value", "only_attribute".to_string())]
    );
}

#[test]
fn the_service_is_held_to_the_hosts_of_the_manifest() {
    let table = HEAD.replace("geo.example.org", "other.example.org")
        + "layers = [{ key = \"a\", type_name = \"x:a\", reading = \"object\", object = \"biotope\" }]\n";
    assert_eq!(
        refused(&table),
        [("policy.host-not-declared", "permits.wfs.url".to_string())]
    );
    // A fixed query that selects the service on the server is kept; a fragment or anything but
    // plain `name=value` pairs is not.
    let layers = "layers = [{ key = \"a\", type_name = \"x:a\", reading = \"object\", object = \"biotope\" }]\n";
    let table = HEAD.replace("/wfs\"", "/wfs?map=a\"") + layers;
    let kept = parse(&with(&table), Path::new("sample")).unwrap();
    assert!(kept.permits[0].url.ends_with("/wfs?map=a"));
    for bad in [
        "/wfs?map=a#b",
        "/wfs?",
        "/wfs?map",
        "/wfs?map=a%20b",
        "/wfs?a=b&&c=d",
    ] {
        let table = HEAD.replace("/wfs\"", &format!("{bad}\"")) + layers;
        assert_eq!(
            refused(&table),
            [("policy.url", "permits.wfs.url".to_string())],
            "{bad}"
        );
    }
}

#[test]
fn keys_and_sections_are_checked() {
    let layers = "layers = [{ key = \"a\", type_name = \"x:a\", reading = \"object\", object = \"biotope\" }]\n";
    // Two sources with one key.
    let twice = format!("{HEAD}{layers}\n{HEAD}{layers}");
    assert_eq!(
        refused(&twice),
        [("manifest.value", "permits.wfs.key".to_string())]
    );
    // A section that does not exist, and a source without layers.
    let section = HEAD.replace("protected-objects", "treasure") + layers;
    assert_eq!(
        refused(&section),
        [("manifest.value", "permits.wfs.sections".to_string())]
    );
    assert_eq!(
        refused(&format!("{HEAD}layers = []\n")),
        [("manifest.value", "permits.wfs.layers".to_string())]
    );
    // Two layers with one key.
    let layer = "{ key = \"a\", type_name = \"x:a\", reading = \"object\", object = \"biotope\" }";
    assert_eq!(
        refused(&format!("{HEAD}layers = [{layer}, {layer}]\n")),
        [("manifest.value", "permits.wfs.layers.key".to_string())]
    );
}

/// What the registry's factory makes.
struct Made;

impl PermitSourceFactory for Made {
    fn wfs(&self, declared: &DeclaredWfs) -> Box<dyn PermitSource> {
        struct One(freemapper_adapter_api::permits::PermitSourceInfo);
        impl PermitSource for One {
            fn info(&self) -> &freemapper_adapter_api::permits::PermitSourceInfo {
                &self.0
            }
            fn fetch(
                &self,
                _: &dyn freemapper_adapter_api::fetch::Fetcher,
                _: &geo_types::Rect<f64>,
                _: &Path,
            ) -> Result<Vec<Vec<serde_json::Value>>, freemapper_adapter_api::error::FetchError>
            {
                Ok(Vec::new())
            }
            fn findings(
                &self,
                _: &str,
                _: &[serde_json::Value],
            ) -> Vec<freemapper_adapter_api::permits::PermitFinding> {
                Vec::new()
            }
        }
        let key: &'static str = Box::leak(declared.key.clone().into_boxed_str());
        Box::new(One(freemapper_adapter_api::permits::PermitSourceInfo {
            key,
            label: freemapper_adapter_api::permits::Wording { en: "", de: "" },
            what: freemapper_adapter_api::permits::Wording { en: "", de: "" },
            credit: "",
            note: None,
            sampled: false,
            gaps: None,
            layers: &[],
            sections: &[],
            offices: None,
        }))
    }
}

struct Square;

impl AdapterFiles for Square {
    fn read(&self, _: &str) -> Result<Cow<'static, str>, String> {
        Ok(Cow::Borrowed(
            r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
            [[[400000,5800000],[401000,5800000],[401000,5801000],[400000,5801000],[400000,5800000]]]}}"#,
        ))
    }
}

#[test]
fn the_adapter_makes_its_sources_with_the_factory_of_the_registry() {
    let table = format!(
        "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"object\", object = \"biotope\" }}]\n"
    );
    let manifest = || parse(&with(&table), Path::new("sample")).unwrap();
    // A program without a factory cannot read what the manifest declares.
    let findings = ManifestAdapter::new(manifest(), &Square, Parse::Now, &Registry::new())
        .err()
        .expect("no factory, no adapter");
    assert_eq!(
        codes(&findings),
        [("manifest.backend-missing", "permits.wfs".to_string())]
    );
    let mut registry = Registry::new();
    registry.set_permit_factory(Arc::new(Made));
    let adapter = ManifestAdapter::new(manifest(), &Square, Parse::Now, &registry).unwrap();
    let keys: Vec<&str> = adapter
        .permit_sources()
        .iter()
        .map(|s| s.info().key)
        .collect();
    assert_eq!(keys, ["natur"]);
    // A manifest that declares none needs no factory.
    let plain = ManifestAdapter::new(
        parse(&sample(), Path::new("sample")).unwrap(),
        &Square,
        Parse::Now,
        &Registry::new(),
    )
    .unwrap();
    assert!(plain.permit_sources().is_empty());
}

#[test]
fn a_feature_type_may_hold_letters_beyond_ascii() {
    let layers = |type_name: &str| {
        format!(
            "layers = [{{ key = \"a\", type_name = \"{type_name}\", reading = \"object\", object = \"biotope\" }}]\n"
        )
    };
    let table = format!("{HEAD}{}", layers("ps:Flächennaturdenkmale"));
    assert!(parse(&with(&table), Path::new("sample")).is_ok());
    for bad in ["ps:Fl%C3%A4chen", "ps:a b", "ps:a:b", "1x", "ps:"] {
        let table = format!("{HEAD}{}", layers(bad));
        assert_eq!(
            refused(&table),
            [("manifest.value", "permits.wfs.layers.type_name".to_string())],
            "{bad}"
        );
    }
}

fn messages(table: &str) -> Vec<String> {
    parse(&with(table), Path::new("sample"))
        .unwrap_err()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

#[test]
fn optional_pairs_may_be_left_out_and_come_together() {
    let layer =
        |extra: &str| format!("{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", {extra} }}]\n");
    // Without the optional pair a reading is still read.
    let water = parse(
        &with(&layer("reading = \"water\", name = \"N\", zone = \"Z\"")),
        Path::new("sample"),
    )
    .unwrap();
    assert!(matches!(
        &water.permits[0].layers[0].reading,
        DeclaredReading::Water { planned: None, .. }
    ));
    let monument = parse(
        &with(&layer("reading = \"monument\", monument = \"Baudenkmal\"")),
        Path::new("sample"),
    )
    .unwrap();
    assert!(matches!(
        &monument.permits[0].layers[0].reading,
        DeclaredReading::Monument { link: None, .. }
    ));
    // Half of a pair is refused.
    assert_eq!(
        refused(&layer(
            "reading = \"monument\", monument = \"B\", link_prefix = \"https://example.org/\""
        )),
        [("manifest.missing-key", "link_attribute".to_string())]
    );
}

#[test]
fn only_values_come_one_to_eight_with_their_attribute() {
    let layer = |values: usize| {
        let list: Vec<String> = (0..values).map(|n| format!("\"{n}\"")).collect();
        format!(
            "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"object\", object = \"biotope\", only_attribute = \"A\", only_values = [{}] }}]\n",
            list.join(", ")
        )
    };
    for fine in [1, 8] {
        assert!(
            parse(&with(&layer(fine)), Path::new("sample")).is_ok(),
            "{fine}"
        );
    }
    for bad in [0, 9] {
        assert_eq!(
            refused(&layer(bad)),
            [("manifest.value", "only_attribute".to_string())],
            "{bad}"
        );
    }
}

#[test]
fn a_half_given_pair_counts_as_given_for_a_reading_without_it() {
    let layer =
        |extra: &str| format!("{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", {extra} }}]\n");
    // `only_values` alone, and `link_prefix` alone, are keys the reading does not have.
    assert!(
        refused(&layer(
            "reading = \"water\", name = \"N\", zone = \"Z\", only_values = [\"1\"]"
        ))
        .contains(&("manifest.unknown-key", "only_attribute".to_string()))
    );
    assert!(
        refused(&layer(
            "reading = \"object\", object = \"biotope\", link_prefix = \"https://example.org/\""
        ))
        .contains(&("manifest.unknown-key", "link_attribute".to_string()))
    );
}

#[test]
fn a_name_in_a_finding_is_shown_without_control_characters() {
    let layer =
        |extra: &str| format!("{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", {extra} }}]\n");
    let kind = messages(&layer("reading = \"object\", object = \"dra\\ngon\""));
    assert_eq!(kind.len(), 1);
    assert!(kind[0].contains("`dragon`"), "{}", kind[0]);
    let reading = messages(&layer("reading = \"sw\\namp\""));
    assert_eq!(reading.len(), 1);
    assert!(reading[0].contains("`swamp`"), "{}", reading[0]);
}

#[test]
fn at_most_twelve_sources_are_declared() {
    let sources = |count: usize| {
        (0..count)
            .map(|n| {
                format!(
                    "{}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"object\", object = \"biotope\" }}]\n",
                    HEAD.replace("\"natur\"", &format!("\"s{n}\""))
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        parse(&with(&sources(12)), Path::new("sample"))
            .unwrap()
            .permits
            .len(),
        12
    );
    assert_eq!(
        refused(&sources(13)),
        [("manifest.value", "permits.wfs".to_string())]
    );
}

#[test]
fn the_permit_sources_are_handed_over_once() {
    let table = format!(
        "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"object\", object = \"biotope\" }}]\n"
    );
    let mut registry = Registry::new();
    registry.set_permit_factory(Arc::new(Made));
    let mut adapter = ManifestAdapter::new(
        parse(&with(&table), Path::new("sample")).unwrap(),
        &Square,
        Parse::Now,
        &registry,
    )
    .unwrap();
    let taken: Vec<&str> = adapter
        .take_permit_sources()
        .iter()
        .map(|s| s.info().key)
        .collect();
    assert_eq!(taken, ["natur"]);
    assert!(adapter.permit_sources().is_empty());
}

#[test]
fn districts_green_spaces_and_a_unit_named_by_its_feature_are_declared() {
    let table = format!(
        "{HEAD}offices = \"hamburg\"\nlayers = [\n\
         {{ key = \"d\", type_name = \"x:d\", reading = \"district\", name = \"BEZIRK\" }},\n\
         {{ key = \"g\", type_name = \"x:g\", reading = \"green-space\", name = \"NAME\", fallback = \"ART\", district_attribute = \"NR\", districts = [\"Mitte\", \"Nord\"] }},\n\
         {{ key = \"h\", type_name = \"x:h\", reading = \"green-space\", name = \"NAME\" }},\n\
         {{ key = \"u\", type_name = \"x:u\", reading = \"unit\", administration = \"Bezirksamt \", administration_attribute = \"BEZIRK\", prefix = \"Revier \", office = \"REVIER\" }},\n\
         ]\n"
    );
    let manifest = parse(&with(&table), Path::new("sample")).unwrap();
    let source = &manifest.permits[0];
    assert_eq!(
        source.offices,
        Some(freemapper_adapter_api::permits::DistrictOffices::Hamburg)
    );
    let readings: Vec<&DeclaredReading> = source.layers.iter().map(|l| &l.reading).collect();
    assert_eq!(
        readings[0],
        &DeclaredReading::District {
            name: "BEZIRK".into()
        }
    );
    assert_eq!(
        readings[1],
        &DeclaredReading::GreenSpace {
            name: "NAME".into(),
            fallback: Some("ART".into()),
            district: Some(("NR".into(), vec!["Mitte".into(), "Nord".into()])),
        }
    );
    assert_eq!(
        readings[2],
        &DeclaredReading::GreenSpace {
            name: "NAME".into(),
            fallback: None,
            district: None,
        }
    );
    assert_eq!(
        readings[3],
        &DeclaredReading::Unit {
            administration: "Bezirksamt ".into(),
            administration_from: Some("BEZIRK".into()),
            prefix: "Revier ".into(),
            office: "REVIER".into(),
        }
    );
    // A source without `offices` names none.
    let plain = parse(
        &with(&format!(
            "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n"
        )),
        Path::new("sample"),
    )
    .unwrap();
    assert_eq!(plain.permits[0].offices, None);
}

#[test]
fn the_keys_of_the_new_readings_are_checked() {
    let layer =
        |extra: &str| format!("{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", {extra} }}]\n");
    // A district needs its name and takes no other key.
    assert_eq!(
        refused(&layer("reading = \"district\"")),
        [("manifest.missing-key", "name".to_string())]
    );
    assert_eq!(
        refused(&layer(
            "reading = \"district\", name = \"N\", fallback = \"F\""
        )),
        [("manifest.unknown-key", "fallback".to_string())]
    );
    // A green space needs its name; the district list comes with its attribute, 1 to 16 names.
    assert_eq!(
        refused(&layer("reading = \"green-space\"")),
        [("manifest.missing-key", "name".to_string())]
    );
    assert_eq!(
        refused(&layer(
            "reading = \"green-space\", name = \"N\", districts = [\"A\"]"
        )),
        [("manifest.value", "district_attribute".to_string())]
    );
    assert_eq!(
        refused(&layer(
            "reading = \"green-space\", name = \"N\", district_attribute = \"NR\", districts = []"
        )),
        [("manifest.value", "district_attribute".to_string())]
    );
    let names: Vec<String> = (0..17).map(|n| format!("\"{n}\"")).collect();
    assert_eq!(
        refused(&layer(&format!(
            "reading = \"green-space\", name = \"N\", district_attribute = \"NR\", districts = [{}]",
            names.join(", ")
        ))),
        [("manifest.value", "district_attribute".to_string())]
    );
    let sixteen: Vec<String> = (0..16).map(|n| format!("\"{n}\"")).collect();
    assert!(
        parse(
            &with(&layer(&format!(
                "reading = \"green-space\", name = \"N\", district_attribute = \"NR\", districts = [{}]",
                sixteen.join(", ")
            ))),
            Path::new("sample")
        )
        .is_ok()
    );
    // The attribute of an administration belongs to the unit only.
    assert_eq!(
        refused(&layer(
            "reading = \"state-forest\", owner = \"O\", prefix = \"P\", office = \"A\", administration_attribute = \"B\""
        )),
        [(
            "manifest.unknown-key",
            "administration_attribute".to_string()
        )]
    );
    // An office of a city state the program does not know.
    assert_eq!(
        refused(&format!(
            "{HEAD}offices = \"bremen\"\nlayers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n"
        )),
        [("manifest.value", "permits.wfs.offices".to_string())]
    );
}
