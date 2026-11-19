#!/usr/bin/env bash
# Writes the README badges from the data the repository carries (ADR 0070).
#
#   scripts/badges.sh            # rewrite the blocks
#   scripts/badges.sh --check    # fail when a block is out of date
#
# Every number has a source in the repository: the coverage from
# `docs/coverage.json`, the ADRs from `docs/adr/`, the crates from the member
# manifests, the MSRV and the licence from `Cargo.toml`, and the checks from
# the gates in `scripts/check.sh`. A number that cannot be read fails the run
# instead of showing a value nobody measured.
#
# The block stands between the `badges` markers in both READMEs, the German one
# and the English one; the AI-cost badge beside it belongs to its own service
# and is left alone.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

check=0
case "${1:-}" in
"") ;;
--check) check=1 ;;
*)
  echo "badges.sh: unknown argument '$1'; expected nothing or --check" >&2
  exit 2
  ;;
esac

start='<!-- badges:start -->'
end='<!-- badges:end -->'

number() {
  local value
  value=$(git ls-files "$1" | wc -l | tr -d ' ')
  echo "$value"
}

coverage=$(python3 - <<'PY'
import json

with open("docs/coverage.json") as handle:
    print(json.load(handle)["lines"])
PY
)
decisions=$(number 'docs/adr/[0-9]*.md')
crates=$(number 'crates/**/Cargo.toml')
msrv=$(sed -n 's/^rust-version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
licence=$(sed -n 's/^license = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
checks=$(grep -c '^gate_[a-z_]*()' scripts/check.sh)

if [ -z "$msrv" ] || [ -z "$licence" ]; then
  echo "badges.sh: Cargo.toml names no rust-version or no license" >&2
  exit 1
fi

# A shields message: `_` is a space and `--` a hyphen.
shields() {
  printf '%s' "$1" | sed -e 's/-/--/g' -e 's/ /_/g' -e 's/%/%25/g'
}

badge_block() {
  local language=$1
  local coverage_label msrv_label decisions_label hygiene_label licence_label
  local coverage_alt licence_alt

  if [ "$language" = de ]; then
    coverage_label=Coverage
    coverage_alt=Zeilenabdeckung
    licence_label=Lizenz
    licence_alt=Lizenz
  else
    coverage_label=Coverage
    coverage_alt="Line coverage"
    licence_label=License
    licence_alt=License
  fi

  msrv_label=MSRV
  decisions_label=ADRs
  hygiene_label=Hygiene

  cat <<EOF
[![$coverage_alt](https://img.shields.io/badge/$coverage_label-$(shields "$coverage")%25-brightgreen)](docs/coverage.json) [![$msrv_label](https://img.shields.io/badge/$msrv_label-$(shields "$msrv")-blue)](Cargo.toml) [![$decisions_label](https://img.shields.io/badge/$decisions_label-$decisions-blue)](docs/adr/README.md) [![Hygiene](https://img.shields.io/badge/$hygiene_label-$checks-brightgreen)](scripts/check.sh) [![$licence_alt](https://img.shields.io/badge/$(shields "$licence_label")-$(shields "$licence")-blue)](LICENSE)
EOF
}

failed=0

for document in README.md README.en.md; do
  if ! grep -qF "$start" "$document" || ! grep -qF "$end" "$document"; then
    echo "badges.sh: $document carries no badge markers" >&2
    failed=1
    continue
  fi

  language=de
  case "$document" in
  *.en.md) language=en ;;
  esac

  desired=$(badge_block "$language")
  current=$(awk -v start="$start" -v end="$end" '
    $0 == start { inside = 1; next }
    $0 == end { inside = 0; next }
    inside { print }
  ' "$document")

  if [ "$check" -eq 1 ]; then
    # A second copy outside the markers is a duplication, not an update: the
    # block is the only place the badges stand.
    copies=$(grep -c 'img.shields.io/badge/Coverage-' "$document" || true)

    if [ "$current" != "$desired" ] || [ "$copies" -ne 1 ]; then
      echo "badges.sh: the badges in $document are out of date; run scripts/badges.sh" >&2
      failed=1
    fi
    continue
  fi

  BLOCK="$desired" python3 - "$document" <<'PY'
import os
import pathlib
import sys

document = pathlib.Path(sys.argv[1])
lines = document.read_text().splitlines()
out = []
inside = False
for line in lines:
    if line == "<!-- badges:start -->":
        out.append(line)
        out.extend(os.environ["BLOCK"].splitlines())
        inside = True
        continue
    if line == "<!-- badges:end -->":
        out.append(line)
        inside = False
        continue
    if not inside:
        # A copy that an older run left outside the markers is dropped, so the
        # block is the only place the badges stand. The match is on the shape,
        # not on equality: such a copy carries the numbers of its own run.
        if "docs/coverage.json" in line and "img.shields.io/badge/Coverage-" in line:
            continue
        out.append(line)
document.write_text("\n".join(out) + "\n")
PY
done

if [ "$failed" -ne 0 ]; then
  exit 1
fi

if [ "$check" -eq 1 ]; then
  echo "badges.sh: up to date"
else
  echo "badges.sh: written ($coverage % coverage, $decisions ADRs, $crates crates, MSRV $msrv, $checks checks)"
fi
