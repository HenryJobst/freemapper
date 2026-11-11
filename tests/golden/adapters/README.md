# The golden net of the adapter work

The region adapters (ADR 0082) replace the `Region` and `Provider` enums by a trait, a registry and
manifests. All eleven states must produce the same output afterwards. The files in this folder
record what v0.12.0 does; the tests in `crates/freemapper/tests/adapters_golden.rs` compare the
current code with them.

| Layer | Records | Files | Needs |
|---|---|---|---|
| G1 | every fact the enums answer per region; the service constants; the credit lines of the project notes and the map sheet per state, year and record combination, and for neighbouring states in one map | `g1-regions.json`, `g1-services.txt`, `g1-credits.json` | the repository |
| G2 | what the region functions answer for 211 boxes on borders and in the example areas | `g2-boxes.json`, `g2-geometry.json` | the repository |
| G3 | the estimate, the requests and the warnings of an online fetch per state | `<id>/index.toml`, `<id>/trace.txt` | the fixture store |
| G3 | nine fetches that lose the network halfway (three states, three places): the requests, the warnings and the project file left behind | `g3-interrupted.json` | the fixture store |
| G4 | the SHA-256 of every file the pipeline leaves in the project folder, per state | `<id>/expected.sha256` | the fixture store |
| G5 | that a project file is read and written back unchanged | the example projects | the repository |

G1, G2 and G5 run with every `cargo test`. G3 and G4 are `#[ignore]`d, because they need the
fixture store:

```sh
scripts/golden.sh test
```

The script fetches the store when it is missing and runs all layers. Without the store, G3 and
G4 fail and name the script; they never pass by skipping.

All golden runs happen at a fixed time (2026-11-11, 12:00) through `freemapper::clock`, and notes
are compared in English.

## The fixture store

A fetch downloads whole tiles, and one tile set of a state has up to 37 MB. The answers of the
services therefore live outside the repository, in one archive per store version:

- `responses/<sha256>.bin`: the answers, named by their hash, one pool for all states.
- `<id>/osm-candidates.geojson`: the OSM objects of the golden area, read from the real extract.
  The replay answers every `.osm.pbf` URL with `empty.osm.pbf` (71 bytes, a header and no data),
  so no test downloads an extract; G4 puts this file in place before it derives.
- `<id>/ATTRIBUTION.txt`: whose data the folder holds and under which licence.

The repository keeps what a reviewer reads: per state the index (each request with the hash of
its answer or with its error) and the trace.

`store.lock` names the version, the URL, the SHA-256 and the size of the archive this tree needs.
The archive is the only asset of the GitHub prerelease `golden-adapters-<version>` of this
repository. `scripts/golden.sh fetch` downloads it with plain `curl`, refuses it when the hash
differs and unpacks it to `target/golden-adapters/<version>/`. `FREEMAPPER_GOLDEN_STORE` names
another folder, for the script and for the tests.

A published version is never changed. A new recording is a new version.

Version 0 was the pilot (Saxony-Anhalt and Hesse, 8.5 MB). Version 1 holds all eleven states:
206 MB as an archive, 234 MB unpacked. Most of it is whole tiles: Schleswig-Holstein 108 MB (one
surface model tile), Berlin 47 MB, Brandenburg and Saxony 31 MB each; the other seven states
have 7 MB or less each.

To look at the outputs of a run, or to compare them with another system's, set
`FREEMAPPER_GOLDEN_KEEP=1`: G4 then copies every file to `target/golden-outputs/<id>/`. A failing
G4 keeps the files that differ in `target/golden-diff/<id>/`, and the CI job `golden` uploads
that folder.

## When a golden changes

A commit whose subject starts with `refactor(` must not change any file here; the CI guard
refuses it. A commit that changes output on purpose writes the files again and says so:

```sh
FREEMAPPER_BLESS=1 cargo test -p freemapper --test adapters_golden       # G1, G2
FREEMAPPER_BLESS=1 cargo test -p freemapper --test adapters_golden -- --ignored g3_trace g4_pipeline
```

Its message names the changed files and carries the trailer `Golden-Bless: <reason>`. Read the
diff before committing: every changed line is a change in what users get.

## Recording a state again

A commit that adds or changes a request needs new answers. Recording talks to the real services.

1. Raise `version` in `store.lock` and copy the unpacked store of the old version to
   `target/golden-adapters/<new version>/`, so the states that stay keep their answers.
2. `scripts/golden.sh record <id>` for each state concerned. It fetches the golden area (about
   150 m square around the centre of the state's example, moved a little in four states so that
   it needs one tile, see `golden_centre`) into an empty folder with an empty cache, keeps every answer, and then writes `trace.txt` and `expected.sha256` from a replay.
   The maintainer's cached OSM extract is used if there is one, so no extract is downloaded.
3. `scripts/golden.sh pack` writes the archive and its hash into `store.lock`.
4. Publish the archive as the prerelease the script prints (`--prerelease --latest=false`), then
   check that <https://github.com/HenryJobst/freemapper/releases/latest> still leads to the newest
   program release: the install steps and `freemapper update` read `latest`. The prerelease
   carries the archive and nothing else, above all no file named `freemapper-update.json`.
5. Commit the indexes, traces, hashes and `store.lock` together, with `Golden-Bless:`.

No request, header or file of a recording carries an e-mail address or other personal data. The
recorder refuses a URL or a user agent that contains an `@`.

If a service is down, try again another day. If it stays down, record what answers and list the
state below; G3 then pins the failing request and the warning, which is behaviour too.

States recorded partially: none.
