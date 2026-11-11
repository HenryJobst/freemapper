#!/usr/bin/env bash
# Writes the update manifest freemapper-update.json (spec 3.1, ADR 0094) to stdout.
#
#   TAG=v0.12.0 .github/scripts/update-manifest.sh <dist-dir> [<previous-manifest>]
#
# <dist-dir> holds, for each of the five platforms, the bare executable gzip-compressed as
# freemapper-$TAG-<platform>.gz and the SHA-256 of the executable itself as
# freemapper-$TAG-<platform>.exe.sha256 (both written by the Package step of release.yml). The script
# checks that each .gz unpacks to that executable and writes size and SHA-256 of the .gz and the
# SHA-256 of the executable into the manifest. release.yml signs the result with minisign.
#
# The version comes from Cargo.toml (CARGO_TOML names another file; the tests use it). GITHUB_REPOSITORY (default HenryJobst/freemapper) names the
# repository of the asset URLs, RELEASED (default today, UTC) the release date.
#
# min_self_update_from, the oldest version whose updater may install this release, is
#   1. `min_self_update_from` under `[workspace.metadata.update]` in Cargo.toml, if set. Set it by
#      hand when a release changes the update path so that older updaters must not install it;
#   2. otherwise the value in the manifest of the previous release (<previous-manifest>, which
#      release.yml downloads and verifies against assets/update/minisign.pub first), so it stays at
#      the first release with the updater;
#   3. otherwise, when there is no previous manifest (the first release with the updater), the
#      version being built.
# It is never above the version being built.
set -euo pipefail

platforms="linux-x86_64 linux-arm64 macos-arm64 macos-x86_64 windows-x86_64"

fail() {
  echo "update-manifest.sh: $*" >&2
  exit 1
}

[ $# -ge 1 ] && [ $# -le 2 ] || fail "usage: TAG=v<version> $0 <dist-dir> [<previous-manifest>]"
dist=$1
previous=${2:-}
tag=${TAG:-}
repo=${GITHUB_REPOSITORY:-HenryJobst/freemapper}
released=${RELEASED:-$(date -u +%Y-%m-%d)}
root=$(cd "$(dirname "$0")/../.." && pwd)
cargo_toml=${CARGO_TOML:-$root/Cargo.toml}

is_version() {
  printf '%s\n' "$1" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'
}

# Whether version $1 <= version $2 (both x.y.z).
version_le() {
  awk -v a="$1" -v b="$2" 'BEGIN {
    split(a, x, "."); split(b, y, ".")
    for (i = 1; i <= 3; i++) {
      if (x[i] + 0 < y[i] + 0) exit 0
      if (x[i] + 0 > y[i] + 0) exit 1
    }
    exit 0
  }'
}

sha256() {
  if command -v sha256sum > /dev/null; then
    sha256sum | cut -d ' ' -f 1
  else
    shasum -a 256 | cut -d ' ' -f 1
  fi
}

is_hash() {
  printf '%s\n' "$1" | grep -Eq '^[0-9a-f]{64}$'
}

printf '%s\n' "$tag" | grep -Eq '^[A-Za-z0-9._-]+$' || fail "TAG is empty or has unexpected characters: '$tag'"
printf '%s\n' "$repo" | grep -Eq '^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$' || fail "unexpected GITHUB_REPOSITORY '$repo'"
printf '%s\n' "$released" | grep -Eq '^[0-9]{4}-[0-9]{2}-[0-9]{2}$' || fail "RELEASED is not YYYY-MM-DD: '$released'"
[ -d "$dist" ] || fail "no folder $dist"
[ -f "$cargo_toml" ] || fail "no file $cargo_toml"

version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$cargo_toml" | head -n 1)
is_version "$version" || fail "no x.y.z version in Cargo.toml: '$version'"

pinned=$(awk '
  /^\[/ { section = $0; next }
  section == "[workspace.metadata.update]" && $0 ~ /^[ \t]*min_self_update_from[ \t]*=/ {
    value = $0; sub(/^[^=]*=[ \t]*"/, "", value); sub(/".*$/, "", value); print value; exit
  }
' "$cargo_toml")
if [ -n "$pinned" ]; then
  min=$pinned
  origin="Cargo.toml"
elif [ -n "$previous" ] && [ -s "$previous" ]; then
  min=$(sed -n 's/^[[:space:]]*"min_self_update_from":[[:space:]]*"\([^"]*\)".*/\1/p' "$previous" | head -n 1)
  origin="the previous manifest"
else
  min=$version
  origin="the version being built (first release with the updater)"
fi
is_version "$min" || fail "min_self_update_from from $origin is not x.y.z: '$min'"
version_le "$min" "$version" || fail "min_self_update_from $min (from $origin) is above version $version"
echo "update-manifest.sh: version $version, min_self_update_from $min from $origin" >&2

assets=""
for platform in $platforms; do
  gz="$dist/freemapper-$tag-$platform.gz"
  exe_hash_file="$dist/freemapper-$tag-$platform.exe.sha256"
  [ -f "$gz" ] || fail "missing $gz"
  [ -f "$exe_hash_file" ] || fail "missing $exe_hash_file"
  size=$(wc -c < "$gz" | tr -d ' ')
  sha=$(sha256 < "$gz")
  exe_sha=$(cut -d ' ' -f 1 < "$exe_hash_file" | head -n 1)
  is_hash "$exe_sha" || fail "$exe_hash_file holds no SHA-256"
  unpacked=$(gzip -dc < "$gz" | sha256)
  [ "$unpacked" = "$exe_sha" ] || fail "$gz does not unpack to the executable of $exe_hash_file"
  url="https://github.com/$repo/releases/download/$tag/freemapper-$tag-$platform.gz"
  [ -z "$assets" ] || assets="$assets,"
  assets="$assets
    \"$platform\": {
      \"url\": \"$url\",
      \"size\": $size,
      \"sha256\": \"$sha\",
      \"exe_sha256\": \"$exe_sha\"
    }"
done

cat << EOF
{
  "format": 1,
  "app": "freemapper",
  "version": "$version",
  "released": "$released",
  "notes": "https://github.com/$repo/releases/tag/$tag",
  "min_self_update_from": "$min",
  "assets": {$assets
  }
}
EOF
