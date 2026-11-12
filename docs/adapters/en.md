# Adding a region to freemapper

A guide for people who want freemapper to make maps in a state or region it does not cover yet. German:
[de.md](de.md). The [tutorial](tutorial.en.md) builds an adapter step by step; the
[reference](manifest-v1.en.md) lists every key of the manifest.

## 1. Who this is for, and what you need

freemapper gets the data of a region through a region adapter. The German states it covers are built in. A
further region is a folder with a text file, `manifest.toml`, that names the services of the region, and a
boundary file. If the services are standard ones, that is all, and you write no code.

You need to know the region's geodata services, or be willing to read their capabilities. Before you start,
find out four things:

- A licence that allows it. freemapper downloads the data, derives map objects from them and writes maps
  that people pass on. The data must be open: dl-de/zero-2-0, dl-de/by-2-0, CC BY 4.0, CC0, ODbL, or another
  licence whose text you can quote. "Free to view" is not enough.
- The cadastre: buildings and land use in the simplified ALKIS schema of the AdV ("ALKIS vereinfacht"), from
  a WFS or an OGC API Features service.
- The rasters: an orthophoto, a terrain model (DGM1) and, if there is one, a surface model (DOM1) at 1 m.
- A boundary of the region as a polygon.

A map is drawn in EPSG:25832 or EPSG:25833, so the region must lie where UTM zone 32 or 33 fits: Germany and
its neighbours.

## 2. Decide: manifest only, manifest plus backend, or code

Go through the questions in order.

