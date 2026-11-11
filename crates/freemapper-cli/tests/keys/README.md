These keys are for tests only: the test key pair of freemapper-core's `freemapper-update` crate
(`crates/freemapper-update/tests/keys/test.*`, key id 09F80E494B3AE92B), copied here because the integration tests
sign manifests with it. It has no password, and its secret key is public on purpose. A release build never accepts
it: `src/update/mod.rs` adds it to the accepted keys only in unit tests and in a debug build with the `test-stub`
feature, and refuses to compile with `FREEMAPPER_DIST=github-release` if `assets/update/minisign.pub` (the release
key) ever holds it.

`../fixtures/update/manifest-99.json.minisig` was made with
`minisign -S -s test.key -m manifest-99.json -x manifest-99.json.minisig -t "freemapper test fixture"`, and `manifest-99-manual.json.minisig` (the same release, but
`min_self_update_from` 98.0.0) the same way. The test
`update_yes_installs_over_a_copied_binary` signs its own manifest the same way at run time and is skipped when
`minisign` is not on the PATH, unless `FREEMAPPER_TEST_REQUIRE_MINISIGN` is set (CI sets it), which makes it fail
instead.
