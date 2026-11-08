# Test fixtures

| File | Content | Source and licence |
|---|---|---|
| `clara-zetkin-schule.geojson` | School grounds of the Clara-Zetkin-Schule, Strausberg (OSM way 992423119) | © OpenStreetMap contributors, ODbL |
| `nominatim-search.json`, `nominatim-lookup.json` | Nominatim answers for "Clara-Zetkin-Schule Strausberg" (search) and `W992423119` (lookup), 2026-11-07 | © OpenStreetMap contributors, ODbL |
| `strausberg-school.osm.pbf` | OSM data around the school (box 13.8405–13.8470 E, 52.5378–52.5425 N), cut from the Geofabrik extract `brandenburg-latest.osm.pbf` of 2026-11-06 with `make_osm_fixture.py` | © OpenStreetMap contributors, ODbL |
| `hessen-dc-*.json` | HVBG Downloadcenter REST answers, trimmed to the fields freemapper reads: the DGM1 product tree (8 of 27 districts), the DGM1 listings of Frankfurt am Main and Wiesbaden, and the two pages of the DOP20 listing of Marburg-Biedenkopf (4 of 20 entries on page 1), 2026-11-07 | © HVBG, dl-de/zero-2-0 |
| `schleswig-holstein-dgm1-nw.tif` | The north-west 300 x 300 m of the LVermGeo SH DGM1 tile `dgm1_32_565_5949_1_sh_2022.tif` (INSPIRE ATOM, 2026-11-08): its first 2 x 2 LZW tiles of 256 px copied byte for byte, all tags kept, cut with `make_sh_dgm1_fixture.py` | © GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert) |
| `schleswig-holstein-dgm1-lzw-ne.tif` | The north-east 488 x 300 m of the LVermGeo SH DGM1 tile `dgm1_32_611_5970_1_sh_2022.tif` (Lübeck, INSPIRE ATOM, 2026-11-08), its tile columns 2–3 of rows 0–1 copied byte for byte with `make_sh_dgm1_fixture.py`; the upper right tile is one the `tiff` crate 0.11.3 refuses ("no lzw end code found") | © GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert) |
| `mecklenburg-vorpommern-wcs-dgm1.tif`, `mecklenburg-vorpommern-wcs-dom1.tif` | LAiV M-V WCS answers (`dgm_wcs` `mv_dgm`, `dom_wcs` `mv_dom1`, `FORMAT=image/tiff&GEOTIFF:COMPRESSION=LZW`) for E 312100–312200, N 5996250–5996350 (EPSG:25833) at the Innerstädtisches Gymnasium Rostock, as served, 2026-11-08 | ©GeoBasis-DE/MV/CC BY 4.0 |
| `mecklenburg-vorpommern-overview-dgm1.json`, `mecklenburg-vorpommern-overview-dom.json` | LAiV M-V Downloadportal `overview.php` answers (`type[]=dgm1`, `type[]=dom`) for the school tile `33_312_5996`, 2026-11-08 | ©GeoBasis-DE/MV/CC BY 4.0 |
| `mecklenburg-vorpommern-overview-dop20.json` | LAiV M-V Downloadportal `overview.php` answer (`type[]=dop20`) around the school, with the flight date of the tiles, 2026-11-08 | ©GeoBasis-DE/MV/CC BY 4.0 |
| `mecklenburg-vorpommern-buildings.json`, `mecklenburg-vorpommern-land-use.json` | LAiV M-V ALKIS simplified (`alkis_vereinfacht`) buildings and land use parts (`NutzungFlurstueck`) from the OGC API Features at the school, 30 features each, 2026-11-08 | ©GeoBasis-DE/MV/CC BY 4.0 |
| `mecklenburg-vorpommern-wfs-buildings.gml`, `mecklenburg-vorpommern-wfs-land-use.gml` | The same objects from the classic LAiV M-V WFS 2.0 (the fallback of the OGC API), 25 buildings and 30 land use parts, 2026-11-08 | ©GeoBasis-DE/MV/CC BY 4.0 |

Regenerate the PBF fixture with `python make_osm_fixture.py brandenburg-latest.osm.pbf strausberg-school.osm.pbf`
(needs `pip install osmium`).

Cut the DGM1 fixtures with `python make_sh_dgm1_fixture.py dgm1_32_565_5949_1_sh_2022.tif schleswig-holstein-dgm1-nw.tif`
and `python make_sh_dgm1_fixture.py dgm1_32_611_5970_1_sh_2022.tif schleswig-holstein-dgm1-lzw-ne.tif 2 0 488 300`.
