# Saarland: services, licences and what an adapter needs

Research of 2026-11-12 for stage E5 of the region-adapter plan (the first state added on the new scheme). All
requests were made with `curl` and no personal data. The survey of 2026-11-08 (`states-survey.md`) found the
geoportal answering HTTP 503 and listed Saarland as blocked; on 2026-11-12 every service below answered.

Agency: Landesamt für Vermessung, Geoinformation und Landentwicklung (LVGL). CRS: EPSG:25832. ALKIS object ids
start with `DESL`. Geofabrik extract: `europe/germany/saarland`.

## Licence

The catalogue of the LVGL (`https://geoportal.saarland.de/gdi-sl/csw`, ISO records) states for each product used
below: **Datenlizenz Deutschland Namensnennung 2.0** (`dl-de/by-2-0`), source note
`© GeoBasis DE/LVGL-SL (Jahr)`, no access restrictions.

| Product | Record (title) | Licence in the record |
|---|---|---|
| DOP20 2025 | "ATKIS - DOP 2025" (id `f808a27f-16c5-40a2-bd0d-9ff49fdb0a35`) | dl-de/by-2-0 |
| DGM1 2025 | "ATKIS - DGM1 (2025)" and its ATOM download service | dl-de/by-2-0 |
| DOM1 2025 | "ATKIS - DOM1 (2025)" and its ATOM download service | dl-de/by-2-0 |
| ALKIS simplified | "WFS SL ALKIS Vereinfacht - GebaeudeBauwerk / Nutzung - OGC API Features" | dl-de/by-2-0 |

Not open, by their records ("any use beyond viewing is subject to a fee and needs a contract"): DGM1 (2006),
bDOM20, DOM025, the "Premium" DOP years.

**One contradiction, and how the adapter avoids it.** The two INSPIRE WCS services
(`…/gdi-sl/inspireraster/inspirewcsel`, DGM1, and `…/inspirewcsoi`, DOP20) carry in the `Fees` element of their
own capabilities the old text: viewing is free, "any other use, including embedding in other applications
(download), is subject to a fee and requires a contract", with `AccessConstraints`
`INSPIRE_Directive_Article13_1e`. The catalogue records of the same two services say dl-de/by-2-0. The adapter
does not use these two services. It takes the heights from the open-data share the ATOM feeds point to, whose
feed states dl-de/by-2-0, and the orthophoto from the free WMS, whose capabilities say `Fees: None`,
`AccessConstraints: noLimitations`.

## Orthophoto

- WMS `https://geoportal.saarland.de/freewms/dop2025`, layer `sl_dop20_rgb` (also `sl_dop20_cir`, `sl_dop20_ir`),
  EPSG:25832 and 25833, JPEG. `https://geoportal.saarland.de/freewms/dop` (layer `sl_dop`) is the newest year
  under a stable name.
- Flight date: `GetFeatureInfo` on layer `sl_dop_info` of the same WMS. `INFO_FORMAT=text/html` answers a table
  with `Name` (`dop20rgbi_32_348_5454_2_sl_2025`, a 2 km tile), `Titel` (the place) and
  `Aktualisierung` (`10.05.2025`). `text/plain` and GML answer the feature without attributes.
- The INSPIRE WCS `OI.OrthoimageCoverage` is not usable anyway: its grid is EPSG:31466 with axes `y x`; a 200 m
  request in EPSG:25832 came back 1026 by 968 pixels.

## Terrain and surface model

Open-data share of the LVGL (a Nextcloud public share, named in the ATOM feeds of DGM1 2025 and DOM1 2025):
`https://www.shop.lvgl.saarland.de/cloud/index.php/s/NK8ndP55qAqGEZD`. Files are served under
`https://www.shop.lvgl.saarland.de/cloud/public.php/dav/files/NK8ndP55qAqGEZD/<folder>/<file>`; a `PROPFIND`
with `Depth: 1` lists a folder.

| Folder | Content | Size |
|---|---|---|
| `OD_DGM1_2025_tif_LK` | six zips, one per district: `DGM1_tif_<NK,SB,SLS,SPK,MZG,WND>_EPSG-25832_Entstehung-2025.zip` | 0.56 to 1.17 GB each, 5.5 GB |
| `OD_DOM1_2025_tif_LK` | six zips: `DOM1_tif_<district>_EPSG-25832_Entstehung-2025.zip` | 0.63 to 1.30 GB each, 6.2 GB |

