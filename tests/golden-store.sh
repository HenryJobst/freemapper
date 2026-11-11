#!/usr/bin/env bash
# Test of `scripts/golden.sh fetch`: it unpacks an archive whose hash matches the lock and
# refuses one whose hash differs. Needs no network: the archive is a throwaway file.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/made/responses"
printf 'an answer\n' > "$tmp/made/responses/abc.bin"
(cd "$tmp/made" && tar -czf "$tmp/store.tar.gz" .)
if command -v sha256sum >/dev/null 2>&1; then
  sha=$(sha256sum < "$tmp/store.tar.gz" | cut -d' ' -f1)
else
  sha=$(shasum -a 256 < "$tmp/store.tar.gz" | cut -d' ' -f1)
fi
# A file URL curl understands on every system (Git Bash: C:/..., not /tmp/...).
if command -v cygpath >/dev/null 2>&1; then
  url="file:///$(cygpath -m "$tmp/store.tar.gz")"
else
  url="file://$tmp/store.tar.gz"
fi

lock_with() {
  printf 'version = 7\nurl = "%s"\nsha256 = "%s"\nbytes = 1\n' "$url" "$1" > "$tmp/store.lock"
}
fetch_into() {
  FREEMAPPER_GOLDEN_LOCK=$tmp/store.lock FREEMAPPER_GOLDEN_STORE=$1 "$root/scripts/golden.sh" fetch
}

lock_with "$sha"
fetch_into "$tmp/good" > /dev/null
[ "$(cat "$tmp/good/responses/abc.bin")" = "an answer" ] || { echo "FAIL: the archive was not unpacked" >&2; exit 1; }
# A second fetch finds the store in place and downloads nothing: the archive may be gone.
mv "$tmp/store.tar.gz" "$tmp/moved.tar.gz"
fetch_into "$tmp/good" > /dev/null || { echo "FAIL: a store in place was fetched again" >&2; exit 1; }
mv "$tmp/moved.tar.gz" "$tmp/store.tar.gz"

# A folder that holds something else is never replaced.
mkdir -p "$tmp/other"
echo "mine" > "$tmp/other/notes.txt"
if fetch_into "$tmp/other" > /dev/null 2> "$tmp/err"; then
  echo "FAIL: a folder that is no fixture store was replaced" >&2
  exit 1
fi
[ "$(cat "$tmp/other/notes.txt")" = "mine" ] || { echo "FAIL: a foreign file is gone" >&2; exit 1; }

lock_with "0000000000000000000000000000000000000000000000000000000000000000"
if fetch_into "$tmp/bad" > /dev/null 2> "$tmp/err"; then
  echo "FAIL: an archive with another hash was accepted" >&2
  exit 1
fi
grep -q "not unpacked" "$tmp/err" || { echo "FAIL: the refusal does not say why" >&2; cat "$tmp/err" >&2; exit 1; }
[ ! -e "$tmp/bad/responses" ] || { echo "FAIL: the refused archive was unpacked" >&2; exit 1; }

echo "golden-store: ok"
