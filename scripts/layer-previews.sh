#!/usr/bin/env bash
# Images of the layers of one example project: the README and the website show them as squares of 4 cm, a
# click opens the full image.
#
#   scripts/layer-previews.sh examples/berlin-viktoriapark
#
# Writes <project>/layers/<layer>.webp (at most 1600 px wide) and <layer>-thumb.webp (320 x 320 px, centre crop):
# - the seven candidate layers templates/candidates-<layer>.omap: drawn to a PDF by freemapper-print (A3, the
#   scale of the map), cut to the map area (the crop box is made for a sprint map on A3 portrait; check the
#   images when you use another project) and turned into an image by pdftoppm
# - the raster templates templates/{orthophoto,hillshade,slope,relief,ndsm,k5} that the project has
# Needs freemapper-print, pdftoppm (poppler) and ImageMagick (`magick`). `FREEMAPPER_PRINT` names the program.
set -euo pipefail

project="${1:?usage: scripts/layer-previews.sh <project folder>}"
print="${FREEMAPPER_PRINT:-freemapper-print}"
for tool in "$print" pdftoppm magick; do
  command -v "$tool" >/dev/null || { echo "layer-previews.sh: $tool is needed" >&2; exit 1; }
done

out="$project/layers"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$out"

# finish <png or jpg> <name> [bold]: the full image and the square thumbnail. With "bold" the thumbnail gets
# thicker lines (thin vector lines vanish at 3 cm); the full image stays true to the map.
finish() {
  local thicken=()
  [[ "${3:-}" == bold ]] && thicken=(-morphology Erode Disk:2)
  magick "$1" -resize '1600x>' -strip -quality 78 -define webp:method=6 "$out/$2.webp"
  magick "$1" -resize '1600x>' "${thicken[@]}" -resize '320x320^' -gravity center -extent 320x320 -strip -quality 72 -define webp:method=6 "$out/$2-thumb.webp"
  printf '%s  %s\n' "$(du -h "$out/$2.webp" | cut -f1)" "$out/$2.webp"
}

for omap in "$project"/templates/candidates-*.omap; do
  [[ -e "$omap" ]] || continue
  layer="$(basename "$omap" .omap)"
  "$print" render --lang en --paper a3 --out "$work/$layer.pdf" "$omap" >/dev/null
  # 170 dpi; the map area of a sprint map on A3 portrait lies in this box
  pdftoppm -r 170 -png -singlefile -x 185 -y 726 -W 1622 -H 1329 "$work/$layer.pdf" "$work/$layer"
  finish "$work/$layer.png" "$layer" bold
done

for name in orthophoto hillshade slope relief ndsm k5; do
  for ext in jpg png; do
    [[ -f "$project/templates/$name.$ext" ]] && finish "$project/templates/$name.$ext" "$name"
  done
done
