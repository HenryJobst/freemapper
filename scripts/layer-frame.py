#!/usr/bin/env python3
"""Helper of scripts/layer-previews.sh.

freemapper-print centres the content of a map on the page. The frame of the candidate layers lies around the origin of
the map, so it sits at another place on the page of every layer, and not on the page at all when one object reaches
far out of it. `anchor` adds two point objects, one on each side of the origin and as far away as the most distant
object, so that the content is centred on the origin: on every page, whatever it holds, the frame is in the middle.

    layer-frame.py anchor IN.omap OUT.omap
"""

import re
import sys


def anchor(src: str, out: str) -> None:
    text = open(src, encoding="utf-8").read()
    point = re.search(r'<symbol type="1" id="(\d+)"', text)
    if not point:
        sys.exit("no point symbol in " + src)
    far = 0
    for coords in re.findall(r'<coords count="\d+">(.*?)</coords>', text, re.S):
        for x, y in re.findall(r"(-?\d+) (-?\d+)", coords):
            far = max(far, abs(int(x)), abs(int(y)))
    far += 20000  # 20 mm of paper more, so that the anchors lie outside of what is cut out
    objects = "".join(
        '<object type="0" symbol="%s"><coords count="1">%d %d;</coords></object>' % (point.group(1), x, y)
        for x, y in ((far, far), (-far, -far))
    )
    text = re.sub(
        r'<objects count="(\d+)">',
        lambda g: '<objects count="%d">%s' % (int(g.group(1)) + 2, objects),
        text,
        count=1,
    )
    open(out, "w", encoding="utf-8").write(text)


if __name__ == "__main__":
    {"anchor": anchor}[sys.argv[1]](*sys.argv[2:])
