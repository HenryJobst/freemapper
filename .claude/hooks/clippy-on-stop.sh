#!/bin/sh
# Stop hook: block stopping while clippy reports warnings, but never twice in a row
# (stop_hook_active), so an unfixable lint cannot lock the session.
input=$(cat)
case "$input" in
  *'"stop_hook_active":true'* | *'"stop_hook_active": true'*) exit 0 ;;
esac
cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0
[ -f Cargo.toml ] || exit 0
cargo clippy --workspace --all-targets -q -- -D warnings 1>&2 || exit 2
