#!/usr/bin/env python3
"""Writes the synthetic GeoPackages of the tests of `gpkg.rs` next to this script.

    python3 make_fixtures.py

`small.gpkg`: the tables `mark` (polygons, big-endian header and WKB like Lantmäteriet's Marktäcke),
`markkantlinje` (lines) and `points` (little endian, with an envelope), page size 512.
`deep.gpkg`: a table of 700 rows (an interior level in the table b-tree) and one row with a text of
5000 bytes that runs through the alphabet (an overflow chain), page size 512.
No data of any real source is in them.
"""
import os
import sqlite3
import struct

here = os.path.dirname(os.path.abspath(__file__))


def header(srs, big=True, envelope=None, empty=False):
    flags = (0 if big else 1) | ((1 if envelope else 0) << 1) | (int(empty) << 4)
    out = b"GP" + bytes([0, flags]) + struct.pack(">i" if big else "<i", srs)
    if envelope:
        out += struct.pack(">4d" if big else "<4d", *envelope)
    return out


def wkb_polygon(rings, big=True):
    e = ">" if big else "<"
    out = bytes([0 if big else 1]) + struct.pack(e + "II", 3, len(rings))
    for ring in rings:
        out += struct.pack(e + "I", len(ring))
        for x, y in ring:
            out += struct.pack(e + "2d", x, y)
    return out


def wkb_multi(kind, parts, big=True):
    e = ">" if big else "<"
    return bytes([0 if big else 1]) + struct.pack(e + "II", kind, len(parts)) + b"".join(parts)


def wkb_line(points, big=True):
    e = ">" if big else "<"
    out = bytes([0 if big else 1]) + struct.pack(e + "II", 2, len(points))
    return out + b"".join(struct.pack(e + "2d", x, y) for x, y in points)


def wkb_point(x, y, big=True):
    e = ">" if big else "<"
    return bytes([0 if big else 1]) + struct.pack(e + "I2d", 1, x, y)


def create(path, page_size):
    if os.path.exists(path):
        os.remove(path)
    db = sqlite3.connect(path)
    db.execute(f"PRAGMA page_size={page_size}")
    db.execute("PRAGMA application_id=1196444487")
    db.execute(
        "CREATE TABLE gpkg_geometry_columns (table_name TEXT NOT NULL, column_name TEXT NOT NULL, "
        "geometry_type_name TEXT NOT NULL, srs_id INTEGER NOT NULL, z TINYINT NOT NULL, m TINYINT NOT NULL)"
    )
    return db


def small():
    db = create(os.path.join(here, "small.gpkg"), 512)
    db.execute(
        'CREATE TABLE "mark" (fid INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, geom POLYGON, '
        "objekttyp TEXT, areal REAL, antal INTEGER, notering BLOB)"
    )
    db.execute(
        "CREATE TABLE markkantlinje (fid INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, geom LINESTRING, typ TEXT)"
    )
    db.execute("CREATE TABLE points (id INTEGER PRIMARY KEY, geom POINT, namn TEXT)")
    outer = [(0, 0), (10, 0), (10, 10), (0, 10), (0, 0)]
    hole = [(2, 2), (4, 2), (4, 4), (2, 4), (2, 2)]
    square = [(20, 20), (21, 20), (21, 21), (20, 21), (20, 20)]
    rows = [
        # a polygon with a hole
        (header(3006) + wkb_polygon([outer, hole]), "Skog", 99.5, 7, b"\x00\x01"),
        # a multipolygon of two squares
        (
            header(3006)
            + wkb_multi(6, [wkb_polygon([square]), wkb_polygon([[(x + 5, y) for x, y in square]])]),
            "Öppen mark",
            2.0,
            None,
            None,
        ),
        # an empty geometry
        (header(3006, empty=True), "Tom", None, -3, None),
        # NULL geometry
        (None, "Utan form", 0.0, 1 << 40, None),
    ]
    db.executemany("INSERT INTO mark (geom, objekttyp, areal, antal, notering) VALUES (?,?,?,?,?)", rows)
    db.execute(
        "INSERT INTO markkantlinje (geom, typ) VALUES (?, ?)",
        (header(3006) + wkb_multi(5, [wkb_line([(0, 0), (5, 5), (9, 1)]), wkb_line([(1, 1), (2, 2)])]), "Kant"),
    )
    db.execute(
        "INSERT INTO points (geom, namn) VALUES (?, ?)",
        (header(3006, big=False, envelope=(7.0, 7.0, 8.0, 8.0)) + wkb_point(7.5, 7.5, big=False), "Punkt"),
    )
    db.executemany(
        "INSERT INTO gpkg_geometry_columns VALUES (?,?,?,?,?,?)",
        [("mark", "geom", "POLYGON", 3006, 0, 0), ("markkantlinje", "geom", "LINESTRING", 3006, 0, 0), ("points", "geom", "POINT", 3006, 0, 0)],
    )
    db.commit()
    db.close()


def deep():
    db = create(os.path.join(here, "deep.gpkg"), 512)
    db.execute("CREATE TABLE ytor (fid INTEGER PRIMARY KEY, geom BLOB, namn TEXT)")
    square = [(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)]
    for i in range(1, 701):
        text = "".join(chr(97 + k % 26) for k in range(5000)) if i == 350 else f"yta {i}"
        db.execute("INSERT INTO ytor VALUES (?,?,?)", (i, header(3006) + wkb_polygon([[(x + i, y) for x, y in square]]), text))
    db.execute("INSERT INTO gpkg_geometry_columns VALUES ('ytor','geom','POLYGON',3006,0,0)")
    db.commit()
    db.close()


small()
deep()
