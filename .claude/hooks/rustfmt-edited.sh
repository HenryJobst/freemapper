#!/bin/sh
# PostToolUse hook: format the edited file with rustfmt if it is a Rust source file.
file=$(sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
case "$file" in
  *.rs) [ -f "$file" ] && rustfmt --edition 2024 "$file" ;;
esac
exit 0
