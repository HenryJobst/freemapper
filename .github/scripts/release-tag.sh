#!/usr/bin/env bash
# Names the release for release.yml and checks the name against the version in Cargo.toml.
#
#   DRY_RUN=true|false TAG_INPUT=<input tag> .github/scripts/release-tag.sh
#
# A real release takes the tag input of a dispatch, else the pushed tag (GITHUB_REF_NAME), and it must
# be v<version>. A dry run is always named v<version>, so the manifest version and the URLs agree; a
# tag input there is ignored, with a warning if it differs. CARGO_TOML names another Cargo.toml (the
# tests use it).
#
# Prints the tag and writes TAG=<tag> to $GITHUB_ENV and tag=<tag> to $GITHUB_OUTPUT when they are set.
set -euo pipefail

cargo_toml=${CARGO_TOML:-Cargo.toml}
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$cargo_toml" | head -n 1)
if ! printf '%s\n' "$version" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'; then
  echo "release-tag.sh: no x.y.z version in $cargo_toml: '$version'" >&2
  exit 1
fi

input=${TAG_INPUT:-}
if [ "${DRY_RUN:-false}" = true ]; then
  tag="v$version"
  if [ -n "$input" ] && [ "$input" != "$tag" ]; then
    echo "::warning::Dry run: the tag input $input is ignored; the files are named $tag after Cargo.toml."
  fi
else
  tag=${input:-${GITHUB_REF_NAME:-}}
fi

if [ "$tag" != "v$version" ]; then
  echo "Tag $tag does not match version $version in Cargo.toml" >&2
  exit 1
fi

if [ -n "${GITHUB_ENV:-}" ]; then echo "TAG=$tag" >> "$GITHUB_ENV"; fi
if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "tag=$tag" >> "$GITHUB_OUTPUT"; fi
echo "$tag"
