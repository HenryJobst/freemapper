# 0018. The Rust toolchain is pinned in rust-toolchain.toml

Date: 2026-11-08
Status: Accepted

## Context

CI installed the newest stable Rust while local builds stayed on 1.96. The newer clippy reported a lint
(`chunks_exact_to_as_chunks`) that the local one did not, and every push since the Saxony merge failed in CI.

## Decision

`rust-toolchain.toml` pins channel 1.99.1 with `rustfmt` and `clippy`. Local builds, the CI workflow and the
release workflow all install the toolchain the file names. The release adds its target to it. The pin is moved by
hand, together with fixing what the newer clippy reports.

## Consequences

- Local and CI lints agree.
- Cost if wrong: freemapper stays on an older compiler until someone moves the pin.

Source: commit 6903438; `rust-toolchain.toml`.
