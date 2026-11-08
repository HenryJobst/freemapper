"""Cuts the first strips out of a Geobasis NRW DGM1 GeoTIFF (1000 x 1000, Float32, LZW, strips of
2 rows) without re-encoding: the first ROWS / 2 strips are copied byte for byte and the image
height is set to ROWS. Every other tag (GeoTIFF keys, GDAL_NODATA) is kept, so the tiepoint (the
north-west corner) stays.

    python make_nrw_dgm1_fixture.py SOURCE TARGET [ROWS]

The fixture:

    python make_nrw_dgm1_fixture.py dgm1_32_354_5642_1_nw_2022.tif nordrhein-westfalen-dgm1.tif 100
"""

import struct
import sys

TYPE_SIZES = {1: 1, 2: 1, 3: 2, 4: 4, 5: 8, 6: 1, 7: 1, 8: 2, 9: 4, 10: 8, 11: 4, 12: 8, 16: 8}


def main(source, target, rows="100"):
    rows = int(rows)
    data = open(source, "rb").read()
    if data[:4] != b"II*\x00":
        sys.exit("expected a little-endian classic TIFF")
    (ifd,) = struct.unpack_from("<I", data, 4)
    (count,) = struct.unpack_from("<H", data, ifd)
    tags = {}
    for i in range(count):
        tag, kind, n, value = struct.unpack_from("<HHII", data, ifd + 2 + 12 * i)
        size = TYPE_SIZES[kind] * n
        raw = data[ifd + 10 + 12 * i : ifd + 10 + 12 * i + size] if size <= 4 else data[value : value + size]
        tags[tag] = (kind, n, raw)

    def longs(tag):
        kind, n, raw = tags[tag]
        return struct.unpack("<%d%s" % (n, "I" if kind == 4 else "H"), raw)

    rows_per_strip = longs(278)[0]
    if rows % rows_per_strip:
        sys.exit("ROWS must be a multiple of the strip height")
    keep = rows // rows_per_strip
    offsets, counts = longs(273)[:keep], longs(279)[:keep]
    tags[257] = (3, 1, struct.pack("<H", rows).ljust(4, b"\x00"))
    tags[273] = (4, keep, bytes(4 * keep))
    tags[279] = (4, keep, struct.pack("<%dI" % keep, *counts))

    entries = sorted(tags)
    ifd_size = 2 + 12 * len(entries) + 4
    values_start = 8 + ifd_size
    extra = sum((len(raw) + 1) // 2 * 2 for kind, n, raw in tags.values() if len(raw) > 4)
    new_offsets, at = [], values_start + extra
    for c in counts:
        new_offsets.append(at)
        at += c
    tags[273] = (4, keep, struct.pack("<%dI" % keep, *new_offsets))

    out = bytearray(b"II*\x00" + struct.pack("<I", 8))
    values = bytearray()
    out += struct.pack("<H", len(entries))
    for tag in entries:
        kind, n, raw = tags[tag]
        if len(raw) <= 4:
            out += struct.pack("<HHI", tag, kind, n) + raw.ljust(4, b"\x00")
        else:
            out += struct.pack("<HHII", tag, kind, n, values_start + len(values))
            values += raw + (b"\x00" if len(raw) % 2 else b"")
    out += struct.pack("<I", 0)
    out += values
    for o, c in zip(offsets, counts):
        out += data[o : o + c]
    open(target, "wb").write(out)


if __name__ == "__main__":
    main(*sys.argv[1:])
