These keys are for tests only: the test key pair of freemapper-core's `freemapper-update` crate
(`crates/freemapper-update/tests/keys/test.*`, key id 09F80E494B3AE92B), copied here because the integration tests
sign manifests with it. It has no password, and its secret key is public on purpose. A release build never accepts
it: `src/update/mod.rs` refuses to compile with `FREEMAPPER_DIST=github-release` while `assets/update/minisign.pub`
still holds this test key.

`../fixtures/update/manifest-99.json.minisig` was made with
`minisign -S -s test.key -m manifest-99.json -x manifest-99.json.minisig -t "freemapper test fixture"`. The test
`update_yes_installs_over_a_copied_binary` signs its own manifest the same way at run time and is skipped when
`minisign` is not on the PATH.