Also there (not used): `OD_TrueDOP20_2025_tif_LK` (233 GB), `OD_LIDAR_Punktwolke_2025_laz_LK` (124 GB),
`OD_ALKIS_shp_LK`, `OD_ALKIS_nas_LK`, `OD_Gebäudemodelle_LoD2_gml_LK`, `OD_Hausumringe_shp_SL`.

- The server answers `Range` requests (HTTP 206) and sends `Last-Modified` and `ETag`. A zip has its central
  directory at the end, no zip64: `DGM1_tif_NK…zip` has 311 entries, directory 34.8 KB.
- Members: `DGM1_tif_NK/dgm1_32_<E km>_<N km>_1_SL_2025.tif`, one per 1 km tile, GeoTIFF Float32, 1000 by 1000,
  4,003,508 bytes, deflated to 1.2 to 2.2 MB. DOM1 has the same layout (`dom1_32_…`, to verify when built).
- Which district zip holds a tile is not in its name: the adapter reads the six directories (about 35 KB each)
  and finds the tile. This is the pattern of Hamburg and Hesse (`freemapper_sources::zip_range`).
- The WCS `EL.GridCoverage` would be simpler (1 m with `RESOLUTION=x(1)&RESOLUTION=y(1)`; without it about
  18 m per pixel), but see "Licence", and there is no WCS for the surface model.

## ALKIS (buildings and land use)

- WFS 2.0.0 `https://geoportal.saarland.de/registry/wfs/414` (the geoportal's proxy of an ArcGIS server whose
  own host is not reachable from outside). Types `ALKIS_ALKIS_WFS_ohne_Eig:GebaeudeBauwerk`, `:Nutzung`,
  `:NutzungFlurstueck`, `:Flurstueck`, `:FlurstueckPunkt`, `:Verwaltungseinheit`, `:Katasterbezirk`;
  default CRS EPSG:25832; answers GML 3.2 with `numberMatched`.
- The same as OGC API Features: `https://geoportal.saarland.de/spatial-objects/414/collections/<type>/items`
  (`f=json`, `bbox` in degrees). `limit` must be one of 1, 5, 10, 20, 50, 100, 200, 500, 1000, 2500; another
  value answers HTTP 200 with an HTML sentence. The WFS is the plainer way.
- The attributes are those of the AdV simplified schema with **upper-case names**:
  buildings `OID`, `AKTUALIT` (`2026-04-16Z`), `GEBNUTZBEZ` ("Gebäude"), `FUNKTION` ("Wohngebäude"),
  `GFKZSHH`, `GMDSCHL`, `LAGEBEZTXT`; land use `OID`, `AKTUALIT`, `NUTZART` ("Industrie- und Gewerbefläche"),
  `BEZ` ("Handel und Dienstleistung"). The geometry element is `SHAPE` (a `gml:MultiSurface`), and ArcGIS adds
  `FID`, `Shape_Area`, `Shape_Length`. The reader of freemapper expects the lower-case names of the `ave:`
  schema, so Saarland needs a normaliser that lowers the names.
- The state boundary: type `:Verwaltungseinheit` (to check: the attribute that marks the state), else the
  union of the districts.

## OSM

`https://download.geofabrik.de/europe/germany/saarland-latest.osm.pbf` and its `saarland-updates/state.txt`.

## What the adapter needs beyond a manifest

1. A raster backend (code in `crates/adapters/de`): the WMS orthophoto with its date from the HTML feature
   info, and the two height models from the district zips by range requests.
2. A normaliser preset for the upper-case attribute names (code in `freemapper-sources`, outside
   `crates/adapters`: a finding for the design, since the plan wanted a new state to touch only its folder and
   the registration line).
3. Hosts: `geoportal.saarland.de`, `www.shop.lvgl.saarland.de`.
4. Credit: `© GeoBasis DE/LVGL-SL {year}, dl-de/by-2-0` (the source note the records ask for).
