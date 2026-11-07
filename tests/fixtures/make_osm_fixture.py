"""Cut the OSM test fixture around the Clara-Zetkin-Schule from a Geofabrik extract.

Usage (needs pyosmium, e.g. in a virtual environment: pip install osmium):

    python make_osm_fixture.py brandenburg-latest.osm.pbf strausberg-school.osm.pbf

Keeps every node inside BOX, every way with at least one node inside BOX, and all nodes those ways
reference (so ways crossing the box edge stay complete). Nodes are written before ways, as the PBF
readers expect. Data © OpenStreetMap contributors, ODbL.
"""

import sys

import osmium

# min_lon, min_lat, max_lon, max_lat: the school grounds plus about 50 m.
BOX = (13.8405, 52.5378, 13.8470, 52.5425)


def inside(location):
    lon, lat = location.lon, location.lat
    return BOX[0] <= lon <= BOX[2] and BOX[1] <= lat <= BOX[3]


class Collect(osmium.SimpleHandler):
    def __init__(self):
        super().__init__()
        self.inside = set()
        self.ways = set()
        self.needed = set()

    def node(self, n):
        if n.location.valid() and inside(n.location):
            self.inside.add(n.id)

    def way(self, w):
        refs = [r.ref for r in w.nodes]
        if any(ref in self.inside for ref in refs):
            self.ways.add(w.id)
            self.needed.update(refs)


class Write(osmium.SimpleHandler):
    def __init__(self, collected, writer):
        super().__init__()
        self.c = collected
        self.writer = writer

    def node(self, n):
        if n.id in self.c.inside or n.id in self.c.needed:
            self.writer.add_node(n)

    def way(self, w):
        if w.id in self.c.ways:
            self.writer.add_way(w)


def main(source, target):
    collected = Collect()
    collected.apply_file(source)
    writer = osmium.SimpleWriter(target)
    try:
        Write(collected, writer).apply_file(source)
    finally:
        writer.close()
    print(f"{len(collected.inside)} nodes inside, {len(collected.ways)} ways")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
