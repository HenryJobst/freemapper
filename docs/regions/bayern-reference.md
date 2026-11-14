# Bavaria: services, licences and what the adapter uses

Research of 2026-11-13 (78 requests with `curl`, freemapper's own `User-Agent`, no personal data; `HEAD` or a
range for everything above 5 MB). It replaces the row of the survey of 2026-11-08 (`states-survey.md`) and the
note of 2026-11-12. Both said "no WFS"; that is wrong for ATKIS (below), and right for the cadastre.

Agency: Bayerische Vermessungsverwaltung, its state office is the Landesamt für Digitalisierung, Breitband und
Vermessung (LDBV). Portal: `https://geodaten.bayern.de/opengeodata/`. The product list the portal itself reads
is `https://geodaten.bayern.de/opengeodata/json/opengeodata_produkte.json` (35 products on 2026-11-06, each with
its delivery kinds). CRS of every product: EPSG:25832, also east of 12° E (Passau, Regensburg). ALKIS object ids
start with `DEBY`.

What makes Bavaria different from the fourteen states before it: the cadastre has **no service**. Buildings and
land use come as files only, so the adapter downloads files and cuts the map area out of them.

## Licence

Everything on the portal is **CC BY 4.0**; no request, no agreement. The FAQ
(`https://www.geodaten.bayern.de/odd/m/3/html/faq.html`) gives:

- the source note that is required: `Bayerische Vermessungsverwaltung – www.geodaten.bayern.de`
- a "possible source note with changes":
  `Geobasisdaten: Bayerische Vermessungsverwaltung – www.geodaten.bayern.de (Daten verändert), Lizenz: CC BY 4.0`

No year in either form. The WMS repeats it: `Fees` "kostenfrei", `AccessConstraints` "CC BY 4.0 vgl.
https://creativecommons.org/licenses/by/4.0/deed.de". freemapper writes the second form, as it changes the data.

Still charged, by the FAQ: the vector Flurkarte and the Hauskoordinaten. The adapter does not use them.

No `robots.txt` on any of the hosts, no rate limit or maintenance window stated in the FAQ, the terms, the WFS
page or the newsletters of 2026. That does not prove that none is enforced.

## Hosts

| Host | What it serves |
|---|---|
| `geoservices.bayern.de` | WMS (orthophoto, land use picture), the ATKIS WFS, `poly2metalink` |
| `download1.bayernwolke.de` | the tiles: DGM1, DOM20, DOP20/40, LoD2, the land use zips (nginx) |
| `download2.bayernwolke.de` | mirror of `download1`, named second in every Metalink |
| `geodaten.bayern.de` | product list, index KML and Metalink files, Hausumringe, LAZ point clouds, the state-wide files |

All data hosts send `Accept-Ranges: bytes` (a 206 was seen on each). Data URLs do not redirect; only http is sent
to https by a 302.

## Orthophoto

WMS `https://geoservices.bayern.de/od/wms/dop/v1/dop20`, WMS 1.3.0.

- Layers `by_dop20c` (colour), `by_dop20g`, `by_dop20cir`, `by_dop20_info` (the flights).
- CRS 25832, 25833, 31468, 4258, 4326, 3857, 5678. MaxWidth and MaxHeight 6500 (the abstract says 6000).
- Formats jpeg, png, tiff.
- `GetMap` of 100 m at `691000,5334500,691100,5334600` (Munich), 500 px, EPSG:25832: 200, `image/jpeg`, 50 692
  bytes. The same in EPSG:25833 at Passau: a real picture too.
- Flight date: `GetFeatureInfo` on `by_dop20_info` with `INFO_FORMAT=text/plain` answers
  `nummer = '124028'`, `unr = '0'`, `ua = '24.08.2024'`. The field is `ua`, the form `DD.MM.YYYY`.

A second WMS `…/od/wms/dop/v1/dop40` (`by_dop40c`, 40 cm) exists; the adapter does not need it. Half the state is
flown each year. Tiles (`…/a/dop20/data/32<E>_<N>.tif`, 54–73 MB per km²) are not used either.

## Terrain: DGM1

