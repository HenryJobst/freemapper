#!/usr/bin/env bash
# Counts the downloads of the program itself and prints a shields.io endpoint document (ADR 0177).
#
#   scripts/downloads.sh [owner/repo]
#
# Only the release archives count: `freemapper-<tag>-<platform>.tar.gz` and `.zip`. The checksums, the
# update manifest and its signature, the `.gz` files of `freemapper update` and the test archives
# `golden-adapters-*` do not, because one installation fetches several of them or none of its own.
set -euo pipefail

repo=${1:-HenryJobst/freemapper}

total=$(gh api --paginate "repos/$repo/releases" --jq '
  [.[].assets[]
   | select(.name | test("^freemapper-v[0-9][^/]*-(macos|linux|windows)-[a-z0-9_]+\\.(tar\\.gz|zip)$"))
   | .download_count]
  | add // 0' | paste -sd+ - | bc)

printf '{"schemaVersion":1,"label":"%s","message":"%s","color":"blue"}\n' "${DOWNLOADS_LABEL:-Downloads}" "$total"
