// SPDX-License-Identifier: AGPL-3.0-or-later

//! The list of corrections in the repository (`corrections/freemapper-sources.json`, issue #273,
//! stage 4): it reads, and every correction in it still applies to the manifests this program
//! ships. A correction that a release brought into the manifests belongs out of the list; this
//! test fails until it is out, so that the list never holds what the program has.

use freemapper_adapters_de::states::{State, shipped_manifest_text};
use freemapper_manifest::corrections::{Outcome, apply, parse_list};

#[test]
fn every_correction_of_the_list_applies_to_the_shipped_manifests() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../corrections/freemapper-sources.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let list = parse_list(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let applied = apply(&list, |id| {
        State::ALL
            .into_iter()
            .find(|state| state.id() == id)
            .map(|state| shipped_manifest_text(state).to_string())
    });
    for row in &applied.rows {
        assert!(
            matches!(row.outcome, Outcome::Applied { .. }),
            "{}.{}: {:?} (a correction that a release brought in is deleted from the list)",
            row.correction.adapter,
            row.correction.path,
            row.outcome
        );
    }
}
