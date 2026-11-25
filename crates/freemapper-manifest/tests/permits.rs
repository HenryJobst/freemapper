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
            planned_label: None,
            authority: None,
            register_names: false,
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
            point_names: Vec::new(),
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
            dedication: None,
            park_names: false,
        }
    );
    assert_eq!(
        readings[2],
        &DeclaredReading::GreenSpace {
            name: "NAME".into(),
            fallback: None,
            district: None,
            dedication: None,
            park_names: false,
        }
    );
    assert_eq!(
        readings[3],
        &DeclaredReading::Unit {
            administration: "Bezirksamt ".into(),
            administration_from: Some("BEZIRK".into()),
            prefix: "Revier ".into(),
            office: "REVIER".into(),
            unit: None,
            rules: Default::default(),
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
    // Either half of the district pair is a key a district does not have.
    for half in ["districts = [\"A\"]", "district_attribute = \"NR\""] {
        assert_eq!(
            refused(&layer(&format!(
                "reading = \"district\", name = \"N\", {half}"
            ))),
            [("manifest.unknown-key", "district_attribute".to_string())],
            "{half}"
        );
    }
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

#[test]
fn a_unit_within_an_office_a_water_authority_and_a_note_are_declared() {
    let table = format!(
        "{HEAD}note = \"What the data are worth\"\nlayers = [\n\
         {{ key = \"u\", type_name = \"x:u\", reading = \"unit\", administration = \"Forst\", prefix = \"Amt \", office = \"AMT\", unit_attribute = \"REVIER\", unit_prefix = \"Revier \" }},\n\
         {{ key = \"w\", type_name = \"x:w\", reading = \"water\", name = \"N\", zone = \"Z\", planned_attribute = \"S\", planned_value = \"vorläufig\", planned_label = \"vorläufig angeordnet\", authority = \"DST\", name_form = \"register\" }},\n\
         {{ key = \"v\", type_name = \"x:v\", reading = \"water\", name = \"N\", zone = \"Z\" }},\n\
         ]\n"
    );
    let manifest = parse(&with(&table), Path::new("sample")).unwrap();
    let source = &manifest.permits[0];
    assert_eq!(source.note.as_deref(), Some("What the data are worth"));
    assert_eq!(
        source.layers[0].reading,
        DeclaredReading::Unit {
            administration: "Forst".into(),
            administration_from: None,
            prefix: "Amt ".into(),
            office: "AMT".into(),
            unit: Some(("REVIER".into(), "Revier ".into())),
            rules: Default::default(),
        }
    );
    assert_eq!(
        source.layers[1].reading,
        DeclaredReading::Water {
            name: "N".into(),
            zone: "Z".into(),
            citation: None,
            planned: Some(("S".into(), "vorläufig".into())),
            planned_label: Some("vorläufig angeordnet".into()),
            authority: Some("DST".into()),
            register_names: true,
        }
    );
    assert_eq!(
        source.layers[2].reading,
        DeclaredReading::Water {
            name: "N".into(),
            zone: "Z".into(),
            citation: None,
            planned: None,
            planned_label: None,
            authority: None,
            register_names: false,
        }
    );
    // Without a note the source has none.
    let plain = parse(
        &with(&format!(
            "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n"
        )),
        Path::new("sample"),
    )
    .unwrap();
    assert_eq!(plain.permits[0].note, None);
}

#[test]
fn the_keys_of_the_unit_and_water_extras_are_checked() {
    let layer =
        |extra: &str| format!("{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", {extra} }}]\n");
    let unit = "reading = \"unit\", administration = \"F\", prefix = \"P\", office = \"O\"";
    // Either half of the unit pair is refused alone.
    for half in ["unit_attribute = \"R\"", "unit_prefix = \"Revier \""] {
        assert_eq!(
            refused(&layer(&format!("{unit}, {half}"))),
            [("manifest.missing-key", "unit_attribute".to_string())],
            "{half}"
        );
    }
    // The pair of a unit is a pair for a reading without it too, whichever half is given.
    for half in ["unit_attribute = \"R\"", "unit_prefix = \"R \""] {
        assert!(
            refused(&layer(&format!(
                "reading = \"state-forest\", owner = \"O\", prefix = \"P\", office = \"A\", {half}"
            )))
            .contains(&("manifest.unknown-key", "unit_attribute".to_string())),
            "{half}"
        );
    }
    // A state forest has no unit within it, nor a water zone.
    assert_eq!(
        refused(&layer(
            "reading = \"state-forest\", owner = \"O\", prefix = \"P\", office = \"A\", unit_attribute = \"R\", unit_prefix = \"R \""
        )),
        [("manifest.unknown-key", "unit_attribute".to_string())]
    );
    for key in [
        "authority = \"A\"",
        "planned_label = \"x\"",
        "name_form = \"register\"",
    ] {
        assert_eq!(
            refused(&layer(&format!(
                "reading = \"function\", function = \"natural\", {key}"
            ))),
            [(
                "manifest.unknown-key",
                key.split(' ').next().unwrap().to_string()
            )],
            "{key}"
        );
    }
    // A label for the planned zones needs the pair that marks them; the only name form is `register`.
    assert_eq!(
        refused(&layer(
            "reading = \"water\", name = \"N\", zone = \"Z\", planned_label = \"x\""
        )),
        [("manifest.missing-key", "planned_attribute".to_string())]
    );
    assert_eq!(
        refused(&layer(
            "reading = \"water\", name = \"N\", zone = \"Z\", name_form = \"capitals\""
        )),
        [("manifest.value", "permits.wfs.layers.name_form".to_string())]
    );
    // The note is text of at most 600 characters.
    let long = "x".repeat(601);
    assert_eq!(
        refused(&format!(
            "{HEAD}note = \"{long}\"\nlayers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n"
        )),
        [("manifest.value", "permits.wfs.note".to_string())]
    );
}