`https://download1.bayernwolke.de/a/dgm/dgm1/<E>_<N>.tif`, `<E>` and `<N>` the south-west corner in km of
EPSG:25832 (`691_5334`). One tile is 1 km, 1000 × 1000 px, Float32, LZW in strips of two rows, predictor 1,
pixel is area, nodata `-9999`, 2.6–2.8 MB. The file has no vertical CRS key (the state's heights are DHHN2016)
and no date.

- East of 12° E the naming stays zone 32: `828_5389.tif` (Passau) answers 200.
- A tile that does not exist answers 404 with a 42-byte HTML body (`400_5500.tif`).
- The tile names follow from the map area, so the adapter needs neither the Metalink files (per municipality,
  `https://geodaten.bayern.de/odd/a/dgm/dgm1/meta/metalink/<AGS8>.meta4`, with size and SHA-256) nor the
  `poly2metalink` service (POST of an EWKT polygon; its Metalink carries neither size nor hash).

## Surface: DOM20

`https://download1.bayernwolke.de/a/dom20/DOM/32<E>_<N>_20_DOM.tif`. One tile is 1 km, 5000 × 5000 px of 20 cm,
Float32, LZW in tiles of 256 px, nodata `-9999`, 32–56 MB. It is computed from the aerial pictures of the
orthophoto flight; the tag `GDAL_METADATA` at the start of the file holds `BILDFLUG_DATUM`. There is no 1 m
surface model.

## Buildings

There are two open sources, both files.

**LoD2** (what the adapter reads): CityGML 1.0 in the AdV profile, one file per 2 km tile,
`https://download1.bayernwolke.de/a/lod2/citygml/<E>_<N>.gml`, `<E>` and `<N>` even, the south-west corner in km.
Sizes: 2.8 MB in the country, 61 to 108 MB in Regensburg, 161.6 MB for `690_5334` (centre of Munich). The root is `CityModel` (default
namespace `http://www.opengis.net/citygml/1.0`), each building a `cityObjectMember` with one `bldg:Building`:

- `gml:id` (`DEBY_LOD2_27318`), and the ALKIS object id in `externalReference/externalObject/name`
  (`DEBYvAAAAABOr6en`);
- `gen:stringAttribute` with `name="Grundrissaktualitaet"` (`2025-12-02`) and `name="Gemeindeschluessel"`;
- `bldg:function` as `<object type>_<code>`, such as `31001_2000`: the ALKIS Gebäudefunktion (`31001` is
  `AX_Gebaeude`);
- `bldg:roofType`, `bldg:measuredHeight`, on some `bldg:storeysAboveGround`;
- `bldg:boundedBy/bldg:GroundSurface` with one `gml:Polygon` whose `gml:posList` has three values per point
  (`srsDimension="3"` stands on the `gml:MultiSurface` above it), the outline of the building at ground level.

The envelope names `urn:adv:crs:ETRS89_UTM32*DE_DHHN2016_NH`. There is no name and no flag for underground
buildings. What else the tiles hold: see "LoD2 against the Hausumringe".

**Hausumringe** (not read): one Shape zip per Regierungsbezirk
(`https://geodaten.bayern.de/odd/m/3/daten/hausumringe/bezirk/data/091_Oberbayern_Hausumringe.zip`), 66 to 174 MB,
last changed 2026-10-01, updated every quarter. The only attribute is `ags`. The `.shp` in the zip is deflated,
so a range request cannot cut an area out of it.

No other open source gives the function of a building.

## Land use: Tatsächliche Nutzung

One Shape zip per Kreis (71 Landkreise and 25 kreisfreie Städte),
`https://download1.bayernwolke.de/a/tn/lkr/tn_<AGS5>.zip` with the five-digit key (`tn_09565.zip`, Schwabach).
Sizes from 1.8 MB (Schwabach) to 12.6 MB (the city of Munich); updated monthly (sample of 2026-10-15).

The zip holds `Nutzung.shp`, `.shx`, `.dbf`, `.prj`, `.cpg`: polygons in EPSG:25832, text in UTF-8, fields
`gml_id`, `oid`, `aktualit`, `nutzart`, `bez`, `name`. That is the simplified ALKIS schema the other states
serve, with texts instead of codes. `nutzart` values of the sample (7 638 records): Wohnbaufläche, Weg,
Unland/Vegetationslose Fläche, Landwirtschaft, Straßenverkehr, Fläche gemischter Nutzung, Industrie- und
Gewerbefläche, Wald, Gehölz, Sport-, Freizeit- und Erholungsfläche, Fläche besonderer funktionaler Prägung,
Fließgewässer, Stehendes Gewässer, Platz, Bahnverkehr, Friedhof, Tagebau/Grube/Steinbruch. `bez` of Sport-,
Freizeit- und Erholungsfläche: Grünanlage, Park, Spielplatz/Bolzplatz, Sportanlage, Kleingarten, Garten and
others; of Platz: Parkplatz, Marktplatz. Wald has no leaf or needle attribute. Other Kreise will show more values.

The index with the outlines of the Kreise: `https://geodaten.bayern.de/odd/m/4/tn/lkr/kml/kreis.kml?service=kml`
(6.3 MB, 96 placemarks). The same land use as one GeoPackage for the state is 5.4 GB; the WMS
`…/od/wms/alkis/v1/tn` answers pictures and attributes, no geometry.

## OpenStreetMap

Geofabrik has `europe/germany/bayern` (818 MB) and seven extracts by Regierungsbezirk, each with a `.poly`:

| Extract | MB |
|---|---|
| `europe/germany/bayern/mittelfranken` | 91 |
| `europe/germany/bayern/niederbayern` | 91 |
| `europe/germany/bayern/oberbayern` | 248 |
| `europe/germany/bayern/oberfranken` | 90 |
| `europe/germany/bayern/oberpfalz` | 81 |
| `europe/germany/bayern/schwaben` | 122 |
| `europe/germany/bayern/unterfranken` | 99 |

## Boundary

Made on 2026-11-14 from the administrative units of the agency
(`https://geodaten.bayern.de/odd/m/4/verwaltung/alkis-verwaltung.zip`, 63.6 MB, of 2026-10-01, CC BY 4.0): one
Shapefile `VerwaltungsEinheit` in EPSG:25832 with 3 871 units, among them the unit `Bundesland` (1), `Kreis /
kreisfreie Stadt` (96) and `Regierungsbezirk` (7). Tools: Python with shapely 2.2.0, pyproj 3.8.0 and pyshp, in a
scratch folder; nothing of them is in the repository.

**The state** (`boundary.geojson`): the unit `Bundesland`, moved to EPSG:25833, simplified 25 m keeping the
topology, on a 1 m grid, the exterior clockwise, in the layout of the file of Baden-Württemberg.

- The source is one part of 70 612.87 km² with 164 099 vertices and three holes: Jungholz (7.07 km², Austria,
  which touches the rest of Austria in a single point) and two degenerate holes (15 m² and 0 m², one of them
  1.2 km long). Holes below 1 ha are left out; with them the simplification kept the topology by widening them
  to 3 422 m².
- The file: 10 663 vertices, 180 890 bytes, area 70 613.00 km², the outline at most 25.4 m from the source.
  Extent E 68 891 … 414 735, N 5 246 260 … 5 612 804 (EPSG:25833).
- Against the neighbours that have a boundary (shared border = within 150 m; gap = within 60 m of both lines
  and in neither):

  | Neighbour | Shared border | Overlap | Gap |
  |---|---|---|---|
  | Baden-Württemberg | 812.4 km | 105 344 m² in 144 pieces (largest 14 671 m²) | 104 940 m² |
  | Hesse | 256.0 km | 50 005 m² in 86 pieces (largest 6 891 m²) | 86 281 m² |
  | Saxony | 39.9 km | 4 500 m² in 4 pieces (largest 2 441 m²) | 12 080 m² |

  That is the size of what the other borders show (Baden-Württemberg against Hesse: 79 633 m² and 135 291 m²);
  the margin of the region check absorbs it. Thuringia has no boundary yet.

**The Kreise** (`assets/kreise/bayern.geojson`; not in the folder of the manifest, which holds only what the manifest names): the 96 units `Kreis / kreisfreie Stadt` with their key (`ags`, five digits,
the same as in `tn_<key>.zip`) and name, moved to EPSG:25833, simplified 100 m. 679 660 bytes, 39 372 vertices;
the outlines lie at most 181 m from the source, and of the 151 parts one of 178 m² fell away. The adapter reads
the file of a Kreis whose outline comes within 500 m of the map area, so the simplification loses no object. (At
40 m the file was 1.3 MB.) The index KML of the land use product was not needed.

**The OSM extracts** (`osm-<name>.geojson`): Geofabrik's `europe/germany/bayern/<name>.poly` of 2026-11-14,
moved to EPSG:25833, simplified 25 m, in the layout of the files of Baden-Württemberg (ODbL). Together they
leave 0.019 km² of the state uncovered; there the nearest extract is taken.

## LoD2 against the Hausumringe

Checked on 2026-11-13 in a box of 2 × 2 km around the Steinerne Brücke in Regensburg (the four LoD2 tiles
`724_5432`, `724_5434`, `726_5432`, `726_5434`, 61 to 108 MB each, changed 2026-09-29; the Hausumringe of the
Oberpfalz, changed 2026-10-01).

- Of the 5 258 Hausumringe with at least 10 m², 5 242 have their inner point in a LoD2 ground surface:
  **99.70 %**. The 16 without one: seven of 10–20 m², seven of 20–50 m², two larger. 445 Hausumringe below 10 m²
  were not counted.
- The box holds 5 783 LoD2 objects. Every one has an ALKIS id and exactly one ground polygon; 72 ground polygons
  have inner rings. `bldg:BuildingPart` does not occur.
- The object types of `bldg:function`: 5 269 buildings (`31001_…`, 24 functions; 2 739 `31001_1000`, 1 697
  `31001_2000`, 510 `31001_2463` Garage, 109 `31001_9998`), **482 `51009_1610` (Überdachung)**, 9 `51009_1700`
  (Mauer), 8 `51007_1500`, 14 `53001_1800` (bridges), 1 `52003_1020`. So LoD2 holds roofed structures, and it
  holds structures that are no buildings at all.
- Five objects are `31001_2465` (Tiefgarage), with measured heights of 1.7 to 3.9 m and areas of 29 to 197 m²:
  what LoD2 holds of an underground garage is the part that stands above the ground.
- A tile without buildings exists as a file of 1 085 bytes: a `core:CityModel` with its envelope and no member
  (`714_5332`, Ebersberger Forst; `672_5306`, Starnberger See; both of 2022). Note the prefix: the root is
  `core:CityModel` there and `CityModel` in the tiles with buildings. Only a tile outside the state answers 404.

## Live run

2026-11-14, with the release build of the branch, an empty cache and `CI=1`:

| | Regensburg, old town (sprint 1:4000, 0.9 × 0.8 km) | Schmausenbuck (forest 1:10000, 1.2 × 0.9 km) |
|---|---|---|
| Time, first run | 1 min 16 s | not measured |
| DGM1 | 2 tiles, 1.9 MB each | 4 tiles |
| DOM20 | 2 tiles, cached reduced with 2.3 MB each; flight 11.05.2025 | 4 tiles; flight 13.06.2025 |
| LoD2 | `724_5434` 61 MB and `726_5434` 107 MB, cached as 1.5 and 2.6 MB of features | 4 tiles; of the two of the first try `654_5478` had 37 MB and `656_5478` next to nothing |
| Land use | the city of Regensburg, `tn_09362.zip`, 3.1 MB | the first try, south of the kept area, read the files of two Kreise: the city of Nuremberg and the Nürnberger Land |
| OSM extract | Oberpfalz, 81 MB | Mittelfranken, 91 MB |
| Buildings, land use parts | 1 821, 555 | 150, 124 |
| Map objects, candidates, symbols | 2 959, 453, 38 | 670, 81, 35 |

No warning in either run. A second run of Regensburg with `--offline --force` wrote the same map and the same
candidates byte for byte, and the same `freemapper.toml`. Of the 2 959 map objects of Regensburg 144 are canopies
(522) from the roofed structures of LoD2. The estimate takes 40 MB per LoD2 tile; in the old town of Regensburg
the two tiles had 168 MB, so the estimate of the buildings is low by half in a city centre and high in the
country.

### The vegetation run

On 2026-11-14 the Schmausenbuck was run again with `--content-set vegetation=candidates`: 5 min 19 s with the
four point-cloud tiles to load (239, 301, 316 and 354 MiB, 1.2 GB), 1 min 30 s offline afterwards, no warning.
All four tiles were flown on 2021-12-20. Before the export 94.0 % of the classified 1.5 km² are runnable forest
(405), 5.1 % open land (401) and 0.8 % green classes; the area is about nine tenths forest. The sports field,
the clearing at the substation and the southern swath of the power line are open. The northern swath, young
growth of 2 to 5 m, has 45 to 55 % of its returns above 0.9 m (the wires give 3 to 7 %) and comes out as forest,
because the bare twigs give a green index of 0.31. The candidates map gets 39 vegetation areas (20 of 401, 3 of
405, 13 of 406, 3 of 410); the example has 670 map objects, 120 candidates and 37 symbols. The offline run wrote
the same map, candidates and project file.

Three more tiles from other parts of the state: 780_5420 (flown 2017-11-24, 10 points per m², its 40 193 points
of class 24 carry a GPS time of the year 8416), 691_5335 (flown 2022-02-27, 20 points per m²) and 610_5480 (GPS
week time, so no day; made 2018-08-08, 9 points per m²).

## Point clouds

Checked live on 2026-11-14 (24 requests, one tile downloaded). Product "Laserdaten" of the portal, CC BY 4.0,
updated lot by lot ("losweise").

- **Address**: `https://geodaten.bayern.de/odd_data/laser/<E>_<N>.laz`, `<E>` and `<N>` the south-west corner in
  km of a 1 km tile in EPSG:25832. The host is a third one, next to `geoservices.bayern.de` and
  `download1.bayernwolke.de`; `download1` does not have the tiles (404). `HEAD` gives `Content-Length`,
  `Last-Modified` and `Accept-Ranges: bytes`. A tile outside the state answers 404 with an HTML page.
- **Size**: 34 to 614 MB in fourteen tiles across the state (median 230 MB); the portal says up to 1 GB. The four
  tiles of the Schmausenbuck have 239 to 354 MB, 1.2 GB together.
- **Format**: LAS 1.2, point format 1 (with GPS time), compressed with LAStools, scale 1 mm, no CRS in the
  header. Tile 655_5479: 51.6 million points (52 per m²), up to seven returns of a pulse.
- **Classes** (the agency's "Beschreibung der Punktklassen", September 2023): 1 unclassified, 2 ground,
  6 building, 9 water (only in data of earlier years), 20 object, for example vegetation, 22 bridge until 2020
  and cellar exit from 2021, 23 synthetic ground from a terrestrial survey, 24 synthetic ground from the update
  of the surface model. No point of the tile carries the synthetic or the withheld flag. In tile 655_5479: 62.8 %
  class 20 (9.7 % of them single echoes), 36.1 % class 2, 1.1 % class 6 (81 % single echoes), 930 points of
  class 7 and 3609 of class 22.
- **Date**: no list and no service names the flight of a tile. The grid service of the portal
  (`https://geoservices.bayern.de/od/wms/grid/v1/opendatagrid`, layer `laser`, `GetFeatureInfo` as `text/html`)
  gives only the name, the size and the address. `Last-Modified` is the day the file was put on the server
  (2023-03-10 for the Schmausenbuck, 2026-09-03 for tiles south of Augsburg). The header's creation day is the
  day of processing (2022-05-18). The points carry it themselves: the header says adjusted standard GPS time,
  and every point of tile 655_5479 was measured on 2021-12-20, a flight without leaves.
- **Metalink** per municipality (`https://geodaten.bayern.de/odd/a/laser/meta/kml/gemeinde.kml`) with size and
  hash, as for the other tiles; an area would first have to find its municipalities, so it is not used.

## Open, and left for later

- **ATKIS Basis-DLM as a WFS 2.0**: `https://geoservices.bayern.de/wfs/v1/ogc_atkis_basisdlm.cgi`, 80 feature
  types, CC BY 4.0, GML 3.2, no paging. A `BBOX` works only together with
  `NAMESPACES=xmlns(adv,http://www.adv-online.de/namespaces/adv/gid/7.1)`, and one type per request. It has
  `AX_WegPfadSteig`, `AX_Fahrwegachse`, `AX_Gewaesserachse`, `AX_DammWallDeich`, `AX_Einschnitt`,
  `AX_FelsenFelsblockFelsnadel`, `AX_Vegetationsmerkmal`; it has no `AX_Gebaeude`, `AX_BoeschungKliff` or
  `AX_Gelaendekante`. Walls are sparse (9 lines in 14 km² of Munich), steps were not seen.
- **Einzelbäume** (single trees from the point clouds): one GeoPackage per flight lot, hundreds of MB.
- **OGC API Features**: the newsletter 02/2026 describes a function of the BayernAtlas viewer. No endpoint of the
  LDBV was found; `https://geoservices.bayern.de/ogcapi/` answers 404.

## Samples

The answers and small samples of this research (WMS capabilities, the feature info, the head of a LoD2 tile, the
Metalink files of one municipality) were kept outside the repository during the design. The fixtures of the
tests are cut from fresh answers when the adapter is built.
