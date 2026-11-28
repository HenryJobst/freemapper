# SPDX-License-Identifier: AGPL-3.0-or-later
"""Writes marktacke.gpkg: a synthetic GeoPackage shaped like the Lantmäteriet land cover file
(tables `mark` and `sankmark`, columns objektidentitet, objekttypnr, objekttyp, geom; big-endian geometry headers
without envelope, EPSG:3006). No real data: the squares are made up. Run with `python3 -I`."""
import sqlite3
import struct
import sys

out = sys.argv[1]


def polygon(x0, y0, x1, y1):
    ring = [(x0, y0), (x1, y0), (x1, y1), (x0, y1), (x0, y0)]
    wkb = b"\x00" + struct.pack(">II", 3, 1) + struct.pack(">I", len(ring))
    for x, y in ring:
        wkb += struct.pack(">dd", x, y)
    return b"GP\x00\x00" + struct.pack(">i", 3006) + wkb


rows = [
    ("id-forest", 1, "Barr- och blandskog", polygon(385050, 6177550, 385100, 6177600)),
    ("id-lake-far", 2, "Sjö", polygon(400000, 6190000, 400100, 6190100)),
    ("id-open", 3, "Öppen mark", polygon(385010, 6177510, 385020, 6177520)),
    ("id-square-across", 4, "Torg", polygon(385150, 6177650, 385300, 6177750)),
    ("id-unknown-1", 5, "Okänd typ", polygon(385060, 6177510, 385070, 6177520)),
    ("id-unknown-2", 5, "Okänd typ", polygon(385080, 6177510, 385090, 6177520)),
    ("id-low-built", 6, "Låg bebyggelse", polygon(385120, 6177520, 385140, 6177540)),
]
db = sqlite3.connect(out)
db.execute("PRAGMA application_id = 1196444487")
db.execute(
    "CREATE TABLE mark (fid INTEGER PRIMARY KEY AUTOINCREMENT, objektidentitet TEXT, "
    "objekttypnr INTEGER, objekttyp TEXT, geom BLOB)"
)
db.executemany(
    "INSERT INTO mark (objektidentitet, objekttypnr, objekttyp, geom) VALUES (?, ?, ?, ?)", rows
)
# The wetland of the real files lies in a table of its own, on top of the land cover.
wet = [
    ("id-wet", 20, "Sankmark, våt", polygon(385100, 6177600, 385140, 6177640)),
    ("id-firm", 21, "Sankmark, fast", polygon(385150, 6177600, 385180, 6177620)),
]
db.execute(
    "CREATE TABLE sankmark (fid INTEGER PRIMARY KEY AUTOINCREMENT, objektidentitet TEXT, "
    "objekttypnr INTEGER, objekttyp TEXT, geom BLOB)"
)
db.executemany(
    "INSERT INTO sankmark (objektidentitet, objekttypnr, objekttyp, geom) VALUES (?, ?, ?, ?)",
    wet,
)
db.commit()
db.execute("VACUUM")
db.close()