#[test]
fn playgrounds_owners_and_the_named_parts_of_a_green_space_are_declared() {
    let table = format!(
        "{HEAD}layers = [\n\
         {{ key = \"g\", type_name = \"x:g\", reading = \"green-space\", name = \"NAME\", name_form = \"park\", district_attribute = \"BEZIRK\", dedication = \"WIDMUNG\" }},\n\
         {{ key = \"p\", type_name = \"x:p\", reading = \"playground\" }},\n\
         {{ key = \"o\", type_name = \"x:o\", reading = \"owner\", owner_attribute = \"SCH\", owners = [\"Land\", \"Bund\"] }},\n\
         {{ key = \"n\", type_name = \"x:n\", reading = \"object\", object = \"natural-monument\", name = \"GEBIET\", point_names = [\"BAUM\", \"STEIN\"] }},\n\
         ]\n"
    );
    let manifest = parse(&with(&table), Path::new("sample")).unwrap();
    let layers = &manifest.permits[0].layers;
    assert_eq!(
        layers[0].reading,
        DeclaredReading::GreenSpace {
            name: "NAME".into(),
            fallback: None,
            // Without a list the attribute holds the name of the district.
            district: Some(("BEZIRK".into(), Vec::new())),
            dedication: Some("WIDMUNG".into()),
            park_names: true,
        }
    );
    assert_eq!(layers[1].reading, DeclaredReading::Playground);
    assert_eq!(
        layers[2].reading,
        DeclaredReading::Owner {
            owner: "SCH".into(),
            named: vec!["Land".into(), "Bund".into()]
        }
    );
    assert!(matches!(
        &layers[3].reading,
        DeclaredReading::Object { point_names, .. } if point_names == &["BAUM", "STEIN"]
    ));
}

#[test]
fn the_keys_of_the_berlin_readings_are_checked() {
    let layer =
        |extra: &str| format!("{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", {extra} }}]\n");
    // A playground takes no key.
    assert_eq!(
        refused(&layer("reading = \"playground\", name = \"N\"")),
        [("manifest.unknown-key", "name".to_string())]
    );
    // An owner needs its attribute and 1 to 8 owners; both halves belong to it only.
    assert_eq!(
        refused(&layer("reading = \"owner\", owners = [\"A\"]")),
        [("manifest.missing-key", "owner_attribute".to_string())]
    );
    assert_eq!(
        refused(&layer("reading = \"owner\", owner_attribute = \"S\"")),
        [("manifest.value", "owners".to_string())]
    );
    assert_eq!(
        refused(&layer(
            "reading = \"owner\", owner_attribute = \"S\", owners = []"
        )),
        [("manifest.value", "owners".to_string())]
    );
    let nine: Vec<String> = (0..9).map(|n| format!("\"{n}\"")).collect();
    assert_eq!(
        refused(&layer(&format!(
            "reading = \"owner\", owner_attribute = \"S\", owners = [{}]",
            nine.join(", ")
        ))),
        [("manifest.value", "owners".to_string())]
    );
    for half in ["owner_attribute = \"S\"", "owners = [\"A\"]"] {
        assert!(
            refused(&layer(&format!(
                "reading = \"district\", name = \"N\", {half}"
            )))
            .contains(&("manifest.unknown-key", "owner_attribute".to_string())),
            "{half}"
        );
    }
    // Point names are for an object, 1 to 4 of them.
    assert_eq!(
        refused(&layer(
            "reading = \"object\", object = \"biotope\", point_names = []"
        )),
        [("manifest.value", "point_names".to_string())]
    );
    let five: Vec<String> = (0..5).map(|n| format!("\"A{n}\"")).collect();
    assert_eq!(
        refused(&layer(&format!(
            "reading = \"object\", object = \"biotope\", point_names = [{}]",
            five.join(", ")
        ))),
        [("manifest.value", "point_names".to_string())]
    );
    assert!(
        refused(&layer(
            "reading = \"water\", name = \"N\", zone = \"Z\", point_names = [\"A\"]"
        ))
        .contains(&("manifest.unknown-key", "point_names".to_string()))
    );
    // A dedication is for a green space; the name form of a green space is `park`.
    assert!(
        refused(&layer(
            "reading = \"district\", name = \"N\", dedication = \"W\""
        ))
        .contains(&("manifest.unknown-key", "dedication".to_string()))
    );
    assert_eq!(
        refused(&layer(
            "reading = \"green-space\", name = \"N\", name_form = \"register\""
        )),
        [("manifest.value", "permits.wfs.layers.name_form".to_string())]
    );
    // A list of districts needs its attribute.
    assert_eq!(
        refused(&layer(
            "reading = \"green-space\", name = \"N\", districts = [\"A\"]"
        )),
        [("manifest.value", "district_attribute".to_string())]
    );
    // A note may be up to 600 characters.
    let long = "x".repeat(600);
    assert!(
        parse(
            &with(&format!(
                "{HEAD}note = \"{long}\"\nlayers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n"
            )),
            Path::new("sample")
        )
        .is_ok()
    );
}

