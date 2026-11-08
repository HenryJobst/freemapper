# 0007. Glossary first, English code, bilingual user docs

Date: 2026-11-06
Status: Accepted

## Context

The users and the official data are German. The code, the specs and most tools are English. Without a fixed
mapping, one concept gets several names across code, docs and data sources.

## Decision

- `docs/glossary.md` came first. Each row holds the German term, the English term, a definition and the code
  identifier. Code and docs use these terms.
- Source code is entirely English.
- The README and the user manual (`docs/manual/de.md`, `docs/manual/en.md`) are bilingual. Technical specs,
  plans and region references are English.

## Consequences

- A new concept needs a glossary row before or with its code, for example "Rasterregion / raster region" or
  "Anbieter / provider".
- Names follow the glossary even when a plan said otherwise: the M7 rule field became `target`/`Target`
  because `layers::Layer` (Ebene) already existed, and the bDOM became `Product::ImageDsm`.
- Cost if wrong: each user-facing change is written twice, and the two manuals can drift apart.

Source: spec header and §3 (Language, Docs); plan-mode notes "Entschieden (vom Nutzer)"; commit 65a8444;
commit 0c70860; M7 rulings (Task 1).
