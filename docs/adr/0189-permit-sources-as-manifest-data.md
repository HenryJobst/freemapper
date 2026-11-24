# ADR 0189: Permit sources as manifest data

Date: 2026-11-23

Status: Accepted

Refs: issue #273, ADR 0130, ADR 0146, ADR 0188.

## Context

The WFS services behind the permit notes (protected areas, monuments, water, forest) were Rust tables: key, label, credit, address, output format and, per layer, a reading with the names of its attributes. Every moved service needed a release, and a folder adapter (ADR 0188) could not reach them. About 220 addresses sit in code; the tables are the largest share.

## Decision

A manifest declares such a source as `[[permits.wfs]]` (format 1, `docs/adapters/manifest-v1.en.md`). The validator checks it like every other table: https on a host of `[net] hosts`, no query, lengths, names, the sections and kinds the program knows, and the keys that belong to each of the six readings (`water`, `function`, `unit`, `state-forest`, `object`, `monument`). The declaration is plain owned data (`DeclaredWfs`, adapter-api). The adapter crate that reads the services registers a `PermitSourceFactory` with the registry, as it registers its raster backends; the manifest adapter asks it for the sources, and a program without one answers `manifest.backend-missing`. The factory keeps the texts for the rest of the run (a manifest holds at most 64 KiB, an adapter is made once), so the tables keep their types.

A built-in state takes the sources of its manifest after those its code brings. The first to move are the protected objects of Hamburg; a test holds them to what the table in the code gave: the same facts, the same requests and the same findings. The other tables follow state by state, one pull request each (issue #273, stage 2b); the protected objects of Saarland, the smallest table without constants of its own, came second, those of Brandenburg third and the biotopes of Berlin fourth and those of Hesse fifth and those of North Rhine-Westphalia sixth and those of Schleswig-Holstein seventh (its note of gaps may be up to 400 characters long), those of Baden-Württemberg eighth and those of Rhineland-Palatinate ninth and those of Saxony-Anhalt tenth and those of Bavaria eleventh and those of Saxony twelfth; the monuments of seven states (Hesse, Baden-Württemberg, Bremen, Saxony, Brandenburg, Rhineland-Palatinate, North Rhine-Westphalia) followed in one pull request, each held to its table by a test, and the module `denkmal_permits` is gone. A feature type of a permit layer may hold letters beyond ASCII (`ps:Flächennaturdenkmale`); the request writes them percent-encoded. The address of a permit service may carry a fixed query of plain `name=value` pairs that selects the service on the server (`?mapfile=glb`); `adapters probe` appends its question to it.

## Consequences

- A moved permit service of a migrated state is corrected in a folder (ADR 0188) without a release, and `adapters probe` (ADR 0185) reads its address from the manifest.
- The notes read the same. The order of the sources of a state is that of the code, then that of the manifest.
- A manifest that declares a source needs freemapper 0.28.0 or later; `min_freemapper` says so.
- The 64-URL limit of a manifest may need to grow when large states move their tables.
