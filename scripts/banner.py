#!/usr/bin/env python3
"""Writes the README banners docs/banner/banner.de.svg and banner.en.svg.

    python3 scripts/banner.py

The word FREEMAPPER stands in a pixel font on a field of contour lines made of two hills, in the
orange of the control flag. The letters are drawn as rectangles, so the banner looks the same
without a font.
"""
import math
import pathlib

ORANGE = "#FF6A13"
CELL_W, CELL_H = 12, 22
WIDTH, HEIGHT = 58, 7
TAGLINES = {
    "de": "Orientierungslaufkarten aus offenen Geodaten",
    "en": "Orienteering maps from open geodata",
}

FONT = {
    "F": ["████", "█   ", "███ ", "█   ", "█   "],
    "R": ["███ ", "█  █", "███ ", "█ █ ", "█  █"],
    "E": ["████", "█   ", "███ ", "█   ", "████"],
    "M": ["█   █", "██ ██", "█ █ █", "█   █", "█   █"],
    "A": [" ██ ", "█  █", "████", "█  █", "█  █"],
    "P": ["███ ", "█  █", "███ ", "█   ", "█   "],
}


def word_pixels(word):
    rows = ["" for _ in range(5)]
    for letter in word:
        glyph = FONT[letter]
        width = max(len(line) for line in glyph)
        for i in range(5):
            rows[i] += glyph[i].ljust(width) + " "
    width = max(len(row) for row in rows)
    return [row.ljust(width) for row in rows] + [" " * width]


def half_blocks(pixels):
    lines = []
    for i in range(0, 6, 2):
        line = ""
        for upper, lower in zip(pixels[i], pixels[i + 1]):
            line += {
                (True, True): "█",
                (True, False): "▀",
                (False, True): "▄",
                (False, False): " ",
            }[(upper == "█", lower == "█")]
        lines.append(line.rstrip())
    return lines


def height(x, y):
    first = math.exp(-(((x - 10) / 16) ** 2 + ((y - 2) / 3) ** 2))
    second = 0.9 * math.exp(-(((x - 48) / 14) ** 2 + ((y - 5) / 3.5) ** 2))
    return first + second


def grid():
    cells = [[" "] * WIDTH for _ in range(HEIGHT)]
    level = lambda x, y: int(height(x, y) * 4)
    for y in range(HEIGHT):
        for x in range(WIDTH):
            here = level(x, y)
            if (x + 1 < WIDTH and level(x + 1, y) != here) or (y + 1 < HEIGHT and level(x, y + 1) != here):
                cells[y][x] = "·"
    text = half_blocks(word_pixels("FREEMAPPER"))
    text_width = max(len(line) for line in text)
    left, top = (WIDTH - text_width) // 2, (HEIGHT - 3) // 2
    for i, line in enumerate(text):
        for x in range(left - 2, left + text_width + 2):
            cells[top + i][x] = " "
        for j, char in enumerate(line):
            if char != " ":
                cells[top + i][left + j] = char
    return cells


def svg(tagline, cells):
    width = WIDTH * CELL_W + 2 * CELL_W
    total = HEIGHT * CELL_H + 2 * CELL_H + 30
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {total}" width="{width * 2 // 3}" '
        f'role="img" aria-labelledby="t"><title id="t">freemapper – {tagline}</title>'
    ]
    for y, row in enumerate(cells):
        for x, char in enumerate(row):
            px, py = CELL_W + x * CELL_W, CELL_H + y * CELL_H
            if char == "█":
                out.append(f'<rect x="{px}" y="{py}" width="{CELL_W}" height="{CELL_H}" fill="{ORANGE}"/>')
            elif char == "▀":
                out.append(f'<rect x="{px}" y="{py}" width="{CELL_W}" height="{CELL_H // 2}" fill="{ORANGE}"/>')
            elif char == "▄":
                out.append(f'<rect x="{px}" y="{py + CELL_H // 2}" width="{CELL_W}" height="{CELL_H // 2}" fill="{ORANGE}"/>')
            elif char == "·":
                out.append(f'<circle cx="{px + CELL_W // 2}" cy="{py + CELL_H // 2}" r="1.8" fill="{ORANGE}" fill-opacity=".45"/>')
    out.append(
        f'<text x="{width // 2}" y="{total - 12}" text-anchor="middle" '
        'font-family="system-ui,Segoe UI,Helvetica,Arial,sans-serif" font-size="15" letter-spacing="1" '
        f'fill="{ORANGE}">{tagline}</text>'
    )
    out.append("</svg>")
    return "\n".join(out) + "\n"


if __name__ == "__main__":
    folder = pathlib.Path(__file__).resolve().parent.parent / "docs" / "banner"
    folder.mkdir(parents=True, exist_ok=True)
    cells = grid()
    for language, tagline in TAGLINES.items():
        (folder / f"banner.{language}.svg").write_text(svg(tagline, cells), encoding="utf-8")
