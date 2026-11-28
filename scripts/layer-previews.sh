#!/usr/bin/env bash
# Images of the layers of example projects: the layer pages (examples/<dir>/layers.md) and the README show them
# as squares of 4 cm, a click opens the full image.
#
#   scripts/layer-previews.sh examples/berlin-viktoriapark [more project folders]
#
# Writes <project>/layers/<layer>.webp (at most 1200 px wide) and <layer>-thumb.webp (320 x 320 px, the whole image on white):
# - the seven candidate layers templates/candidates-<layer>.omap: drawn to a PDF by freemapper-print (A3, the
#   scale of the map), turned into an image by pdftoppm and cut to one box for all seven layers: the smallest figure
#   of connected lines a layer draws (the map frame), so that the layers of a project match when you flip through them
# - the raster templates templates/{orthophoto,hillshade,slope,relief,ndsm,k5} that the project has
# The pages come from scripts/layer-pages.py. Needs freemapper-print, pdftoppm (poppler) and ImageMagick
# (`magick`). `FREEMAPPER_PRINT` names the program.
set -euo pipefail

(($# >= 1)) || { echo "usage: scripts/layer-previews.sh <project folder>..." >&2; exit 2; }
print="${FREEMAPPER_PRINT:-freemapper-print}"
for tool in "$print" pdftoppm magick; do
  command -v "$tool" >/dev/null || { echo "layer-previews.sh: $tool is needed" >&2; exit 1; }
done

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# finish <png or jpg> <name> [bold]: the full image and the square thumbnail. With "bold" the thumbnail gets
# thicker lines (thin vector lines vanish at 4 cm); the full image stays true to the map.
finish() {
  local thicken=()
  [[ "${3:-}" == bold ]] && thicken=(-morphology Erode Disk:2)
  magick "$1" -resize '1200x>' -strip -quality 74 -define webp:method=6 "$out/$2.webp"
  magick "$1" -resize '1200x>' ${thicken[@]+"${thicken[@]}"} -resize '320x320' -background white -gravity center -extent 320x320 -strip -quality 74 \
    -define webp:method=6 "$out/$2-thumb.webp"
  printf '%s  %s\n' "$(du -h "$out/$2.webp" | cut -f1)" "$out/$2.webp"
}

for project in "$@"; do
  out="$project/layers"
  rm -rf "${work:?}"/*
  layers=()
  for omap in "$project"/templates/candidates-*.omap; do
    [[ -e "$omap" ]] || continue
    layer="$(basename "$omap" .omap)"
    "$print" render --lang en --paper a3 --out "$work/$layer.pdf" "$omap" >/dev/null
    pdftoppm -r 150 -png -singlefile "$work/$layer.pdf" "$work/$layer"
    layers+=("$layer")
  done
  mkdir -p "$out"
  if ((${#layers[@]})); then
    # one box for all layers: the smallest figure of connected lines any layer draws. Every layer draws the map
    # frame; roads and areas that run through the frame join it in some layers only, so the smallest figure is the
    # frame. The credits at the foot of the page are small separate figures and stay out.
    box=""
    best=0
    for layer in "${layers[@]}"; do
      info="$(magick "$work/$layer.png" -colorspace gray -threshold 97% -negate \
        -define connected-components:verbose=true -connected-components 8 null: |
        awk '/gray\(255\)/ { split($2, a, /[x+]/); if (a[1] * a[2] > m) { m = a[1] * a[2]; g = $2 } } END { print m, g }')"
      area="${info%% *}" geom="${info#* }"
      if [[ -z "$box" ]] || ((area < best)); then best=$area; box="$geom"; fi
    done
    for layer in "${layers[@]}"; do
      magick "$work/$layer.png" -crop "$box" +repage -bordercolor white -border 12 "$work/$layer.png"
      finish "$work/$layer.png" "$layer" bold
    done
  fi
  for name in orthophoto hillshade slope relief ndsm k5; do
    for ext in jpg png; do
      if [[ -f "$project/templates/$name.$ext" ]]; then finish "$project/templates/$name.$ext" "$name"; fi
    done
  done
done
