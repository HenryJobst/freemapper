#!/usr/bin/env bash
# Small WebP images for the documentation: the pages show them, a click opens the full image.
#
#   scripts/thumbnails.sh            make the thumbnails that are missing or older than their image
#   scripts/thumbnails.sh --force    make all of them again
#
# - examples/*/preview*.webp -> preview*-thumb.webp, 480 px wide (the tables show them at 220 px)
# - examples/berlin-viktoriapark/preview.webp -> preview-medium.webp, 1200 px wide (the README
#   shows it at 760 px)
# - docs/manual/images/*.jpg -> *-thumb.webp, at most 640 px wide
#
# Run it after an example or a manual image was made again. Needs ImageMagick (`magick`).
set -euo pipefail

cd "$(dirname "$0")/.."
force=false
[[ "${1:-}" == "--force" ]] && force=true

command -v magick >/dev/null || { echo "thumbnails.sh: ImageMagick (magick) is needed" >&2; exit 1; }

# make <source> <target> <width>: a WebP of at most <width> px, made only when needed.
make() {
  local source="$1" target="$2" width="$3"
  if ! $force && [[ -f "$target" && "$target" -nt "$source" ]]; then
    return
  fi
  magick "$source" -resize "${width}x>" -strip -quality 72 -define webp:method=6 "$target"
  printf '%s  %s\n' "$(du -h "$target" | cut -f1)" "$target"
}

for source in examples/*/preview*.webp; do
  case "$source" in *-thumb.webp|*-medium.webp) continue ;; esac
  make "$source" "${source%.webp}-thumb.webp" 480
done
make examples/berlin-viktoriapark/preview.webp examples/berlin-viktoriapark/preview-medium.webp 1200

for source in docs/manual/images/*.jpg; do
  make "$source" "${source%.jpg}-thumb.webp" 640
done
