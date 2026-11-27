// SPDX-License-Identifier: AGPL-3.0-or-later

//! The addresses of a manifest text and the change of one (issue #273, stage 3).

use freemapper_manifest::edit::{EditError, addresses, host_of, replace};

const TEXT: &str = r#"# a comment that stays
format = 1

[net]
hosts = ["a.example.org", "b.example.org"]

[alkis]
url = "https://a.example.org/wfs" # the cadastre
note = "not an address"
count = 5

[[permits.wfs]]
url = 'https://b.example.org/permits'

[[permits.wfs]]
url = "https://a.example.org/second"

[orthophoto.endpoints]
hm = "https://b.example.org/hm"
"#;

fn paths(text: &str) -> Vec<(String, String)> {
    addresses(text)
        .unwrap()
        .into_iter()
        .map(|a| (a.path, a.value))
        .collect()
}

#[test]
fn the_addresses_come_in_the_order_of_the_file_with_their_paths() {
    let expected = [
        ("net.hosts[0]", "a.example.org"),
        ("net.hosts[1]", "b.example.org"),
        ("alkis.url", "https://a.example.org/wfs"),
        ("permits.wfs[0].url", "https://b.example.org/permits"),
        ("permits.wfs[1].url", "https://a.example.org/second"),
        ("orthophoto.endpoints.hm", "https://b.example.org/hm"),
    ];
    assert_eq!(
        paths(TEXT),
        expected
            .iter()
            .map(|(p, v)| ((*p).to_string(), (*v).to_string()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn the_span_covers_the_value_with_its_quotes() {
    let all = addresses(TEXT).unwrap();
    let alkis = all.iter().find(|a| a.path == "alkis.url").unwrap();
    assert_eq!(&TEXT[alkis.span.clone()], "\"https://a.example.org/wfs\"");
    let single = all.iter().find(|a| a.path == "permits.wfs[0].url").unwrap();
    assert_eq!(
        &TEXT[single.span.clone()],
        "'https://b.example.org/permits'"
    );
}

#[test]
fn text_that_is_no_toml_is_a_syntax_error() {
    assert!(matches!(addresses("a = = 1"), Err(EditError::Syntax(_))));
}

#[test]
fn replace_changes_only_the_value_and_keeps_the_rest_byte_for_byte() {
    let new = replace(TEXT, "alkis.url", "https://a.example.org/wfs2", false)
        .unwrap()
        .text;
    assert_eq!(
        new,
        TEXT.replace(
            "\"https://a.example.org/wfs\"",
            "\"https://a.example.org/wfs2\""
        )
    );
    // The single-quoted value is replaced too, now with double quotes.
    let new = replace(
        TEXT,
        "permits.wfs[0].url",
        "https://b.example.org/p2",
        false,
    )
    .unwrap()
    .text;
    assert!(new.contains("url = \"https://b.example.org/p2\"\n"));
    assert!(new.contains("# a comment that stays"));
}

#[test]
fn replace_refuses_an_unknown_path_and_a_bad_value() {
    assert_eq!(
        replace(TEXT, "alkis.nothing", "https://a.example.org/x", false),
        Err(EditError::UnknownPath("alkis.nothing".to_string()))
    );
    for bad in [
        "",
        "https://a.example.org/\"x",
        "https://a.example.org/\\x",
        "https://a.example.org/\nx",
    ] {
        assert_eq!(
            replace(TEXT, "alkis.url", bad, false),
            Err(EditError::BadValue),
            "{bad:?}"
        );
    }
}

#[test]
fn a_new_host_needs_add_host_and_then_joins_the_hosts() {
    assert_eq!(
        replace(TEXT, "alkis.url", "https://New.Example.org/wfs", false),
        Err(EditError::HostNotDeclared("new.example.org".to_string()))
    );
    let added = replace(TEXT, "alkis.url", "https://New.Example.org/wfs", true).unwrap();
    assert_eq!(added.host_added.as_deref(), Some("new.example.org"));
    let new = added.text;
    assert!(new.contains("hosts = [\"a.example.org\", \"b.example.org\", \"new.example.org\"]"));
    assert!(new.contains("url = \"https://New.Example.org/wfs\" # the cadastre"));
    // A host that is declared is not added twice.
    let same = replace(TEXT, "alkis.url", "https://b.example.org/wfs", true).unwrap();
    assert_eq!(same.host_added, None);
    let same = same.text;
    assert!(same.contains("hosts = [\"a.example.org\", \"b.example.org\"]"));
}

#[test]
fn a_url_with_a_port_or_a_user_has_no_host() {
    assert_eq!(
        replace(TEXT, "alkis.url", "https://a.example.org:8443/x", true),
        Err(EditError::BadHost)
    );
    assert_eq!(
        replace(TEXT, "alkis.url", "https://user@a.example.org/x", true),
        Err(EditError::BadHost)
    );
    assert_eq!(
        host_of("https://A.example.org/x?y#z").as_deref(),
        Some("a.example.org")
    );
    assert_eq!(host_of("http://a.example.org/"), None);
    assert_eq!(host_of("https:///x"), None);
}

#[test]
fn a_host_entry_is_replaced_without_a_host_check() {
    let new = replace(TEXT, "net.hosts[1]", "c.example.org", false)
        .unwrap()
        .text;
    assert!(new.contains("hosts = [\"a.example.org\", \"c.example.org\"]"));
}
