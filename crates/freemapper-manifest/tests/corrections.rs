// SPDX-License-Identifier: AGPL-3.0-or-later

//! The list of corrections of addresses (issue #273, stage 4): reading it and applying it.

use std::path::Path;

use freemapper_manifest::corrections::{
    Applied, Correction, List, ListError, MAX_BYTES, MAX_CORRECTIONS, Outcome, apply, parse_list,
};
use freemapper_manifest::edit::addresses;

fn sample() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/invalid/unknown-key.toml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .replace("colour = \"red\"\n", "")
}

const OLD: &str = "https://geo.example.org/alkis/wfs";

fn correction(to: &str) -> Correction {
    Correction {
        adapter: "sample".into(),
        path: "alkis.url".into(),
        from: OLD.into(),
        to: to.into(),
        add_host: false,
        note: String::new(),
    }
}

fn list_of(corrections: Vec<Correction>) -> List {
    List {
        format: 1,
        issued: "2026-11-27".into(),
        corrections,
    }
}

fn run(list: &List) -> Applied {
    let text = sample();
    apply(list, |id| (id == "sample").then(|| text.clone()))
}

fn json(corrections: &str) -> String {
    format!(r#"{{"format":1,"issued":"2026-11-27","corrections":[{corrections}]}}"#)
}

const GOOD: &str = r#"{"adapter":"sample","path":"alkis.url","from":"https://a.example.org/a","to":"https://a.example.org/b"}"#;

#[test]
fn a_list_is_read_with_its_defaults() {
    let list = parse_list(json(GOOD).as_bytes()).unwrap();
    assert_eq!(list.issued, "2026-11-27");
    assert_eq!(list.corrections.len(), 1);
    assert!(!list.corrections[0].add_host);
    assert_eq!(list.corrections[0].note, "");
    assert_eq!(parse_list(json("").as_bytes()).unwrap().corrections, []);
}

#[test]
fn a_list_that_is_not_of_this_schema_is_refused() {
    let bad = |text: &str| parse_list(text.as_bytes()).unwrap_err();
    assert!(matches!(bad("not json"), ListError::Syntax(_)));
    assert!(matches!(
        bad(r#"{"format":1,"issued":"2026-11-27","corrections":[],"extra":1}"#),
        ListError::Syntax(_)
    ));
    assert_eq!(
        bad(r#"{"format":2,"issued":"2026-11-27","corrections":[]}"#),
        ListError::Format(2)
    );
    assert_eq!(
        bad(r#"{"format":1,"issued":"27.11.2026","corrections":[]}"#),
        ListError::Issued
    );
    assert_eq!(
        bad(r#"{"format":1,"issued":"2026-02-30","corrections":[]}"#),
        ListError::Issued
    );
}

fn one(adapter: &str, path: &str, from: &str, to: &str, note: &str) -> Result<List, ListError> {
    let text = serde_json::json!({
        "format": 1,
        "issued": "2026-11-27",
        "corrections": [{"adapter": adapter, "path": path, "from": from, "to": to, "note": note}],
    })
    .to_string();
    parse_list(text.as_bytes())
}

#[test]
fn every_field_of_a_correction_is_checked() {
    let (a, b) = ("https://a.example.org/a", "https://a.example.org/b");
    assert!(one("sample", "alkis.url", a, b, "").is_ok());
    // The shapes of the ids: lower case, 2 to 32 characters, no path.
    for bad in ["Sample", "a", "../x", "with space", &"a".repeat(33)] {
        assert_eq!(
            one(bad, "alkis.url", a, b, ""),
            Err(ListError::Field(0, "adapter")),
            "{bad}"
        );
    }
    assert!(one(&"a".repeat(32), "alkis.url", a, b, "").is_ok());
    assert!(one("ab", "alkis.url", a, b, "").is_ok());
    assert_eq!(
        one("sample", "alkis.url", "http://a.example.org/a", b, ""),
        Err(ListError::Field(0, "from"))
    );
    assert_eq!(
        one("sample", "alkis.url", a, "ftp://a.example.org/b", ""),
        Err(ListError::Field(0, "to"))
    );
    let long = format!("https://h.example.org/{}", "x".repeat(2000 - 22));
    assert_eq!(long.len(), 2000);
    assert!(one("sample", "alkis.url", a, &long, "").is_ok());
    assert_eq!(
        one("sample", "alkis.url", a, &format!("{long}x"), ""),
        Err(ListError::Field(0, "to"))
    );
    assert_eq!(
        one("sample", "", a, b, ""),
        Err(ListError::Field(0, "path"))
    );
    assert!(one("sample", &"p".repeat(128), a, b, "").is_ok());
    assert_eq!(
        one("sample", &"p".repeat(129), a, b, ""),
        Err(ListError::Field(0, "path"))
    );
    assert!(one("sample", "alkis.url", a, b, &"n".repeat(300)).is_ok());
    assert_eq!(
        one("sample", "alkis.url", a, b, &"n".repeat(301)),
        Err(ListError::Field(0, "note"))
    );
    assert_eq!(
        one("sample", "alkis.url", a, b, "two\nlines"),
        Err(ListError::Field(0, "note"))
    );
}

#[test]
fn the_size_and_the_count_are_limited() {
    assert_eq!(
        parse_list(&vec![b' '; MAX_BYTES + 1]),
        Err(ListError::TooLarge)
    );
    let many = vec![GOOD; MAX_CORRECTIONS + 1].join(",");
    assert_eq!(parse_list(json(&many).as_bytes()), Err(ListError::TooMany));
    let enough = vec![GOOD; MAX_CORRECTIONS].join(",");
    assert!(parse_list(json(&enough).as_bytes()).is_ok());
}

#[test]
fn a_correction_changes_the_one_value_and_nothing_else() {
    let new = "https://geo.example.org/alkis/wfs2";
    let applied = run(&list_of(vec![correction(new)]));
    assert_eq!(
        applied.rows[0].outcome,
        Outcome::Applied { host_added: None }
    );
    let text = &applied.texts["sample"];
    assert_eq!(*text, sample().replace(OLD, new));
    let value = addresses(text)
        .unwrap()
        .into_iter()
        .find(|a| a.path == "alkis.url")
        .unwrap()
        .value;
    assert_eq!(value, new);
}

#[test]
fn a_new_host_needs_add_host_and_then_joins_the_hosts() {
    let to = "https://new.example.org/wfs";
    let applied = run(&list_of(vec![correction(to)]));
    assert!(
        matches!(&applied.rows[0].outcome, Outcome::Refused(why) if why.contains("new.example.org"))
    );
    assert!(applied.texts.is_empty());
    let mut with_host = correction(to);
    with_host.add_host = true;
    let applied = run(&list_of(vec![with_host]));
    assert_eq!(
        applied.rows[0].outcome,
        Outcome::Applied {
            host_added: Some("new.example.org".into())
        }
    );
    assert!(applied.texts["sample"].contains("\"new.example.org\""));
}

#[test]
fn a_correction_that_no_longer_fits_is_left_out() {
    let new = "https://geo.example.org/alkis/wfs2";
    // The manifest holds `to` already: a release brought it.
    let mut text = sample().replace(OLD, new);
    let list = list_of(vec![correction(new)]);
    let applied = apply(&list, |_| Some(text.clone()));
    assert_eq!(applied.rows[0].outcome, Outcome::AlreadyCurrent);
    assert!(applied.texts.is_empty());
    // The manifest holds a third address: another change came in between.
    text = sample().replace(OLD, "https://geo.example.org/alkis/other");
    let applied = apply(&list, |_| Some(text.clone()));
    assert_eq!(applied.rows[0].outcome, Outcome::Stale);
    assert!(applied.texts.is_empty());
}

#[test]
fn an_unknown_adapter_and_an_unknown_path_are_named() {
    let mut other = correction("https://geo.example.org/x");
    other.adapter = "nowhere".into();
    let mut path = correction("https://geo.example.org/x");
    path.path = "alkis.nothing".into();
    let applied = run(&list_of(vec![other, path]));
    assert_eq!(applied.rows[0].outcome, Outcome::UnknownAdapter);
    assert_eq!(applied.rows[1].outcome, Outcome::UnknownPath);
}

#[test]
fn a_change_that_breaks_the_manifest_is_refused_alone() {
    // An address of a private network passes the host check of `edit` but not the validator.
    let mut bad = correction("https://127.0.0.1/wfs");
    bad.add_host = true;
    let good = correction("https://geo.example.org/alkis/wfs2");
    let applied = run(&list_of(vec![bad, good]));
    assert!(matches!(applied.rows[0].outcome, Outcome::Refused(_)));
    assert_eq!(
        applied.rows[1].outcome,
        Outcome::Applied { host_added: None }
    );
    assert!(applied.texts["sample"].contains("alkis/wfs2"));
}

#[test]
fn corrections_of_one_manifest_apply_one_after_the_other() {
    let second = Correction {
        from: "https://geo.example.org/alkis/wfs2".into(),
        to: "https://geo.example.org/alkis/wfs3".into(),
        ..correction("")
    };
    let applied = run(&list_of(vec![
        correction("https://geo.example.org/alkis/wfs2"),
        second,
    ]));
    assert!(
        applied
            .rows
            .iter()
            .all(|row| matches!(row.outcome, Outcome::Applied { .. }))
    );
    assert!(applied.texts["sample"].contains("alkis/wfs3"));
}

/// A list of exactly the largest size is read; one byte more is refused.
#[test]
fn the_size_limit_is_exact() {
    let head = br#"{"format":1,"issued":"2026-11-27","corrections":[]}"#;
    let mut at_limit = head.to_vec();
    at_limit.resize(MAX_BYTES, b' ');
    assert!(parse_list(&at_limit).is_ok());
    let mut over = head.to_vec();
    over.resize(MAX_BYTES + 1, b' ');
    assert_eq!(parse_list(&over), Err(ListError::TooLarge));
}
