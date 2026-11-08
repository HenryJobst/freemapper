"""Cuts the north-west corner out of an LVermGeo SH DGM1 GeoTIFF (1000 x 1000, Float32, LZW,
256 x 256 internal tiles) without re-encoding: the first 2 x 2 tiles are copied byte for byte and
the image size is set to SIZE x SIZE, so the right and bottom tiles are partly outside the image,
like the edge tiles of the full file. Every other tag (GeoTIFF keys, GDAL_NODATA) is kept.

    python make_sh_dgm1_fixture.py dgm1_32_565_5949_1_sh_2022.tif schleswig-holstein-dgm1-nw.tif
"""

import struct
import sys

SIZE = 300
TILE_COLUMNS = 2
TYPE_SIZES = {1: 1, 2: 1, 3: 2, 4: 4, 5: 8, 6: 1, 7: 1, 8: 2, 9: 4, 10: 8, 11: 4, 12: 8, 16: 8}


def main(source, target):
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

    width = longs(256)[0]
    tile_width = longs(322)[0]
    across = -(-width // tile_width)
    offsets, counts = longs(324), longs(325)
    keep = [row * across + column for row in range(TILE_COLUMNS) for column in range(TILE_COLUMNS)]
    tags[256] = (4, 1, struct.pack("<I", SIZE))
    tags[257] = (4, 1, struct.pack("<I", SIZE))

    entries = sorted(tags)
    ifd_size = 2 + 12 * len(entries) + 4
    # Layout: header, IFD at offset 8 (as in the source), the out-of-line values, the tiles.
    out = bytearray(b"II*\x00" + struct.pack("<I", 8))
    values = bytearray()
    values_start = 8 + ifd_size
    tile_bytes = b"".join(data[offsets[i] : offsets[i] + counts[i]] for i in keep)
    # Out-of-line sizes do not depend on the offsets, so compute the tile start first.
    tags[324] = (4, len(keep), bytes(4 * len(keep)))
    tags[325] = (4, len(keep), struct.pack("<%dI" % len(keep), *(counts[i] for i in keep)))
    extra = sum((len(raw) + 1) // 2 * 2 for kind, n, raw in tags.values() if len(raw) > 4)
    tiles_start = values_start + extra
    new_offsets, at = [], tiles_start
    for i in keep:
        new_offsets.append(at)
        at += counts[i]
    tags[324] = (4, len(keep), struct.pack("<%dI" % len(keep), *new_offsets))

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
    assert len(out) == tiles_start
    out += tile_bytes
    open(target, "wb").write(out)


if __name__ == "__main__":
    main(*sys.argv[1:])