const SHAPE_HEAD: &str = "[[permits.shapefile]]\nkey = \"wsg\"\n\
    label = { en = \"Land\", de = \"Land\" }\n\
    what = { en = \"water protection areas\", de = \"Wasserschutzgebiete\" }\n\
    credit = \"Datenlizenz Deutschland Zero\"\n\
    sections = [\"water-protection\"]\n\
    url = \"https://geo.example.org/files/wsg.zip\"\n\
    crs = 25833\n";

const SHAPE_LAYER: &str =
    "layers = [{ key = \"w\", reading = \"water\", name = \"NAME\", zone = \"ZONE\" }]\n";

#[test]
fn a_shapefile_source_is_declared_with_its_zip_and_crs() {
    let table = format!("{SHAPE_HEAD}note = \"A note\"\n{SHAPE_LAYER}");
    let manifest = parse(&with(&table), Path::new("sample")).unwrap();
    let source = &manifest.permits[0];
    assert!(source.shapefile);
    assert_eq!(source.url, "https://geo.example.org/files/wsg.zip");
    assert_eq!(source.native_crs, Some(25833));
    assert_eq!(source.output_format, "");
    assert!(!source.gml && source.offices.is_none());
    assert_eq!(source.note.as_deref(), Some("A note"));
    assert_eq!(source.layers.len(), 1);
    assert_eq!(source.layers[0].key, "w");
    assert_eq!(source.layers[0].type_name, "");
    assert!(matches!(
        source.layers[0].reading,
        DeclaredReading::Water { .. }
    ));
    // A service is no shapefile.
    let wfs = parse(
        &with(&format!(
            "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n"
        )),
        Path::new("sample"),
    )
    .unwrap();
    assert!(!wfs.permits[0].shapefile);
    // The services come first in the order of the manifest, then the shapefiles.
    let both = format!(
        "{SHAPE_HEAD}{SHAPE_LAYER}\n{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n"
    );
    let keys: Vec<String> = parse(&with(&both), Path::new("sample"))
        .unwrap()
        .permits
        .into_iter()
        .map(|source| source.key)
        .collect();
    assert_eq!(keys, ["natur", "wsg"]);
}

#[test]
fn what_is_wrong_with_a_shapefile_source_is_named() {
    // The keys of a service are no keys of a shapefile source.
    for (extra, key) in [
        ("output_format = \"GEOJSON\"\n", "output_format"),
        ("gml = true\n", "gml"),
        ("native_crs = 25832\n", "native_crs"),
        ("offices = \"berlin\"\n", "offices"),
    ] {
        assert_eq!(
            refused(&format!("{SHAPE_HEAD}{extra}{SHAPE_LAYER}")),
            [("manifest.unknown-key", key.to_string())],
            "{key}"
        );
    }
    // It needs its CRS, which must be one of a map.
    assert_eq!(
        refused(&format!(
            "{}{SHAPE_LAYER}",
            SHAPE_HEAD.replace("crs = 25833\n", "")
        )),
        [("manifest.missing-key", "crs".to_string())]
    );
    assert_eq!(
        refused(&format!(
            "{}{SHAPE_LAYER}",
            SHAPE_HEAD.replace("25833", "4326")
        )),
        [("manifest.crs", "permits.shapefile.crs".to_string())]
    );
    // Exactly one layer, without a feature type.
    let layer = "{ key = \"w\", reading = \"water\", name = \"N\", zone = \"Z\" }";
    for layers in [
        "layers = []\n".to_string(),
        format!("layers = [{layer}, {}]\n", layer.replace("\"w\"", "\"v\"")),
    ] {
        assert_eq!(
            refused(&format!("{SHAPE_HEAD}{layers}")),
            [("manifest.value", "permits.shapefile.layers".to_string())],
            "{layers}"
        );
    }
    assert_eq!(
        refused(&format!(
            "{SHAPE_HEAD}layers = [{{ key = \"w\", type_name = \"x:w\", reading = \"water\", name = \"N\", zone = \"Z\" }}]\n"
        )),
        [("manifest.unknown-key", "type_name".to_string())]
    );
    // A zip is asked as it is: no query.
    assert_eq!(
        refused(&format!(
            "{}{SHAPE_LAYER}",
            SHAPE_HEAD.replace("wsg.zip", "wsg.zip?a=b")
        )),
        [("policy.url", "permits.shapefile.url".to_string())]
    );
    // The host must be one of the manifest.
    assert_eq!(
        refused(&format!(
            "{}{SHAPE_LAYER}",
            SHAPE_HEAD.replace("geo.example.org", "other.example.org")
        )),
        [(
            "policy.host-not-declared",
            "permits.shapefile.url".to_string()
        )]
    );
    // A service needs its feature types and its format, and has no `crs`.
    assert_eq!(
        refused(&format!(
            "{HEAD}layers = [{{ key = \"a\", reading = \"district\", name = \"N\" }}]\n"
        )),
        [("manifest.missing-key", "type_name".to_string())]
    );
    assert_eq!(
        refused(&format!(
            "{}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n",
            HEAD.replace(
                "output_format = \"application/geo%2Bjson\"\n",
                "crs = 25832\n"
            )
        )),
        [
            ("manifest.unknown-key", "crs".to_string()),
            ("manifest.missing-key", "output_format".to_string())
        ]
    );
    // The two tables share the keys of their sources and the limit of twelve.
    assert_eq!(
        refused(&format!(
            "{SHAPE_HEAD}{SHAPE_LAYER}\n{}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n",
            HEAD.replace("\"natur\"", "\"wsg\"")
        )),
        [("manifest.value", "permits.shapefile.key".to_string())]
    );
    let sources = |wfs: usize, shapes: usize| {
        let mut all: Vec<String> = (0..wfs)
            .map(|n| {
                format!(
                    "{}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n",
                    HEAD.replace("\"natur\"", &format!("\"s{n}\""))
                )
            })
            .collect();
        all.extend((0..shapes).map(|n| {
            format!(
                "{}{SHAPE_LAYER}",
                SHAPE_HEAD.replace("\"wsg\"", &format!("\"z{n}\""))
            )
        }));
        all.join("\n")
    };
    assert!(parse(&with(&sources(8, 4)), Path::new("sample")).is_ok());
    assert_eq!(
        refused(&sources(8, 5)),
        [("manifest.value", "permits.wfs".to_string())]
    );
}

