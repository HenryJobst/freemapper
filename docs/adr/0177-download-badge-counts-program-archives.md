# ADR 0177: The download badge counts the program archives

Date: 2026-11-22

Status: Accepted, amends ADR 0070, 0163 and 0176

## Context

The README showed the download count of shields.io, the sum of `download_count` over every asset of
every release. A release holds the archives of the program, but also their checksums, the update
manifest with its signature, the `.gz` files of `freemapper update` and the test archives of the
golden files, which CI fetches. One installation raises several of these counters, CI raises
others, and the number was 909 where the archives stood at 295. shields.io cannot filter by asset
name with a pattern.

The badges also stood in two places, a generated block and a hand-written row, and mixed what the
gates measure with what GitHub reports.

## Decision

- `scripts/downloads.sh` sums `download_count` of the release archives only
  (`freemapper-<tag>-<platform>.tar.gz` and `.zip`) and prints a shields.io endpoint document.
- The workflow `downloads-badge.yml` runs it daily and on every published release, and keeps the
  result in `downloads.json` and `downloads.en.json` on the branch `badges`, which holds nothing
  else and is replaced on every run. The README badge reads the file through the endpoint badge of
  shields.io.
- `scripts/badges.sh` writes all badges, in three rows: the project live (version, downloads,
  commits, CI), what the gates measured (tests, test budget, coverage, hygiene) and what the
  repository declares (MSRV, ADRs, licence). The AI-cost badge keeps its own markers.

## Consequences

- The count is a lower bound of the installations: a person who copies the program from a friend or
  builds it from source is not in it, and a person who downloads twice is in it twice.
- The number is as old as the last run, at most a day.
- The branch `badges` is a second ref in the repository. The workflow needs `contents: write`.
- Until the first run the endpoint badge reads "invalid"; `workflow_dispatch` starts it by hand.