1. Does the cadastre come from a WFS or an OGC API Features service with the attributes of "ALKIS vereinfacht"
   (`oid`, `gebnutzbez`, `funktion` for buildings; `nutzart`, `bez` for land use)?
   - Yes: the manifest declares it. Go on with 2.
   - No (another schema, another country's cadastre, files to download): the region needs a **code adapter**
     in Rust that turns its data into that schema. Berlin is one. Open an issue before you start; this is more
     than a manifest.
2. Does the orthophoto come from a WMS or a WCS, and does each height model come from a WCS 2.0 that answers a
   GeoTIFF with heights at 1 m?
   - Yes: **manifest only**. Saxony-Anhalt is the example, and the tutorial rebuilds it. Such an adapter works
     from your own adapter folder, without a new freemapper.
   - No (tiles in a folder, archives, a STAC catalogue, a download index): **manifest plus backend**. The
     manifest declares everything else and names a raster backend, which is Rust code inside freemapper.
     Saarland is the example: its heights are six archives per model. A folder cannot bring code, so such a
     region reaches users only with a new version of freemapper.

Most states turn out to be the second case of question 2. Their services are standard on paper, and then one
of them has an odd side: Saarland's WFS writes its attribute names in capitals and refuses `SORTBY`, and its WCS
would have needed one parameter twice. Expect one such thing per service, and name it in your pull request.

Three limits of format 1 that send a region to a backend:

- The three raster tables are all services, or they all name one backend. One odd source means a backend for
  all three.
- `params` cannot hold a parameter twice.
- A date comes from a service the format knows (a WFS with GeoJSON, a STAC API, the feature info of the WMS)
  or not at all.

## 3. Create the folder

There is no command for this yet. By hand:

1. Pick the id: a lower-case letter, then 1 to 31 lower-case letters or digits (`saarland`, `bremen`). The
   project file of every map stores it, so it stays.
2. Make a folder with that name. It will hold `manifest.toml`, the boundary file and nothing that is not
   needed (64 files at most, no links).
3. Start `manifest.toml` from the manifest that is closest to your region in
   [`crates/adapters/de/manifests/`](https://github.com/HenryJobst/freemapper/tree/main/crates/adapters/de/manifests): `saxonyanhalt` for standard
   services, `saarland` for a WFS cadastre with a backend, `hamburg` for an OGC API with a WFS fallback.
4. Fill `[adapter]`, `[net]`, `[[osm.extract]]` first. `[net] hosts` lists every server the adapter asks.
   freemapper refuses a request to any other host, also when a redirect or a link in an answer leads there.

Work in a folder of your own, not in your adapter folder, until the check passes.

## 4. The boundary

freemapper asks an adapter for map areas inside its boundary, and at a border it uses the boundaries to tell
which region delivers what. The file is GeoJSON with one feature, a `Polygon` or `MultiPolygon`, in
EPSG:25833 or in longitude and latitude (EPSG:4326), at most 2 MiB.

There is no command for this either. What Saarland did:

1. Took the state's own outline from the cadastre: the WFS has a feature type with the administrative units,
   and one feature is the state. OpenStreetMap's boundary relation or the BKG's VG250 would do as well; write
   down which source you used, its licence applies to the file.
2. Reprojected it to EPSG:25833.
3. Simplified it to 25 m in a way that keeps the polygon valid (14 037 vertices became 1 384). QGIS does this
   with "Simplify" from the geometry tools and "Export > Save features as… > GeoJSON".
4. Compared the shared border with the neighbour's file in `crates/adapters/de/manifests/`. Two boundaries
   that overlap by more than a strip make maps at the border get objects twice; the check warns
   (`check.shares-ground`) when your region lies on another one.

`[boundary] crs` states the CRS of the file, `simplify_m` how far you simplified. `adapters check` refuses a
file whose positions cannot be of the stated CRS, a ring that crosses itself, and warns about an area under
1 km², which usually means the wrong unit.

## 5. The cadastre

Open the capabilities of the service and find the two feature types: buildings (`GebaeudeBauwerk`) and land
use. For land use there are two variants, and states differ in which one carries the attributes:

- `Nutzung`: one object per area of one use. Saxony-Anhalt, Saarland.
- `NutzungFlurstueck`: the use per parcel part. Hesse, North Rhine-Westphalia, Lower Saxony, Saxony.

Fetch a few features of each for a small box and look at them: they need `oid`, the class in `nutzart` and
`bez`, the function of a building in `gebnutzbez` and `funktion`. Then choose:

- `kind = "wfs"`: a WFS 2.0 that answers GML 3.2. Set `sorted = false` if it answers `SORTBY=oid` with an
  exception. Set `square_bez` if the state never records what a square is used for; such squares then stay
  candidates.
- `kind = "ogc-api-features"`: the two items endpoints. `normalise` says how the service spells the class
  names: `lgb` when it writes them with umlauts as Brandenburg's does, `hamburg-ascii` when it writes
  `Wohnbauflaeche`. If neither fits, the region needs a new preset, which is code. An `[alkis.fallback]` WFS is
  asked when the API fails.

If the state also has a WFS with the full ALKIS (NAS), `[[extra.nas]]` adds structures, vegetation features
and small waters. A city's tree cadastre on an Opendatasoft portal goes into `[[extra.trees]]`.

## 6. The rasters

### As services

For each table you need the address, the name of the layer or coverage, and the limits of one
request:

- `[orthophoto]`, `kind = "wms"`: the layer with the true-colour image at 20 cm, and `max_px`, the longest side
  the server delivers in one GetMap (its capabilities may name `MaxWidth` and `MaxHeight`; Saxony-Anhalt
  is asked with 4000).
- `[terrain]`, `[surface]`, `kind = "wcs"`: the coverage id, its extent in the CRS of the adapter (`envelope`,
  from DescribeCoverage) and `max_m`, the longest side of one request in metres. Ask for a small box by hand
  once and look at the answer: a TIFF, or a `multipart/related` body with the TIFF inside
  (`multipart = true`); 1 m per pixel, or a coarser grid that needs a parameter in `params`.
- `date`: where the flight date or the date of the model can be asked. Without it the map notes carry no date.

### As a backend

When a source is not a service, the three tables say `backend = "<name>"` and nothing else,
and the code lives in the adapter crate of the country, for Germany `crates/adapters/de`. A backend implements
one trait:

```rust
impl RasterBackend for MyBackend {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        // The orthophoto, the terrain model and the surface model of `bbox` (the map area
        // plus buffer, in the CRS of the map): into the project folder, with their records,
        // saving the project after each part. `env.fetcher` is the only way to the network,
        // `env.cache` holds what other maps can reuse, `env.mode` says whether the run is
        // offline, and with `env.skip_surface` the surface model is left out.
        Ok(())
    }
}
```

Read [`crates/adapters/de/src/lvgl.rs`](https://github.com/HenryJobst/freemapper/blob/main/crates/adapters/de/src/lvgl.rs) (Saarland) before you write
one. It shows the parts every backend has: the orthophoto from a WMS through the shared helper, tiles read out
of archives by range requests and kept in the cache, a missing surface model as a warning and not a failure,
and tests that run without the network against synthetic archives. The backend is registered under its name in
`crates/adapters/de/src/states/mod.rs`.

Adding a built-in state is not one line yet. Saarland touched about twenty places in the crate, because the
states are an enum, and some tests that count the states;
[`saarland-reference.md`](https://github.com/HenryJobst/freemapper/blob/main/docs/regions/saarland-reference.md) has the list and commit `dde0b5e` shows them. If
you do not write Rust, send the manifest folder and the research, and say so.

## 7. Credit and licence

Every map carries the credit lines of its data. Getting them right is the part of an adapter that can hurt
other people, so do it from the sources:

1. Find the licence of each product in the agency's catalogue record, and read the `Fees` and
   `AccessConstraints` of each service's capabilities. Where a service says in its own words that downloading
   needs a contract, do not use that service, whatever the catalogue says. Saarland's two INSPIRE WCS services
   were left out for this reason. Where the capabilities only point to general terms, the agency's terms of
   use decide; quote the sentence in your pull request.
2. Take the name of the provider exactly as the agency prescribes it (`© GeoBasis-DE / LVermGeo ST`,
   `© GeoBasis DE/LVGL-SL (Jahr)`), blanks and brackets included.
3. Write `[attribution] text` with what the licence asks for:

   | Licence | The line needs |
   |---|---|
   | `dl-de/by-2-0` | the provider's name, `dl-de/by-2-0`, a note that the data were changed (`Daten geändert`), and in `note` the address of the licence text |
   | `cc-by-4.0` | the provider's name, `CC BY 4.0`, a note of changes, and in `note` the address of the licence text |
   | `dl-de/zero-2-0`, `cc0-1.0` | nothing; name the provider anyway |
   | `odbl` | the provider's name and `ODbL` |
   | `other` | what its text says; the text goes into `licence_text` |

   `{year}` stands for the year of the download. freemapper adds the lines of OpenStreetMap and its own.
4. `[preview] attribution` is the credit under the aerial photo of the area page; it has no note of changes,
   because that image is shown as it comes.

Leave out what is not open, even when it would make a better map. A product behind a contract is not built into
freemapper.

## 8. Test

`freemapper adapters check <folder>` is the test. Without a switch it asks no server and tells whether the
folder would load, with every finding at once: file, line, key, and a code that the
[reference](manifest-v1.en.md#findings) explains. The texts of the findings are English in every language of
the program.

```sh
freemapper adapters check ~/work/saarland
freemapper adapters check ~/work/saarland --live --bbox 6.9930,49.2330,6.9952,49.2344
```

With `--live` it fetches what a map of the box would need, on the hosts the manifest declares, and prints one
line per capability: the rasters, buildings, land use, the full ALKIS, the OSM extract. Run it for three boxes
of about 150 m:

- a town centre, where every source has data;
- a place at the edge of the region, where services often behave differently;
- a place where two sources of yours meet, if there is one (two districts, two flight blocks).

Then make one real map with the adapter in your adapter folder and look at it: buildings where the orthophoto
shows them, contours that follow the terrain, the credit line in `ATTRIBUTION.txt`.

For a state that is built in, the code also gets tests that run without the network: recorded answers of the
services, cut down to a few features, in `tests/fixtures/`, and for a backend synthetic archives or tiles.
[`fixtures.md`](fixtures.md) describes the format of a recording. Recorded answers are published with the
repository: no address with an `@`, no name of a person, no e-mail address in a request or a file.

## 9. Submit

Open a pull request at <https://github.com/HenryJobst/freemapper>. For a German state the folder goes to
`crates/adapters/de/manifests/<id>/`. The checklist:

- [ ] `freemapper adapters check <folder>` passes; its output is in the pull request.
- [ ] `--live` passes for three boxes; the commands and the output are in the pull request.
- [ ] The licence of every product is named with the address of the page or record that states it, and the
      credit text matches what that page prescribes.
- [ ] `maintainers` names who answers when a service changes (a GitHub name, no e-mail address), and
      `last_verified` is the day of your live check.
- [ ] `[net] hosts` lists the hosts the adapter needs and no others.
- [ ] The source of the boundary is named.
- [ ] No personal data in recorded answers, file names or the manifest.
- [ ] For a new state: an example project in `examples/`, a row in the manual's table of regions (German and
      English), a row in both READMEs and a changelog entry in both languages. The maintainer helps with
      these.

What the maintainer looks at: the licence first, then the hosts, then whether the live check passes on another
day and another machine, then the map of the example. A manifest that only its author can check is not merged.

## 10. Keeping it alive

Services move, and flight years change. What an adapter needs over time:

- Run `adapters check --live` when a map in your region fails, and at least once a year. Set `last_verified`
  to that day. The check warns when the date is older than twelve months.
- When a service moves to another address, change the URL and, if the host is new, `[net] hosts`. Raise
  `revision` when the change alters the data a map gets: another service, another product, another year.
- For a built-in state, users can help themselves until the next version: the message about the refused
  address names the command `freemapper adapters allow-host <id> <host>`. Ask them to report the move.
- A new key of the format needs a newer freemapper. Set `min_freemapper` to the version that introduced the
  key, so that an older program says "needs a newer freemapper" and not "unknown key".

There is no scheduled live check of all adapters yet; broken services are noticed by people who make maps. An
adapter in the repository whose maintainer does not answer stays in as long as it works. When it breaks and
nobody fixes it, the maintainer of freemapper fixes it or takes it out with a changelog entry.
