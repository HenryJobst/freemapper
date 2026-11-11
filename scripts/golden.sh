#!/usr/bin/env bash
# The fixture store of the golden net: the recorded answers of the data services, too large
# for the repository. tests/golden/adapters/README.md has the details.
#
#   scripts/golden.sh fetch         download the store named in store.lock, check it, unpack it
#   scripts/golden.sh test          fetch, then run all five layers of the net
#   scripts/golden.sh record <id>   record one state from the network and bless its goldens
#   scripts/golden.sh pack          pack the store folder and write its hash into store.lock
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
lock=${FREEMAPPER_GOLDEN_LOCK:-$root/tests/golden/adapters/store.lock}

# The value of a key of store.lock, without its quotes.
field() {
  sed -n "s/^$1 = \"\{0,1\}\([^\"]*\)\"\{0,1\}[[:space:]]*\$/\1/p" "$lock" | head -n 1
}

version=$(field version)
[ -n "$version" ] || { echo "$lock names no version" >&2; exit 1; }
store=${FREEMAPPER_GOLDEN_STORE:-$root/target/golden-adapters/$version}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum < "$1" | cut -d' ' -f1
  else
    shasum -a 256 < "$1" | cut -d' ' -f1
  fi
}

fetch() {
  local url sha got tmp
  url=$(field url)
  sha=$(field sha256)
  if [ -f "$store/.sha256" ] && [ "$(cat "$store/.sha256")" = "$sha" ]; then
    echo "fixture store $version is in place: $store"
    return
  fi
  if [ -z "$url" ] || [ -z "$sha" ]; then
    echo "$lock names no archive: version $version is not published yet" >&2
    exit 1
  fi
  tmp=$(mktemp -d)
  # No token and no header of our own: the archive is public.
  curl -fsSL --retry 3 -o "$tmp/store.tar.gz" "$url"
  got=$(sha256_of "$tmp/store.tar.gz")
  if [ "$got" != "$sha" ]; then
    rm -rf "$tmp"
    echo "the archive of $url has the SHA-256 $got, store.lock says $sha; not unpacked" >&2
    exit 1
  fi
  rm -rf "$store"
  mkdir -p "$store"
  tar -xzf "$tmp/store.tar.gz" -C "$store"
  rm -rf "$tmp"
  printf '%s\n' "$sha" > "$store/.sha256"
  echo "fixture store $version unpacked: $store"
}

run_tests() {
  fetch
  cd "$root"
  cargo test -p freemapper --test adapters_golden --locked -- --include-ignored
}

record() {
  local id=${1:?usage: scripts/golden.sh record <id>}
  cd "$root"
  mkdir -p "$store"
  FREEMAPPER_GOLDEN_RECORD=$id cargo test -p freemapper --test adapters_golden --locked \
    record_state -- --ignored --exact --nocapture
  # The traces and hashes of this state, from a replay of what was just recorded.
  FREEMAPPER_BLESS=1 cargo test -p freemapper --test adapters_golden --locked \
    "::$id" -- --ignored
}

pack() {
  local archive sha bytes
  [ -d "$store/responses" ] || { echo "nothing is recorded in $store" >&2; exit 1; }
  archive=$root/target/golden-adapters/golden-adapters-$version.tar.gz
  # Answers no index names any more (a state recorded again) do not go into the archive.
  for body in "$store"/responses/*.bin; do
    name=$(basename "$body" .bin)
    grep -qs "^body = \"$name\"" "$root"/tests/golden/adapters/*/index.toml || rm -f "$body"
  done
  (cd "$store" && COPYFILE_DISABLE=1 tar -czf "$archive" --exclude .sha256 .)
  sha=$(sha256_of "$archive")
  bytes=$(wc -c < "$archive" | tr -d ' ')
  cat > "$lock" <<LOCK
# The fixture store this tree needs: scripts/golden.sh fetch (tests/golden/adapters/README.md).
version = $version
url = "https://github.com/HenryJobst/freemapper/releases/download/golden-adapters-$version/golden-adapters-$version.tar.gz"
sha256 = "$sha"
bytes = $bytes
LOCK
  printf '%s\n' "$sha" > "$store/.sha256"
  echo "packed $archive ($bytes bytes, SHA-256 $sha)"
  echo "publish it once, and never change it afterwards:"
  echo "  gh release create golden-adapters-$version --prerelease --latest=false \\"
  echo "    --title \"Fixture store $version of the golden net\" --notes-file tests/golden/adapters/STORE-NOTES.md \\"
  echo "    $archive"
}

case ${1:-} in
  fetch) fetch ;;
  test) run_tests ;;
  record) record "${2:-}" ;;
  pack) pack ;;
  *) sed -n '2,9p' "$0" >&2; exit 2 ;;
esac
