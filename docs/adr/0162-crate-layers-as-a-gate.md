# 0162. The crate layers are a gate

Date: 2026-11-20
Status: Accepted

## Context

The workspace has six crates in layers: `freemapper-adapter-api` is the contract; `freemapper-sources`
reads archives and rasters; `freemapper-manifest` reads the data manifests on top of the sources;
`freemapper-adapters-de` builds the German adapters on all three; `freemapper` is the library and
`freemapper-cli` the driver. The order held by habit only. A convenient `ureq` in a reader would
make it fetch instead of read; a `clap` or `tokio` in the library would drag a command line or an
async runtime into everything that builds on it.

## Decision

- **`scripts/check.sh architecture`** reads the workspace with `cargo metadata --no-deps` and
  fails when
  - a workspace member is in no layer (a new crate is a decision, made in the table `ALLOWED`),
  - a crate depends on a sibling its row does not list (every edge is listed, so a new edge shows
    in the diff),
  - a crate other than the CLI depends on a crate only the CLI may use: `anyhow`, `axum`, `clap`,
    `freemapper-update`, `freemapper-web`, `rmcp`, `rust-embed`, `schemars`, `tokio`,
  - a crate other than the library and the CLI depends on `ureq`: the readers and adapters stay
    offline, only the library fetches.
  Development dependencies are not counted.
- **Measured:** under a second, passes on today's tree. Probes that gave `freemapper-sources` a
  dependency on `freemapper-manifest` and on `ureq` each failed it with a line naming the crate;
  both were reverted.
- **Cheap gate:** it runs in `hygiene` and before a commit; the badge for the number of checks
  counts it.
- **Not covered:** `freemapper-site` is outside the workspace and has its own checks.

## Consequences

- A drive-by dependency against the layer order is a failing gate, not a review remark.
- Moving a crate between layers, or adding one, is an edit of the tables in `scripts/check.sh`
  with a reason in an ADR.
