#!/usr/bin/env python3
"""Writes the layer page of every example project that has layer images: examples/<dir>/layers.md (German) and
layers.en.md. Run it after scripts/layer-previews.sh.

    scripts/layer-pages.py [examples dir]

The title of a project comes from the example tables (README.md, README.en.md, international.md and
international.en.md in the examples folder). A candidate layer that holds only the outline of the map area is
labelled as empty."""

import re
import sys
from pathlib import Path

CANDIDATES = ["terrain", "water", "vegetation", "paths", "barriers", "features", "out-of-bounds"]
TEMPLATES = ["orthophoto", "hillshade", "slope", "relief", "ndsm", "k5"]

TEXT = {
    "de": {
        "file": "layers.md",
        "other": "layers.en.md",
        "other_name": "English",
        "heading": "Die Ebenen",
        "back": "Beispielübersicht",
        "readme": "README.md",
        "intro": (
            "freemapper schreibt zur Karte sieben Kandidaten-Layer und die Vorlagen, nach denen man sie prüft. "
            "Die Layer liegen als `templates/candidates-<Layer>.omap` im Projekt und starten in OOM ausgeblendet. "
            "Ein Klick auf ein Bild zeigt die Ebene in voller Größe."
        ),
        "candidates": "Kandidaten-Layer",
        "templates": "Vorlagen hinter der Karte",
        "empty": "leer",
        "empty_note": "Ein Layer mit dem Zusatz „leer“ hält in diesem Gebiet nur den Umriss des Kartengebiets.",
        "no_templates": "Für dieses Gebiet gibt es keine Raster-Vorlagen im Beispiel.",
        "labels": {
            "terrain": "Gelände",
            "water": "Gewässer",
            "vegetation": "Vegetation",
            "paths": "Wege",
            "barriers": "Barrieren",
            "features": "Objekte",
            "out-of-bounds": "Sperrgebiete",
            "orthophoto": "Orthofoto",
            "hillshade": "Schummerung",
            "slope": "Hangneigung",
            "relief": "Lokales Relief",
            "ndsm": "nDOM",
            "k5": "Karte 1:5000 (Berlin)",
        },
        "tables": ["README.md", "international.md"],
    },
    "en": {
        "file": "layers.en.md",
        "other": "layers.md",
        "other_name": "Deutsch",
        "heading": "The layers",
        "back": "Examples overview",
        "readme": "README.en.md",
        "intro": (
            "Next to the map, freemapper writes seven candidate layers and the templates you check them against. "
            "The layers are `templates/candidates-<layer>.omap` in the project and start hidden in OOM. "
            "A click on an image shows the layer at full size."
        ),
        "candidates": "Candidate layers",
        "templates": "Templates behind the map",
        "empty": "empty",
        "empty_note": "A layer marked “empty” holds only the outline of the map area in this area.",
        "no_templates": "The example has no raster templates for this area.",
        "labels": {
            "terrain": "Terrain",
            "water": "Water",
            "vegetation": "Vegetation",
            "paths": "Paths",
            "barriers": "Barriers",
            "features": "Features",
            "out-of-bounds": "Out of bounds",
            "orthophoto": "Orthophoto",
            "hillshade": "Hillshade",
            "slope": "Slope",
            "relief": "Local relief",
            "ndsm": "nDSM",
            "k5": "Map 1:5000 (Berlin)",
        },
        "tables": ["README.en.md", "international.en.md"],
    },
}

TITLE = re.compile(r"\[(?P<title>[^\]]+)\]\((?P<dir>[^/)]+)/\)")


def titles(examples: Path, files: list[str]) -> dict[str, str]:
    found: dict[str, str] = {}
    for name in files:
        path = examples / name
        if not path.exists():
            continue
        for line in path.read_text(encoding="utf-8").splitlines():
            m = TITLE.search(line)
            if line.startswith("|") and m:
                found.setdefault(m["dir"], m["title"])
    return found


def is_empty(project: Path, layer: str) -> bool:
    """A candidate layer with at most the outline of the map area."""
    omap = project / "templates" / f"candidates-{layer}.omap"
    if not omap.exists():
        return False
    counts = re.findall(r'<objects count="(\d+)"', omap.read_text(encoding="utf-8"))
    return bool(counts) and sum(int(c) for c in counts) <= 1


def cell(layers: str, name: str, label: str) -> str:
    return (
        f'<td width="170" align="center" valign="top"><a href="{layers}/{name}.webp">'
        f'<img src="{layers}/{name}-thumb.webp" width="151" height="151" alt="{label}" loading="lazy"></a>'
        f"<br><sub>{label}</sub></td>"
    )


def table(cells: list[str]) -> str:
    rows = ["<tr>" + "".join(cells[i : i + 4]) + "</tr>" for i in range(0, len(cells), 4)]
    return '<table align="center">\n' + "\n".join(rows) + "\n</table>"


def page(project: Path, lang: str, title: str) -> str:
    t = TEXT[lang]
    have = lambda n: (project / "layers" / f"{n}.webp").exists()
    candidates = []
    any_empty = False
    for layer in CANDIDATES:
        name = f"candidates-{layer}"
        if not have(name):
            continue
        label = t["labels"][layer]
        if is_empty(project, layer):
            label += f" ({t['empty']})"
            any_empty = True
        candidates.append(cell("layers", name, label))
    templates = [cell("layers", n, t["labels"][n]) for n in TEMPLATES if have(n)]
    out = [
        f"# {t['heading']}: {title}",
        "",
        f"[{t['back']}](../{t['readme']}) · [{t['other_name']}]({t['other']})",
        "",
        t["intro"],
        "",
        f"## {t['candidates']}",
        "",
        table(candidates),
    ]
    if any_empty:
        out += ["", t["empty_note"]]
    out += ["", f"## {t['templates']}", ""]
    out += [table(templates)] if templates else [t["no_templates"]]
    return "\n".join(out) + "\n"


def main() -> None:
    examples = Path(sys.argv[1] if len(sys.argv) > 1 else "examples")
    names = {lang: titles(examples, TEXT[lang]["tables"]) for lang in TEXT}
    written = 0
    for project in sorted(p for p in examples.iterdir() if (p / "layers").is_dir()):
        for lang in TEXT:
            title = names[lang].get(project.name, project.name)
            (project / TEXT[lang]["file"]).write_text(page(project, lang, title), encoding="utf-8")
            written += 1
    print(f"layer pages: {written} files for {written // 2} projects")


if __name__ == "__main__":
    main()
