// SPDX-License-Identifier: AGPL-3.0-or-later

//! The manifests in force after corrections of addresses (issue #273, stage 4). One test, because
//! the corrected manifests are set once for the whole process.

use std::collections::BTreeMap;

use freemapper_adapters_de::states::{
    State, manifest_text, manifest_texts, set_corrected_manifests, shipped_manifest_text,
};

#[test]
fn corrected_manifests_replace_the_shipped_ones_for_the_whole_process() {
    let shipped = shipped_manifest_text(State::Hamburg);
    assert_eq!(manifest_text(State::Hamburg), shipped);
    let corrected = format!("{shipped}\n# corrected\n");
    let first = BTreeMap::from([("hamburg".to_string(), corrected.clone())]);
    assert!(set_corrected_manifests(first));
    // The corrected state reads its new text, every other state its shipped one.
    assert_eq!(manifest_text(State::Hamburg), corrected);
    assert_eq!(shipped_manifest_text(State::Hamburg), shipped);
    assert_eq!(
        manifest_text(State::Berlin),
        shipped_manifest_text(State::Berlin)
    );
    let all = manifest_texts();
    let at = State::ALL
        .iter()
        .position(|state| *state == State::Hamburg)
        .unwrap();
    assert_eq!(all[at], corrected);
    // A second list does not replace the first: all readers of the process see one set.
    assert!(!set_corrected_manifests(BTreeMap::from([(
        "hamburg".to_string(),
        "# other".to_string()
    )])));
    assert_eq!(manifest_text(State::Hamburg), corrected);
}
