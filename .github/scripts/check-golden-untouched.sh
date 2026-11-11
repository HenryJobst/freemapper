#!/usr/bin/env bash
# Guard of the golden net (tests/golden/adapters/README.md): checks every commit of a pushed
# range that changes a file under tests/golden/adapters/.
#
#   check-golden-untouched.sh <base> <head>
#
# Such a commit fails the check when
#   - its subject starts with "refactor(": a move never blesses its own change, or
#   - its message has no trailer line "Golden-Bless: <reason>": a feat or fix has to say that it
#     meant to change the output.
# Merge commits are skipped; their parents are checked on their own. A base of all zeros (a new
# branch) or one that is gone means: the commits not on origin/main.
set -euo pipefail

base=${1:?usage: check-golden-untouched.sh <base> <head>}
head=${2:?usage: check-golden-untouched.sh <base> <head>}
golden=tests/golden/adapters/

if [ -z "${base//0/}" ] || ! git cat-file -e "$base^{commit}" 2>/dev/null; then
  base=origin/main
fi

failed=0
for commit in $(git rev-list --no-merges "$base..$head"); do
  # The Markdown files of the folder explain the net; they are no goldens.
  paths=$(git diff-tree --no-commit-id --name-only -r "$commit" -- "$golden" | grep -v '\.md$' || true)
  [ -n "$paths" ] || continue
  subject=$(git log -1 --format=%s "$commit")
  short=$(git rev-parse --short "$commit")
  case $subject in
    "refactor("*)
      echo "::error::$short \"$subject\" is a refactor and changes golden files:"
      echo "$paths" | sed 's/^/  /'
      failed=1
      continue
      ;;
  esac
  if ! git log -1 --format=%B "$commit" | grep -q '^Golden-Bless: .'; then
    echo "::error::$short \"$subject\" changes golden files without a \"Golden-Bless: <reason>\" trailer:"
    echo "$paths" | sed 's/^/  /'
    failed=1
  fi
done

if [ "$failed" -eq 0 ]; then
  echo "golden files: no unexplained change in $base..$head"
fi
exit "$failed"