#[test]
fn the_names_of_forest_offices_follow_the_rules_a_layer_declares() {
    let layer = |extra: &str| {
        format!(
            "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"unit\", administration = \"F\", prefix = \"P \", office = \"O\"{extra} }}]\n"
        )
    };
    let manifest = parse(
        &with(&layer(
            ", office_unless = \"amt\", office_exceptions = [{ name = \"Wald\", prefix = \"Lehr \" }, { name = \"Forst\", prefix = \"Alt \" }]",
        )),
        Path::new("sample"),
    )
    .unwrap();
    let DeclaredReading::Unit { rules, .. } = &manifest.permits[0].layers[0].reading else {
        panic!()
    };
    assert_eq!(rules.unless.as_deref(), Some("amt"));
    assert_eq!(
        rules.exceptions,
        [
            ("Wald".to_string(), "Lehr ".to_string()),
            ("Forst".to_string(), "Alt ".to_string())
        ]
    );
    let plain = parse(&with(&layer("")), Path::new("sample")).unwrap();
    let DeclaredReading::Unit { rules, .. } = &plain.permits[0].layers[0].reading else {
        panic!()
    };
    assert_eq!(rules.unless, None);
    assert!(rules.exceptions.is_empty());
    // The state forest takes them too.
    let forest = parse(
        &with(&format!(
            "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"state-forest\", owner = \"O\", prefix = \"P \", office = \"A\", office_unless = \"amt\" }}]\n"
        )),
        Path::new("sample"),
    )
    .unwrap();
    assert!(matches!(
        &forest.permits[0].layers[0].reading,
        DeclaredReading::StateForest { rules, .. } if rules.unless.as_deref() == Some("amt")
    ));
    // Between one and four exceptions.
    assert_eq!(
        refused(&layer(", office_exceptions = []")),
        [("manifest.value", "office_exceptions".to_string())]
    );
    let five: Vec<String> = (0..5)
        .map(|n| format!("{{ name = \"N{n}\", prefix = \"P\" }}"))
        .collect();
    assert_eq!(
        refused(&layer(&format!(
            ", office_exceptions = [{}]",
            five.join(", ")
        ))),
        [("manifest.value", "office_exceptions".to_string())]
    );
    // Other readings have no office.
    for key in [
        "office_unless = \"x\"",
        "office_exceptions = [{ name = \"N\", prefix = \"P\" }]",
    ] {
        assert!(
            refused(&format!(
                "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\", {key} }}]\n"
            ))
            .contains(&("manifest.unknown-key", key.split(' ').next().unwrap().to_string())),
            "{key}"
        );
    }
}

const POINT_HEAD: &str = "[[permits.getfeatureinfo]]\nkey = \"forst\"\n\
    label = { en = \"Forest\", de = \"Forst\" }\n\
    what = { en = \"forest units\", de = \"Forstbezirke\" }\n\
    credit = \"Datenlizenz Deutschland Namensnennung 2.0\"\n\
    sections = [\"forest-units\"]\n";

const POINT_REVIER: &str = "{ key = \"reviere\", url = \"https://geo.example.org/wms\", wms_layers = \"Reviere\", one = true, reading = \"unit\", administration = \"Forst\", prefix = \"Bezirk \", office = \"B\", office_renames = [{ starts_with = \"Nationalpark\", name = \"Nationalparkverwaltung\" }] }";

const POINT_CODES: &str = "{ key = \"funktionen\", url = \"https://geo.example.org/wms2\", wms_layers = \"7,46,101\", reading = \"function-code\", code_attribute = \"WFZ\", name = \"GNM\", codes = [{ code = \"7110\", function = \"recreation\" }, { code = \"1110\", function = \"protected\", label = \"Boden\" }], water_prefix = \"211\", water_length = 5, zone = \"KNM\", zone_after = \" Z \" }";

