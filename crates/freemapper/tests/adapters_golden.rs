//! The golden net of the region adapter migration (ADR 0082, plan stage A). Five layers, all
//! offline, recorded from the v0.12.0 behaviour. A refactor must leave every file under
//! `tests/golden/adapters/` as it is; `FREEMAPPER_BLESS=1` rewrites them on purpose, see
//! `tests/golden/adapters/README.md`.

use std::path::{Path, PathBuf};

use freemapper::sources::region::golden::legacy_facts;

/// `tests/golden/adapters/` at the repository root.
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/adapters")
}

fn blessing() -> bool {
    std::env::var_os("FREEMAPPER_BLESS").is_some()
}

/// Compares `actual` with the golden file `name`, or writes it when blessing.
fn check_golden(name: &str, actual: &str) {
    let path = golden_dir().join(name);
    if blessing() {
        std::fs::create_dir_all(path.parent().expect("golden file has a folder")).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; record it with FREEMAPPER_BLESS=1", path.display()));
    // Line endings are not part of the record (a Windows checkout may turn LF into CRLF).
    assert!(
        expected.replace("\r\n", "\n") == actual,
        "{name} differs from the recorded behaviour; a refactor must not change it"
    );
}

#[test]
fn g1_registry_data_is_unchanged() {
    let facts = legacy_facts();
    assert_eq!(facts.len(), 11);
    let mut text = serde_json::to_string_pretty(&facts).unwrap();
    text.push('\n');
    check_golden("g1-regions.json", &text);
}
