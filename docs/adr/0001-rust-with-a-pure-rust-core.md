# 0001. Rust with a pure-Rust core

Date: 2026-11-06
Status: Accepted

## Context

freemapper must run on Linux, macOS and Windows for club mappers who have no GIS installed. The spec lists
"requiring GDAL/PROJ/Python or any other installation for normal use" as an anti-goal. Go and
Elixir/Phoenix/Ash were compared during brainstorming. Pure-Rust crates exist for LAS/LAZ, contours, geometry,
projections, XML, PBF and MCP.

## Decision

freemapper is written in Rust (edition 2024). The stack is pure Rust. C libraries such as GDAL, PROJ or PDAL may
come later only as optional features. All domain logic lives in the library crate `crates/freemapper`, so a later
hosted service can embed it. The binary crate `freemapper-cli` holds the CLI, the MCP server and the area page.
Releases ship one static binary per target: Linux x86_64 and ARM64 musl, macOS ARM and Intel, Windows.

## Consequences

- Users download one file and run it.
- freemapper carries code that GDAL would otherwise provide: the OOM and OCD writers, the ALKIS GML reader, a
  zip range reader, an xlsx reader and a lenient LZW decoder (ADR 0026).
- Cost if wrong: a format the Rust crates cannot read needs its own reader, or the optional GDAL feature has
  to be built earlier than planned.

Source: spec §2 (anti-goals), §3 (Language, Tech alternatives, Dependencies, Core as library), §4; plan-mode notes
"Entscheidungen von Claude"; commit 1d1bac2 (release workflow).