#[test]
fn a_source_asked_at_points_declares_its_services_in_its_layers() {
    let table = format!("{POINT_HEAD}layers = [{POINT_REVIER}, {POINT_CODES}]\n");
    let manifest = parse(&with(&table), Path::new("sample")).unwrap();
    let source = &manifest.permits[0];
    assert!(source.getfeatureinfo && !source.shapefile && !source.gml);
    assert_eq!(source.url, "");
    assert_eq!(source.output_format, "");
    assert_eq!(source.native_crs, None);
    assert!(source.offices.is_none());
    let revier = &source.layers[0];
    assert_eq!(revier.type_name, "");
    let query = revier.query.as_ref().unwrap();
    assert_eq!(query.url, "https://geo.example.org/wms");
    assert_eq!(query.layers, "Reviere");
    assert!(query.one);
    match &revier.reading {
        DeclaredReading::Unit { rules, .. } => assert_eq!(
            rules.renames,
            [(
                "Nationalpark".to_string(),
                "Nationalparkverwaltung".to_string()
            )]
        ),
        other => panic!("{other:?}"),
    }
    let funktionen = &source.layers[1];
    let query = funktionen.query.as_ref().unwrap();
    assert_eq!(query.layers, "7,46,101");
    assert!(!query.one);
    match &funktionen.reading {
        DeclaredReading::FunctionCode {
            code,
            name,
            codes,
            water,
        } => {
            assert_eq!(code, "WFZ");
            assert_eq!(name.as_deref(), Some("GNM"));
            assert_eq!(codes.len(), 2);
            assert_eq!(codes[0].code, "7110");
            assert_eq!(codes[0].kind, ForestFunctionKind::Recreation);
            assert_eq!(codes[0].label, None);
            assert_eq!(codes[1].kind, ForestFunctionKind::Protected);
            assert_eq!(codes[1].label.as_deref(), Some("Boden"));
            let water = water.as_ref().unwrap();
            assert_eq!(
                (water.prefix.as_str(), water.length, water.zone.as_str()),
                ("211", 5, "KNM")
            );
            assert_eq!(water.after, " Z ");
        }
        other => panic!("{other:?}"),
    }
    // Without the water codes and the name, and a table of one code.
    let plain = format!(
        "{POINT_HEAD}layers = [{{ key = \"f\", url = \"https://geo.example.org/wms\", wms_layers = \"1\", reading = \"function-code\", code_attribute = \"C\", codes = [{{ code = \"1\", function = \"natural\" }}] }}]\n"
    );
    let manifest = parse(&with(&plain), Path::new("sample")).unwrap();
    assert!(!manifest.permits[0].layers[0].query.as_ref().unwrap().one);
    assert!(matches!(
        manifest.permits[0].layers[0].reading,
        DeclaredReading::FunctionCode {
            name: None,
            water: None,
            ..
        }
    ));
    // The tables come in the order of services, shapefiles, points.
    let both = format!(
        "{POINT_HEAD}layers = [{POINT_REVIER}]\n\n{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n"
    );
    let keys: Vec<String> = parse(&with(&both), Path::new("sample"))
        .unwrap()
        .permits
        .into_iter()
        .map(|source| source.key)
        .collect();
    assert_eq!(keys, ["natur", "forst"]);
}

