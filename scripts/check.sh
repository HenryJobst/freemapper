#!/usr/bin/env bash
# The gates that CI and a commit run over this repository (ADR 0070).
#
#   scripts/check.sh                 # all cheap gates
#   scripts/check.sh typos links     # only these
#   scripts/check.sh spdx --fix      # write the licence line into files that lack it
#
# Gates:
#   spdx       every Rust file starts with the licence line
#   typos      the English documents carry no misspelling (typos, `_typos.toml`)
#   links      internal links and anchors resolve (lychee, offline)
#   secrets    no secret in the history (gitleaks, `.gitleaks.toml`)
#   machete    no dependency is declared and never used
#   actions    every action in `.github/workflows` hangs on a commit
#   adr        `docs/adr/README.md` lists exactly the ADRs that exist
#   changelog  `CHANGELOG.md` and `CHANGELOG.en.md` match `Cargo.toml`
#   badges     the README badges match the data in the repository
#   doc        rustdoc builds without warnings
#   coverage   line coverage is not below its floor or its snapshot
#   msrv       the workspace builds with the documented MSRV
#
# `coverage`, `doc` and `msrv` build the workspace and stay out of the default
# run; CI gives each its own job. The tools installed on the runner are pinned
# in `.github/workflows/ci.yml`; locally, `cargo install` or a release archive
# of the same version does the job.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

header='// SPDX-License-Identifier: AGPL-3.0-or-later'

# Floor of the line coverage in tenths of a percent. It catches a drop and is
# raised by hand once the code has grown into it; `docs/coverage.json` is the
# snapshot that follows the last measurement upwards.
#
# Coverage instrumentation differs between platforms and compiler patches: the
# same code measured 96.6 % on macOS and 96.3 % on Linux. A difference within
# this many tenths of a percent is noise, not a drop, and does not rewrite the
# snapshot.
coverage_tolerance=3

coverage_floor=940

failed=0

report() {
  echo "check.sh: $*" >&2
  failed=1
}

# Every Rust file in the repository, so a build script or example cannot slip
# past the header.
rust_files() {
  git ls-files '*.rs'
}

# The English documents. German documents, examples with German text inside
# their English README, fixtures and the vendor tree are not spell-checked;
# `_typos.toml` holds the German words that stand as quotes in English text.
english_documents() {
  git ls-files '*.md' | grep -Ev \
    '^(examples/|tests/fixtures/|assets/|docs/ai-cost/|docs/manual/de\.md$|docs/glossary\.md$|README\.md$|CHANGELOG\.md$)' |
    grep -E '(\.en\.md$|(^|/)AGENTS\.md$|(^|/)CLAUDE\.md$|^docs/adr/|^docs/superpowers/|/en\.md$)'
}

gate_spdx() {
  local fix=$1
  local file first tmp

  while IFS= read -r file; do
    first=$(head -n 1 "$file")

    if [ "$first" = "$header" ]; then
      continue
    fi

    if [ "$fix" = 1 ]; then
      tmp=$(mktemp)
      { printf '%s\n\n' "$header"; cat "$file"; } >"$tmp"
      mv "$tmp" "$file"
      continue
    fi

    report "no licence line in $file"
  done < <(rust_files)
}

gate_typos() {
  local files
  files=$(english_documents)

  if [ -z "$files" ]; then
    report "no English document found; the pattern in scripts/check.sh is stale"
    return
  fi

  # The file list goes through the config so that `--force-exclude` applies.
  echo "$files" | typos --config _typos.toml --force-exclude --format brief --file-list -
}

gate_links() {
  # Offline and with fragments: the check looks at this repository, not at the
  # internet, and `#anchor` links are half of the cross-references here. The
  # plans and specs under `docs/superpowers/` are a record of how the code
  # came to be, not living documents: they name files as they stood then.
  lychee --offline --no-progress --include-fragments \
    $(git ls-files '*.md' | grep -Ev '^(tests/fixtures/|docs/superpowers/)') ||
    report "a link does not resolve"
}

gate_secrets() {
  # The whole history: a token that left the working tree must still surface.
  gitleaks git --config .gitleaks.toml --redact --no-banner --exit-code 1 ||
    report "gitleaks found a secret"
}

gate_machete() {
  cargo machete || report "a dependency is declared and never used"
}

