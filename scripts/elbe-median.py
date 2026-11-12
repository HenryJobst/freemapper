"""Splits the ground that the boundaries of Lower Saxony and Schleswig-Holstein both hold in the
mouth of the Elbe at the line of equal distance from the two claim lines (ADR 0113).

It was run once, on the boundary files as the agencies' data gave them; on the files it has
written it finds no such ground and stops. Needs Python 3 with shapely 2.

usage: elbe-median.py crates/adapters/de/manifests [--write]
"""
import json, sys
import shapely
from shapely.geometry import shape, Point, LineString, MultiPoint, Polygon
from shapely.ops import unary_union

M = sys.argv[1]
WRITE = "--write" in sys.argv
STEP = 20.0      # metres between the points the line of equal distance is built from
TOLERANCE = 10.0  # metres the simplified line may leave the built one

def path(state): return f"{M}/{state}/boundary.geojson"
def load(state):
    text = open(path(state), encoding="utf-8").read()
    d = json.loads(text)
    return text, d, shape(d["features"][0]["geometry"])

textA, dA, A = load("niedersachsen")
textB, dB, B = load("schleswigholstein")
common = A.intersection(B)
O = max(getattr(common, "geoms", [common]), key=lambda g: g.area)
if not 88e6 < O.area < 90e6:
    print(f"the two boundaries have {common.area / 1e6:.3f} km² in common, the largest piece "
          f"{O.area / 1e6:.3f} km²: the mouth of the Elbe is split already, nothing to do")
    sys.exit(0)
assert not O.interiors
bA, bB = A.boundary, B.boundary

# The two sides of the lens: the claim line of Lower Saxony (north) and of Schleswig-Holstein
# (south). They meet in the two tips.
cs = list(O.exterior.coords)[:-1]
side = []
for i, c in enumerate(cs):
    n = cs[(i + 1) % len(cs)]
    mid = Point((c[0] + n[0]) / 2, (c[1] + n[1]) / 2)
    onA, onB = bA.distance(mid) < 0.01, bB.distance(mid) < 0.01
    assert onA != onB, (c, n)
    side.append("A" if onA else "B")
tips = [cs[(i + 1) % len(cs)] for i in range(len(cs)) if side[i] != side[(i + 1) % len(cs)]]
assert len(tips) == 2, tips
west, east = sorted(tips)
print("tips", west, east)

def dense(kind):
    pts = []
    for i, c in enumerate(cs):
        if side[i] != kind: continue
        n = cs[(i + 1) % len(cs)]
        L = LineString([c, n])
        k = max(1, int(L.length // STEP))
        pts += [L.interpolate(j / k, normalized=True) for j in range(1, k)]
    return pts
north, south = dense("A"), dense("B")
cells = shapely.voronoi_polygons(MultiPoint(north + south), extend_to=O.buffer(2000))
tree = shapely.STRtree(list(cells.geoms))
def cells_of(points):
    idx = tree.query(points, predicate="within")  # point within cell
    return unary_union([cells.geoms[j] for j in idx[1]])
near_north = cells_of(north).intersection(O)   # nearer to the line of Lower Saxony: goes to SH
near_south = cells_of(south).intersection(O)   # nearer to the line of SH: stays with Lower Saxony
assert abs(near_north.area + near_south.area - O.area) < 1.0
line = shapely.line_merge(near_north.boundary.intersection(near_south.boundary))
if line.geom_type != "LineString":
    # In the last metres before a tip the lens is narrower than the step and the cells alternate:
    # the long part is the line, and the tips close it.
    parts = sorted(line.geoms, key=lambda g: -g.length)
    assert sum(g.length for g in parts[1:]) < 100, [g.length for g in parts]
    line = parts[0]
pts = list(line.simplify(TOLERANCE).coords)
if Point(pts[0]).distance(Point(west)) > Point(pts[-1]).distance(Point(west)): pts.reverse()
print("median: built", len(line.coords), "vertices, kept", len(pts), "length m", round(line.length),
      "ends off the tips by m", round(Point(pts[0]).distance(Point(west)), 1), round(Point(pts[-1]).distance(Point(east)), 1))
r = lambda p: [int(round(p[0])), int(round(p[1]))]
median = [r(west)] + [r(p) for p in pts[1:-1]] + [r(east)]
assert all(a != b for a, b in zip(median, median[1:]))

def cut(d, geom_before, name):
    """Replaces the run of ring vertices inside the lens by the median."""
    lens = O.buffer(0.5)
    done = 0
    for poly in d["features"][0]["geometry"]["coordinates"]:
        for ring in poly:
            closed = ring[0] == ring[-1]
            body = ring[:-1] if closed else ring
            inside = [lens.covers(Point(v)) for v in body]
            if not any(inside): continue
            n = len(body)
            assert not all(inside)
            start = next(i for i in range(n) if inside[i] and not inside[i - 1])
            run = 0
            while inside[(start + run) % n]: run += 1
            assert sum(inside) == run, (name, sum(inside), run)  # one run only
            before, after = body[start - 1], body[(start + run) % n]
            m = median if Point(before).distance(Point(west)) < Point(before).distance(Point(east)) else median[::-1]
            # The other vertices keep their places, so the ring starts where it started.
            if start + run <= n:
                new = body[:start] + m + body[start + run:]
            else:
                new = m + body[start + run - n:start]
            ring[:] = new + [new[0]]
            done += 1
            print(name, "ring of", n, "vertices: replaced", run, "by", len(m), "between", before, "and", after)
    assert done == 1, done

cut(dA, A, "niedersachsen")
cut(dB, B, "schleswigholstein")
A2, B2 = shape(dA["features"][0]["geometry"]), shape(dB["features"][0]["geometry"])
assert A2.is_valid and B2.is_valid, (shapely.is_valid_reason(A2), shapely.is_valid_reason(B2))
both = A2.intersection(B2).area
lost = A.union(B).difference(A2.union(B2)).area
gained = A2.union(B2).difference(A.union(B)).area
print("NI km2", round(A.area/1e6, 2), "->", round(A2.area/1e6, 2), " SH km2", round(B.area/1e6, 2), "->", round(B2.area/1e6, 2))
print("shared now m2", round(both), "(before", round(A.intersection(B).area), ") lost m2", round(lost, 1), "gained m2", round(gained, 1))
print("largest share of the smaller:", round(both / min(A2.area, B2.area), 6))
# The tips are rounded to whole metres and closed by straight pieces: a few hundred m² of water.
assert lost < 500 and gained < 500
if WRITE:
    note = "Mouth of the Elbe: the 89 km² that the cadastres of Lower Saxony and Schleswig-Holstein both hold are split at the line of equal distance from the two claims (freemapper, 2026-11-13)"
    for d, text, state in ((dA, textA, "niedersachsen"), (dB, textB, "schleswigholstein")):
        d["features"][0]["properties"]["changed"] = note
        ascii_only = "\\u00a9" in text
        out = json.dumps(d, ensure_ascii=ascii_only, separators=(",", ":"))
        open(path(state), "w", encoding="utf-8").write(out + ("\n" if text.endswith("\n") else ""))
    print("written")