#[test]
fn what_is_wrong_with_a_source_asked_at_points_is_named() {
    let one = |layer: &str| refused(&format!("{POINT_HEAD}layers = [{layer}]\n"));
    // The keys of a service or a zip are no keys of it, and it has no address of its own.
    for (extra, key) in [
        ("output_format = \"GEOJSON\"\n", "output_format"),
        ("gml = true\n", "gml"),
        ("native_crs = 25832\n", "native_crs"),
        ("offices = \"berlin\"\n", "offices"),
        ("crs = 25832\n", "crs"),
    ] {
        assert_eq!(
            refused(&format!("{POINT_HEAD}{extra}layers = [{POINT_REVIER}]\n")),
            [("manifest.unknown-key", key.to_string())],
            "{key}"
        );
    }
    assert_eq!(
        refused(&format!(
            "{POINT_HEAD}url = \"https://geo.example.org/wms\"\nlayers = [{POINT_REVIER}]\n"
        )),
        [("manifest.unknown-key", "url".to_string())]
    );
    // A layer needs its service and its layers; a feature type is no key of it.
    assert_eq!(
        one(&POINT_REVIER.replace("url = \"https://geo.example.org/wms\", ", "")),
        [("manifest.missing-key", "url".to_string())]
    );
    assert_eq!(
        one(&POINT_REVIER.replace("wms_layers = \"Reviere\", ", "")),
        [("manifest.missing-key", "wms_layers".to_string())]
    );
    assert_eq!(
        one(&POINT_REVIER.replace(
            "key = \"reviere\", ",
            "key = \"reviere\", type_name = \"x:a\", "
        )),
        [("manifest.unknown-key", "type_name".to_string())]
    );
    // The service is asked as it is, and its host is one of the manifest.
    assert_eq!(
        one(&POINT_REVIER.replace("wms\"", "wms?a=b\"")),
        [(
            "policy.url",
            "permits.getfeatureinfo.layers.url".to_string()
        )]
    );
    assert_eq!(
        one(&POINT_REVIER.replace("geo.example.org", "other.example.org")),
        [(
            "policy.host-not-declared",
            "permits.getfeatureinfo.layers.url".to_string()
        )]
    );
    // The layers are names of a WMS, at most 24, in at most 200 characters.
    for layers in ["", "a b", "a,,b", "a;b"] {
        assert_eq!(
            one(&POINT_REVIER.replace("\"Reviere\"", &format!("\"{layers}\""))),
            [(
                "manifest.value",
                "permits.getfeatureinfo.layers.wms_layers".to_string()
            )],
            "{layers}"
        );
    }
    let many = |n: usize| {
        (0..n)
            .map(|i| format!("l{i}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    assert!(
        parse(
            &with(&format!(
                "{POINT_HEAD}layers = [{}]\n",
                POINT_REVIER.replace("Reviere", &many(24))
            )),
            Path::new("sample")
        )
        .is_ok()
    );
    assert_eq!(
        one(&POINT_REVIER.replace("Reviere", &many(25))),
        [(
            "manifest.value",
            "permits.getfeatureinfo.layers.wms_layers".to_string()
        )]
    );
    let long = format!("{}a", "a".repeat(100));
    let text = format!("{long},{long}");
    assert!(text.len() > 200);
    assert_eq!(
        one(&POINT_REVIER.replace("Reviere", &text)),
        [(
            "manifest.value",
            "permits.getfeatureinfo.layers.wms_layers".to_string()
        )]
    );
    // The three keys belong to these layers only.
    for (extra, key) in [
        ("url = \"https://geo.example.org/wfs\", ", "url"),
        ("wms_layers = \"a\", ", "wms_layers"),
        ("one = true, ", "one"),
    ] {
        assert_eq!(
            refused(&format!(
                "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", {extra}reading = \"district\", name = \"N\" }}]\n"
            )),
            [("manifest.unknown-key", key.to_string())],
            "{key}"
        );
        assert_eq!(
            refused(&format!(
                "{SHAPE_HEAD}layers = [{{ key = \"w\", {extra}reading = \"water\", name = \"N\", zone = \"Z\" }}]\n"
            )),
            [("manifest.unknown-key", key.to_string())],
            "{key}"
        );
    }
    // A source asked at points has 1 to 8 layers.
    assert_eq!(
        refused(&format!("{POINT_HEAD}layers = []\n")),
        [(
            "manifest.value",
            "permits.getfeatureinfo.layers".to_string()
        )]
    );
    // Its key is one of the permit sources, whatever the table.
    assert_eq!(
        refused(&format!(
            "{POINT_HEAD}layers = [{POINT_REVIER}]\n\n{}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n",
            HEAD.replace("\"natur\"", "\"forst\"")
        )),
        [("manifest.value", "permits.getfeatureinfo.key".to_string())]
    );
}

#[test]
fn the_keys_of_the_function_code_and_the_office_renames_are_checked() {
    let one = |layer: &str| refused(&format!("{POINT_HEAD}layers = [{layer}]\n"));
    let codes_key = "codes = [{ code = \"7110\", function = \"recreation\" }, { code = \"1110\", function = \"protected\", label = \"Boden\" }], ";
    // The reading needs its attribute and 1 to 8 codes.
    assert_eq!(
        one(&POINT_CODES.replace("code_attribute = \"WFZ\", ", "")),
        [("manifest.missing-key", "code_attribute".to_string())]
    );
    assert_eq!(
        one(&POINT_CODES.replace(codes_key, "")),
        [("manifest.value", "codes".to_string())]
    );
    assert_eq!(
        one(&POINT_CODES.replace(codes_key, "codes = [], ")),
        [("manifest.value", "codes".to_string())]
    );
    let codes = |n: usize| {
        let list: Vec<String> = (0..n)
            .map(|i| format!("{{ code = \"c{i}\", function = \"natural\" }}"))
            .collect();
        POINT_CODES.replace(codes_key, &format!("codes = [{}], ", list.join(", ")))
    };
    assert!(
        parse(
            &with(&format!("{POINT_HEAD}layers = [{}]\n", codes(8))),
            Path::new("sample")
        )
        .is_ok()
    );
    assert_eq!(one(&codes(9)), [("manifest.value", "codes".to_string())]);
    // A code names a function the program knows.
    assert_eq!(
        one(&POINT_CODES.replace("\"recreation\"", "\"picnic\"")),
        [(
            "manifest.value",
            "permits.wfs.layers.codes.function".to_string()
        )]
    );
    // The four keys of the water codes come together, or none of them.
    for half in [
        "water_prefix = \"211\", ",
        "water_length = 5, ",
        "zone = \"KNM\", ",
        "zone_after = \" Z \"",
    ] {
        let layer = POINT_CODES.replace(half, "");
        let layer = if half.starts_with("zone_after") {
            layer.replace(", }", " }")
        } else {
            layer
        };
        assert_eq!(
            one(&layer),
            [("manifest.missing-key", "water_prefix".to_string())],
            "{half}"
        );
    }
    let without = POINT_CODES
        .replace("water_prefix = \"211\", ", "")
        .replace("water_length = 5, ", "")
        .replace("zone = \"KNM\", ", "")
        .replace(", zone_after = \" Z \"", "");
    assert!(
        parse(
            &with(&format!("{POINT_HEAD}layers = [{without}]\n")),
            Path::new("sample")
        )
        .is_ok()
    );
    for length in ["0", "21"] {
        assert_eq!(
            one(&POINT_CODES.replace("water_length = 5", &format!("water_length = {length}"))),
            [(
                "manifest.value",
                "permits.wfs.layers.water_length".to_string()
            )],
            "{length}"
        );
    }
    // No other reading has these keys.
    for extra in [
        "code_attribute = \"C\"",
        "codes = [{ code = \"1\", function = \"natural\" }]",
        "water_prefix = \"211\"",
        "water_length = 5",
        "zone_after = \" Z \"",
        "office_renames = [{ starts_with = \"a\", name = \"b\" }]",
    ] {
        // A pair is named by its first key.
        let key = match extra.split(' ').next().unwrap() {
            "codes" => "code_attribute",
            "water_length" | "zone_after" => "water_prefix",
            other => other,
        };
        assert_eq!(
            refused(&format!(
                "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\", {extra} }}]\n"
            )),
            [("manifest.unknown-key", key.to_string())],
            "{key}"
        );
    }
    // One or two renames of an office, on a unit and on the state forest.
    let renames = |n: usize| {
        let list: Vec<String> = (0..n)
            .map(|i| format!("{{ starts_with = \"a{i}\", name = \"Name {i}\" }}"))
            .collect();
        format!("office_renames = [{}]", list.join(", "))
    };
    let revier = |renames: &str| {
        POINT_REVIER.replace(
            "office_renames = [{ starts_with = \"Nationalpark\", name = \"Nationalparkverwaltung\" }]",
            renames,
        )
    };
    for n in [1, 2] {
        assert!(
            parse(
                &with(&format!("{POINT_HEAD}layers = [{}]\n", revier(&renames(n)))),
                Path::new("sample")
            )
            .is_ok(),
            "{n}"
        );
    }
    for n in [0, 3] {
        assert_eq!(
            one(&revier(&renames(n))),
            [("manifest.value", "office_renames".to_string())],
            "{n}"
        );
    }
    let state_forest = format!(
        "{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"state-forest\", owner = \"Land\", prefix = \"Amt \", office = \"B\", {} }}]\n",
        renames(1)
    );
    assert!(parse(&with(&state_forest), Path::new("sample")).is_ok());
}

#[test]
fn what_holds_up_to_300_characters_and_label_up_to_200() {
    let layers =
        "layers = [{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }]\n";
    let with_what = |n: usize| HEAD.replace("geschützte Objekte", &"x".repeat(n));
    assert!(
        parse(
            &with(&format!("{}{layers}", with_what(300))),
            Path::new("sample")
        )
        .is_ok()
    );
    assert_eq!(
        refused(&format!("{}{layers}", with_what(301))),
        [("manifest.value", "permits.wfs.what.de".to_string())]
    );
    let with_label = |n: usize| {
        HEAD.replace(
            "label = { en = \"Geoportal\", de = \"Geoportal\" }",
            &format!(
                "label = {{ en = \"Geoportal\", de = \"{}\" }}",
                "x".repeat(n)
            ),
        )
    };
    assert!(
        parse(
            &with(&format!("{}{layers}", with_label(200))),
            Path::new("sample")
        )
        .is_ok()
    );
    assert_eq!(
        refused(&format!("{}{layers}", with_label(201))),
        [("manifest.value", "permits.wfs.label.de".to_string())]
    );
}

#[test]
fn the_messages_name_the_kind_of_the_source() {
    // A WFS source without its address.
    assert_eq!(
        messages(&format!(
            "{}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n",
            HEAD.replace("url = \"https://geo.example.org/wfs\"\n", "")
        )),
        ["a WFS source needs `url`"]
    );
    // A foreign key of a shapefile source and of a source asked at points.
    assert_eq!(
        messages(&format!("{SHAPE_HEAD}gml = true\n{SHAPE_LAYER}")),
        ["a shapefile source has no key `gml`"]
    );
    assert_eq!(
        messages(&format!(
            "{POINT_HEAD}gml = true\nlayers = [{POINT_REVIER}]\n"
        )),
        ["a source asked at points has no key `gml`"]
    );
    // A feature type in a layer of either.
    assert_eq!(
        messages(&format!(
            "{SHAPE_HEAD}layers = [{{ key = \"w\", type_name = \"x:w\", reading = \"water\", name = \"N\", zone = \"Z\" }}]\n"
        )),
        ["the layer of a shapefile source has no feature type"]
    );
    assert_eq!(
        messages(&format!(
            "{POINT_HEAD}layers = [{}]\n",
            POINT_REVIER.replace(
                "key = \"reviere\", ",
                "key = \"reviere\", type_name = \"x:a\", "
            )
        )),
        ["the layer of a source asked at points has no feature type"]
    );
}

const GEOJSON_HEAD: &str = "[[permits.geojson]]\nkey = \"denkmale\"\n\
    label = { en = \"Office\", de = \"Amt\" }\n\
    what = { en = \"monuments\", de = \"Denkmale\" }\n\
    credit = \"CC BY 4.0\"\n\
    sections = [\"monuments\"]\n\
    url = \"https://geo.example.org/files/list.geojson\"\n\
    crs = 25832\n";

const GEOJSON_LAYER: &str = "layers = [{ key = \"d\", reading = \"monument\", monument = \"Denkmal\", kind_from = \"Art\" }]\n";

#[test]
fn a_geojson_source_is_declared_with_its_file_and_crs() {
    let table = format!("{GEOJSON_HEAD}note = \"A note\"\n{GEOJSON_LAYER}");
    let manifest = parse(&with(&table), Path::new("sample")).unwrap();
    let source = &manifest.permits[0];
    assert!(source.geojson);
    assert!(!source.shapefile && !source.getfeatureinfo);
    assert_eq!(source.url, "https://geo.example.org/files/list.geojson");
    assert_eq!(source.native_crs, Some(25832));
    assert_eq!(source.output_format, "");
    assert!(!source.gml && source.offices.is_none());
    assert_eq!(source.note.as_deref(), Some("A note"));
    assert_eq!(source.layers.len(), 1);
    assert_eq!(source.layers[0].type_name, "");
    assert!(matches!(
        source.layers[0].reading,
        DeclaredReading::Monument { .. }
    ));
    // The other tables are no GeoJSON files.
    let shape = parse(
        &with(&format!("{SHAPE_HEAD}{SHAPE_LAYER}")),
        Path::new("sample"),
    )
    .unwrap();
    assert!(!shape.permits[0].geojson);
    // The order: services, shapefiles, files.
    let all = format!(
        "{GEOJSON_HEAD}{GEOJSON_LAYER}\n{SHAPE_HEAD}{SHAPE_LAYER}\n{HEAD}layers = [{{ key = \"a\", type_name = \"x:a\", reading = \"district\", name = \"N\" }}]\n"
    );
    let keys: Vec<String> = parse(&with(&all), Path::new("sample"))
        .unwrap()
        .permits
        .into_iter()
        .map(|source| source.key)
        .collect();
    assert_eq!(keys, ["natur", "wsg", "denkmale"]);
}

#[test]
fn what_is_wrong_with_a_geojson_source_is_named() {
    for (extra, key) in [
        ("output_format = \"GEOJSON\"\n", "output_format"),
        ("gml = true\n", "gml"),
        ("native_crs = 25832\n", "native_crs"),
        ("offices = \"berlin\"\n", "offices"),
    ] {
        assert_eq!(
            refused(&format!("{GEOJSON_HEAD}{extra}{GEOJSON_LAYER}")),
            [("manifest.unknown-key", key.to_string())],
            "{key}"
        );
    }
    // It needs its CRS, which must be one of a map.
    assert_eq!(
        refused(&format!(
            "{}{GEOJSON_LAYER}",
            GEOJSON_HEAD.replace("crs = 25832\n", "")
        )),
        [("manifest.missing-key", "crs".to_string())]
    );
    assert_eq!(
        refused(&format!(
            "{}{GEOJSON_LAYER}",
            GEOJSON_HEAD.replace("25832", "4326")
        )),
        [("manifest.crs", "permits.geojson.crs".to_string())]
    );
    // It needs its address.
    assert_eq!(
        refused(&format!(
            "{}{GEOJSON_LAYER}",
            GEOJSON_HEAD.replace("url = \"https://geo.example.org/files/list.geojson\"\n", "")
        )),
        [("manifest.missing-key", "url".to_string())]
    );
    // Exactly one layer, without a feature type.
    let layer = "{ key = \"d\", reading = \"monument\", monument = \"Denkmal\" }";
    for layers in [
        "layers = []\n".to_string(),
        format!("layers = [{layer}, {}]\n", layer.replace("\"d\"", "\"e\"")),
    ] {
        assert_eq!(
            refused(&format!("{GEOJSON_HEAD}{layers}")),
            [("manifest.value", "permits.geojson.layers".to_string())],
            "{layers}"
        );
    }
    assert_eq!(
        refused(&format!(
            "{GEOJSON_HEAD}layers = [{{ key = \"d\", type_name = \"x:d\", reading = \"monument\", monument = \"Denkmal\" }}]\n"
        )),
        [("manifest.unknown-key", "type_name".to_string())]
    );
    // A file is asked as it is: no query.
    assert_eq!(
        refused(&format!(
            "{}{GEOJSON_LAYER}",
            GEOJSON_HEAD.replace("list.geojson", "list.geojson?a=b")
        )),
        [("policy.url", "permits.geojson.url".to_string())]
    );
    // The host must be one of the manifest.
    assert_eq!(
        refused(&format!(
            "{}{GEOJSON_LAYER}",
            GEOJSON_HEAD.replace("geo.example.org", "other.example.org")
        )),
        [(
            "policy.host-not-declared",
            "permits.geojson.url".to_string()
        )]
    );
    // The keys of the tables are one set, and the sources at most twelve.
    assert_eq!(
        refused(&format!(
            "{GEOJSON_HEAD}{GEOJSON_LAYER}\n{}{SHAPE_LAYER}",
            SHAPE_HEAD.replace("\"wsg\"", "\"denkmale\"")
        )),
        [("manifest.value", "permits.geojson.key".to_string())]
    );
    let files = |n: usize| {
        (0..n)
            .map(|n| {
                format!(
                    "{}{GEOJSON_LAYER}",
                    GEOJSON_HEAD.replace("\"denkmale\"", &format!("\"f{n}\""))
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert!(parse(&with(&files(12)), Path::new("sample")).is_ok());
    assert_eq!(
        refused(&files(13)),
        [("manifest.value", "permits.wfs".to_string())]
    );
}