gate_actions() {
  local file value ref

  while IFS= read -r file; do
    while IFS= read -r line; do
      value=${line#*uses:}
      value=${value%%#*}
      value=$(printf '%s' "$value" | tr -d "[:space:]\"'")

      # A local action stands in this repository and needs no pin.
      case "$value" in
      ./* | '') continue ;;
      esac

      ref=${value##*@}

      if [ "$ref" = "$value" ] || ! [[ "$ref" =~ ^[0-9a-f]{40}$ ]]; then
        report "$file: '$value' hangs on a tag, not on a commit"
      fi
    done < <(grep -E '^[[:space:]]*(-[[:space:]]*)?uses:' "$file")
  done < <(git ls-files '.github/workflows/*.yml')
}

gate_adr() {
  local table=docs/adr/README.md
  local file name

  while IFS= read -r file; do
    name=$(basename "$file")

    if ! grep -qF "]($name)" "$table"; then
      report "$table has no row for $name"
    fi
  done < <(git ls-files 'docs/adr/[0-9]*.md')

  while IFS= read -r name; do
    if [ ! -f "docs/adr/$name" ]; then
      report "$table names docs/adr/$name, which does not exist"
    fi
  done < <(grep -oE '\]\([0-9]{4}-[^)]*\.md\)' "$table" | sed -e 's/^](//' -e 's/)$//' | sort -u)
}

gate_changelog() {
  local version headings_de headings_en

  version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)

  if [ -z "$version" ]; then
    report "no version in Cargo.toml"
    return
  fi

  grep -q "^## \[$version\]" CHANGELOG.md || report "CHANGELOG.md has no section for $version"
  grep -q "^## \[$version\]" CHANGELOG.en.md || report "CHANGELOG.en.md has no section for $version"

  # The section for the next change is optional, because a release turns it into
  # the released one; it must stand in both files or in neither.
  if [ "$(grep -c '^## \[Unveröffentlicht\]' CHANGELOG.md)" -ne \
    "$(grep -c '^## \[Unreleased\]' CHANGELOG.en.md)" ]; then
    report "one changelog has an open section for the next change, the other has not"
  fi

  headings_de=$(grep -E '^## \[' CHANGELOG.md | sed -e 's/^## \[Unveröffentlicht\]/## [Unreleased]/' -e 's/ - .*//')
  headings_en=$(grep -E '^## \[' CHANGELOG.en.md | sed -e 's/ - .*//')

  if [ "$headings_de" != "$headings_en" ]; then
    report "CHANGELOG.md and CHANGELOG.en.md name different versions"
  fi
}

gate_badges() {
  # Called through `bash` so the gate works before the file carries its
  # executable bit.
  bash scripts/badges.sh --check || report "the README badges are out of date; run scripts/badges.sh"
}

gate_doc() {
  RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps || report "rustdoc warns"
}

gate_coverage() {
  local measured tenths snapshot floor

  cargo llvm-cov --workspace --all-features --json --summary-only \
    --output-path target/coverage.json >/dev/null || {
    report "cargo llvm-cov failed"
    return
  }

  measured=$(python3 - <<'PY'
import json
data = json.load(open("target/coverage.json"))
print(data["data"][0]["totals"]["lines"]["percent"])
PY
)

  tenths=$(python3 -c "print(round(float('$measured') * 10))")

  # The first run writes the snapshot; every later run compares against it.
  if [ ! -f docs/coverage.json ]; then
    snapshot=0
  else
    snapshot=$(python3 - <<'PY'
import json
print(round(float(json.load(open("docs/coverage.json"))["lines"]) * 10))
PY
)
  fi
  floor=${MERGEWARDEN_COVERAGE_FLOOR:-$coverage_floor}

  echo "check.sh: line coverage $((tenths / 10)).$((tenths % 10)) % (floor $((floor / 10)).$((floor % 10)) %, snapshot $((snapshot / 10)).$((snapshot % 10)) %)"

  if [ "$tenths" -lt "$floor" ]; then
    report "line coverage is $((tenths / 10)).$((tenths % 10)) %, below the floor of $((floor / 10)).$((floor % 10)) %"
    return
  fi

  if [ $((tenths + coverage_tolerance)) -lt "$snapshot" ]; then
    report "line coverage dropped to $((tenths / 10)).$((tenths % 10)) %, below the snapshot of $((snapshot / 10)).$((snapshot % 10)) %; add tests, or lower docs/coverage.json deliberately"
    return
  fi

  if [ $((tenths - coverage_tolerance)) -gt "$snapshot" ]; then
    printf '{\n  "lines": "%s.%s"\n}\n' "$((tenths / 10))" "$((tenths % 10))" >docs/coverage.json
    echo "check.sh: snapshot raised to $((tenths / 10)).$((tenths % 10)) % in docs/coverage.json; commit it"
  fi
}

gate_msrv() {
  local msrv
  msrv=$(sed -n 's/^rust-version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)

  if [ -z "$msrv" ]; then
    report "no rust-version in Cargo.toml"
    return
  fi

  # `rust-version` names the minor ("1.88"); the toolchain wants the patch.
  case "$msrv" in
  *.*.*) ;;
  *) msrv="$msrv.0" ;;
  esac

  rustup run "$msrv" cargo check --workspace --all-features --locked ||
    report "the workspace does not build with the documented MSRV $msrv"
}

cheap=(spdx typos links secrets machete actions adr changelog badges)
gates=()
spdx_fix=0

for argument in "$@"; do
  case "$argument" in
  --fix) spdx_fix=1 ;;
  *) gates+=("$argument") ;;
  esac
done

if [ ${#gates[@]} -eq 0 ]; then
  gates=("${cheap[@]}")
fi

for gate in "${gates[@]}"; do
  case "$gate" in
  spdx) gate_spdx "$spdx_fix" ;;
  typos | links | secrets | machete | actions | adr | changelog | badges | doc | coverage | msrv)
    "gate_$gate"
    ;;
  *)
    echo "check.sh: unknown gate '$gate'" >&2
    exit 2
    ;;
  esac
done

if [ "$failed" -ne 0 ]; then
  echo "check.sh: failed" >&2
  exit 1
fi

echo "check.sh: ok"
