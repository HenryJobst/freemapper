#!/usr/bin/env python3
"""Writes the tiny OSM extract of the Polish test area (a school yard in Łódź), without any library:
a header block and one data block of plain nodes and ways, zlib-compressed, as the PBF format has them.

    python3 -I scripts/make-pl-osm-fixture.py crates/freemapper-cli/tests/fixtures/pl/lodzkie_2026-11-19.osm.pbf

The extract is synthetic: the objects are made up (ids from 1), only the place is real.
"""
import struct
import sys
import zlib
from datetime import datetime, timezone


def varint(n):
    out = bytearray()
    while True:
        b = n & 0x7F
        n >>= 7
        if n:
            out.append(b | 0x80)
        else:
            out.append(b)
            return bytes(out)


def zigzag(n):
    return (n << 1) ^ (n >> 63)


def key(field, wire):
    return varint(field << 3 | wire)


def ld(field, payload):
    return key(field, 2) + varint(len(payload)) + payload


def packed(field, values, signed=False):
    body = b"".join(varint(zigzag(v) if signed else v) for v in values)
    return ld(field, body)


strings = [b""]


def sid(text):
    raw = text.encode()
    if raw not in strings:
        strings.append(raw)
    return strings.index(raw)


def node(id_, lat, lon, tags=()):
    out = key(1, 0) + varint(zigzag(id_))
    if tags:
        out += packed(2, [sid(k) for k, _ in tags]) + packed(3, [sid(v) for _, v in tags])
    out += key(8, 0) + varint(zigzag(round(lat * 1e7)))
    out += key(9, 0) + varint(zigzag(round(lon * 1e7)))
    return out


def way(id_, refs, tags):
    deltas = [refs[0]] + [b - a for a, b in zip(refs, refs[1:])]
    return (
        key(1, 0) + varint(id_)
        + packed(2, [sid(k) for k, _ in tags])
        + packed(3, [sid(v) for _, v in tags])
        + packed(8, deltas, signed=True)
    )


# The school yard: corners of a square of 60 m around 51.7580 N, 19.4535 E.
points = {
    1: (51.7577, 19.4530), 2: (51.7577, 19.4540), 3: (51.7583, 19.4540), 4: (51.7583, 19.4530),
    5: (51.7578, 19.4531), 6: (51.7578, 19.4534), 7: (51.7580, 19.4534), 8: (51.7580, 19.4531),
    9: (51.7570, 19.4520), 10: (51.7590, 19.4550),
    11: (51.7579, 19.4537), 12: (51.7579, 19.4539), 13: (51.7581, 19.4539), 14: (51.7581, 19.4537),
    15: (51.7582, 19.4533),
}
nodes = [node(i, *p, tags=(("natural", "tree"),) if i == 15 else ()) for i, p in points.items()]
ways = [
    way(101, [1, 2, 3, 4, 1], [("leisure", "pitch"), ("sport", "soccer")]),
    way(102, [5, 6, 7, 8, 5], [("building", "school"), ("name", "Szkoła Podstawowa nr 5")]),
    way(103, [9, 10], [("highway", "footway")]),
    way(104, [11, 12, 13, 14, 11], [("natural", "water")]),
]
table = b"".join(ld(1, s) for s in strings)
group = b"".join(ld(1, n) for n in nodes) + b"".join(ld(3, w) for w in ways)
block = ld(1, table) + ld(2, group)


def blob(kind, payload):
    raw = zlib.compress(payload)
    body = key(2, 0) + varint(len(payload)) + ld(3, raw)
    header = ld(1, kind.encode()) + key(3, 0) + varint(len(body))
    return struct.pack(">I", len(header)) + header + body


stamp = int(datetime(2026, 11, 19, 21, 22, tzinfo=timezone.utc).timestamp())
header_block = ld(4, b"OsmSchema-V0.6") + ld(16, b"freemapper test fixture") + key(32, 0) + varint(stamp)
with open(sys.argv[1], "wb") as f:
    f.write(blob("OSMHeader", header_block))
    f.write(blob("OSMData", block))
