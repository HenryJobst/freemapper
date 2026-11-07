# Test fixtures

| File | Content | Source and licence |
|---|---|---|
| `clara-zetkin-schule.geojson` | School grounds of the Clara-Zetkin-Schule, Strausberg (OSM way 992423119) | © OpenStreetMap contributors, ODbL |
| `nominatim-search.json`, `nominatim-lookup.json` | Nominatim answers for "Clara-Zetkin-Schule Strausberg" (search) and `W992423119` (lookup), 2026-11-07 | © OpenStreetMap contributors, ODbL |
| `strausberg-school.osm.pbf` | OSM data around the school (box 13.8405–13.8470 E, 52.5378–52.5425 N), cut from the Geofabrik extract `brandenburg-latest.osm.pbf` of 2026-11-06 with `make_osm_fixture.py` | © OpenStreetMap contributors, ODbL |

Regenerate the PBF fixture with `python make_osm_fixture.py brandenburg-latest.osm.pbf strausberg-school.osm.pbf`
(needs `pip install osmium`).
