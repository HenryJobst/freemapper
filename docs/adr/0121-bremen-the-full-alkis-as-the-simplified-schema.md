# 0121. Bremen: the full ALKIS as the simplified schema, and grid points taken as cells

Date: 2026-11-13
Status: Accepted

The user chose Bremen as the next state on 2026-11-13. Research:
[`docs/regions/bremen-reference.md`](../regions/bremen-reference.md) (2026-11-13). It is the third state added on
the adapter scheme of [0082](0082-region-adapters-trait-and-manifests.md), after Saarland
([0111](0111-saarland-the-first-state-on-the-adapter-scheme.md)) and Baden-Württemberg
([0119](0119-baden-wuerttemberg-ndom-on-the-terrain-and-alkis-by-code.md)).

## Context

The Landesamt GeoInformation Bremen publishes what a map needs under CC BY 4.0. Four things do not fit the forms
freemapper had:

- **No simplified ALKIS.** The state serves its full ALKIS only: a WFS in the AAA schema of the AdV, run by the
  LGLN of Lower Saxony, with one feature type per land use class and codes in place of texts. The simplified
  schema that every other state serves is derived from exactly these objects.
- **Layers named by flight.** The orthophoto WMS has one layer per flight year and city (`dop10_2025_HB`,
  `dop10_2025_BHV`) and no layer for "the newest". The two cities were not always flown in the same year.
- **Height tiles in three spellings.** DGM1 and DOM1 are XYZ text, all tiles of a city in one archive. Three of
  the four archives write the points at whole metres, one at the cell centres; two put the zone number in front
  of the easting; one starts in the south and repeats the first column of the next tile.
- **Old height models.** Both come from the laser scans of 2017 (Bremen) and 2015 (Bremerhaven); the orthophoto
  is of 2025. A newer surface model exists (image-based, March 2025) and costs a fee.

## Decision

- Bremen is a built-in state: manifest folder `crates/adapters/de/manifests/bremen/`, id `bremen`, provider
  `geobremen`, raster backend `geobremen` (`crates/adapters/de/src/geobremen.rs`), EPSG:25832.
- **The full ALKIS is read as the simplified schema**, by a source that knows no state:
  `freemapper_sources::aaa::AaaAlkis`. It asks `AX_Gebaeude`, `AX_Bauteil`, the roofs among
  `AX_SonstigesBauwerkOderSonstigeEinrichtung` and the 26 land use types, one request each, with the reader of
  the full ALKIS layers ([0062](0062-full-alkis-from-the-nas-wfs.md)), and writes `gebnutzbez`, `funktion`,
  `rellage`, `nutzart`, `bez`, `aktualit` and the oid of the simplified services (`gml:id` + `BL` or `TN`). The
  texts are the labels of the AdV code lists, added to `assets/rules/adv-codes.toml` (21 lists, 502 codes). A
  use that lies over another one (`istWeitereNutzung`) is left out, as the simplified services do. The rule was
  compared with the two services the LGLN runs for Lower Saxony, object by object: 7 463 land use areas and
  22 585 buildings, parts and roofs in five areas, none different.
- The adapter of Bremen names the service and the `bez` of a square without a function
  (`ohne Unterart (Bremen)`: such squares stay candidates, as in Lower Saxony). The manifest declares no
  `[alkis]`; the source is code, like Berlin's and Baden-Württemberg's, and no kind was added to the manifest
  format.
- **The surveyed lines come by the manifest**: the same service is the `[[extra.nas]]` source, as the LGB's is
  for Brandenburg. Stairs, ramps, retaining walls, monuments, embankment tops and bridges become what they
  become there.
- **The newest flight is read from the capabilities.** The backend takes the newest colour layer of each city
  and asks for both in one request; the date of the image is the flight year of the city at the centre of the
  map area, recorded with the label `Bildflugjahr`. When the capabilities cannot be read, it asks for the layers
  of 2025.
- **A grid point stands for the cell it lies in.** A point at whole metres becomes the cell to its north-east.
  The terrain and the surface of Bremen and the surface of Bremerhaven therefore lie half a metre off to the
  north-east; the terrain of Bremerhaven, written at cell centres, does not. Points outside the tile are passed
  over, and a file without a point of the tile is an error.
- **The surface model is the DOM1 of the laser scan.** The fetch warns that it is older than the orthophoto, with
  both years, as Lower Saxony does.
- The tiles of an area are read out of the archive of their city (north of N 5 910 000: Bremerhaven) by range
  requests and kept in the cache as GeoTIFFs. The directory of an archive and an XYZ tile kept as a GeoTIFF are
  now shared with Baden-Württemberg (`crates/adapters/de/src/xyz_archive.rs`).
- Credit: `© GeoBasis-DE / Landesamt GeoInformation Bremen (<year>), CC BY 4.0, Daten geändert`, the source note
  of the agency's open-data page with the licence and the note of changes.
- The boundary is the union of the municipal areas of the cadastre. It leaves 8 km² of the Weser off Bremerhaven
  to Lower Saxony, whose boundary holds the same water, so that no two states share ground.
- Like Saarland and Baden-Württemberg, the state is not in the golden net. It has unit tests with synthetic
  archives and recorded answers of the WFS, and `adapters check --live`. Twenty border boxes of the net's
  geometry layer changed: they lie on the border of Lower Saxony to Bremen and now name it.

## Consequences

- An area of Lower Saxony that reaches into Bremen gets buildings and land use from Bremen there. It used to get
  the warning of a part outside the supported states, which now names Bavaria and Thuringia.
- The nDSM template shows the city as it was in 2017 or 2015: buildings put up since are missing, demolished
  ones are still there. Cost if the choice of the DOM1 is wrong: none to switch, there is no other open surface
  model.
- Cost if taking a grid point for its cell is wrong: the hillshade, the contours and the nDSM lie 0.7 m off
  diagonally, a fifth of a millimetre at 1:4000. Putting it right means writing those models on a grid moved by
  half a metre, and moving the surface of Bremerhaven onto the grid of its terrain.
- A map asks about 30 requests for the cadastre where another state asks two. Around the main station of Bremen
  they took 14 s for 4 km². The service refuses several types in one request.
- The archives carry the year of the scan in their name. When the agency publishes a new scan, the fetch fails
  with the address that is gone, until the adapter names the new one. The orthophoto does not have this
  problem.
- The area page shows the aerial photo of the city of Bremen and none of Bremerhaven: a preview is one layer.
  A second layer needs a list in the manifest format.
- `AaaAlkis` is code. A folder adapter cannot use an AAA service until the manifest format gets a kind for it;
  the source is written so that such a kind only has to name the service.
- Towers, tanks, masts and chimneys, which the simplified services carry as further kinds, are not derived:
  freemapper draws none of them. The names of buildings are left out, because Bremen writes file numbers there.
- `adapters check --live` on the manifest folder asks the rasters, the layers of the full ALKIS and OSM, and
  says "alkis … not asked: the adapter declares none", as for Berlin and Baden-Württemberg. Buildings and land
  use are covered by a real run.
