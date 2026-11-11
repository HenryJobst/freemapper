#!/usr/bin/env bash
# Test of .github/scripts/check-golden-untouched.sh in a throwaway repository.
set -euo pipefail

guard=$(cd "$(dirname "$0")/.." && pwd)/.github/scripts/check-golden-untouched.sh
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

git init -q -b main .
git config user.name "golden guard test"
git config user.email "guard@example.invalid"
git config commit.gpgsign false
mkdir -p tests/golden/adapters src
echo 1 > tests/golden/adapters/g1.json
echo 1 > src/code.rs
git add -A
git commit -q -m "chore: start"
start=$(git rev-parse HEAD)
git update-ref refs/remotes/origin/main "$start"

# commit <subject> <file to change> [trailer]
commit() {
  echo "$RANDOM$RANDOM" >> "$2"
  git add -A
  if [ -n "${3:-}" ]; then
    git commit -q -m "$1" -m "$3"
  else
    git commit -q -m "$1"
  fi
}
# expect <pass|fail> <what> <base>
expect() {
  local got=pass
  "$guard" "$3" HEAD > "$tmp/out" 2>&1 || got=fail
  if [ "$got" != "$1" ]; then
    echo "FAIL: $2: expected $1, got $got" >&2
    cat "$tmp/out" >&2
    exit 1
  fi
}

commit "refactor(adapters): move a reader" src/code.rs
expect pass "a refactor that leaves the goldens alone" "$start"

commit "refactor(adapters): move, and explain it" tests/golden/adapters/README.md
expect pass "a refactor that edits the README of the goldens" "$start"

before=$(git rev-parse HEAD)
commit "refactor(adapters): move and change" tests/golden/adapters/g1.json
expect fail "a refactor that edits a golden" "$before"
git reset -q --hard "$before"

commit "refactor(adapters): move and bless" tests/golden/adapters/g1.json "Golden-Bless: it moved"
expect fail "a refactor that edits a golden, even with the trailer" "$before"
git reset -q --hard "$before"

commit "fix(render): a new symbol" tests/golden/adapters/g1.json "Golden-Bless: the symbol is new"
expect pass "a fix that edits a golden with the trailer" "$before"
# The same range seen from a new branch: an all-zero base means the commits not on origin/main.
expect pass "a new branch" 0000000000000000000000000000000000000000
git reset -q --hard "$before"

commit "fix(render): a new symbol" tests/golden/adapters/g1.json
expect fail "a fix that edits a golden without the trailer" "$before"
expect fail "a new branch with an unexplained change" 0000000000000000000000000000000000000000
grep -q "Golden-Bless" "$tmp/out" || { echo "FAIL: the refusal does not name the trailer" >&2; exit 1; }
git reset -q --hard "$before"

# A merge commit is skipped; the commit it brings is checked on its own.
git checkout -q -b side
commit "fix(render): on a side branch" tests/golden/adapters/g1.json "Golden-Bless: side"
git checkout -q main
commit "docs: something else" src/code.rs
git merge -q --no-ff -m "Merge branch 'side'" side
expect pass "a merge of an explained change" "$before"

echo "golden-guard: ok"
