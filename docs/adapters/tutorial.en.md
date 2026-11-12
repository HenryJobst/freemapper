# Tutorial: an adapter in six steps

You build a folder adapter for a made-up region, "Elbaue", a piece of land around Dessau, and end with a check
that passes and with real data from real servers. It takes about twenty minutes. German:
[tutorial.de.md](tutorial.de.md).

The region is made up, the services are not: they are the open services of Saxony-Anhalt, so that every request
in this tutorial gets an answer. Saxony-Anhalt is built into freemapper already. The check will tell you so,
and you should not copy this adapter into your adapter folder: a map there would get every object twice. For a
region of your own you go through the same steps with the services of that region; the [guide](en.md) says how
to find them, the [reference](manifest-v1.en.md) lists every key.

You need freemapper 0.14.0 or later and a text editor.

## 1. The folder

An adapter is a folder that has the id of the adapter as its name. The id is a lower-case letter followed by 1
to 31 lower-case letters or digits.

```sh
mkdir -p ~/work/elbaue
cd ~/work
```

## 2. The boundary

freemapper asks an adapter only for map areas inside its boundary. The boundary is a GeoJSON file with one
polygon. For this tutorial six corners are enough; write them into `elbaue/boundary.geojson`:

```json
{
  "type": "Feature",
  "properties": { "name": "Elbaue" },
  "geometry": {
    "type": "Polygon",
    "coordinates": [[
      [12.15, 51.80], [12.33, 51.80], [12.36, 51.86],
      [12.30, 51.90], [12.17, 51.89], [12.15, 51.80]
    ]]
  }
}
```

The positions are longitude and latitude, and the last one repeats the first.

## 3. The manifest: who, where, which licence, and the cadastre

Write this into `elbaue/manifest.toml`:

```toml
format = 1

[adapter]
id = "elbaue"
revision = 1
min_freemapper = "0.14.0"
crs = 25832
names = { en = "Elbe meadows", de = "Elbaue" }
agency = "LVermGeo"

[net]
hosts = ["www.geodatenportal.sachsen-anhalt.de"]

[boundary]
file = "boundary.geojson"
crs = 4326
simplify_m = 0

[[osm.extract]]
geofabrik = "europe/germany/sachsen-anhalt"
megabytes = 177
label = "Saxony-Anhalt"

[attribution]
licence = "dl-de/by-2-0"
text = "© GeoBasis-DE / LVermGeo ST {year}, dl-de/by-2-0, Daten geändert"
note = "Lizenz dl-de/by-2-0: https://www.govdata.de/dl-de/by-2-0"

[alkis]
kind = "wfs"
url = "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_ALKIS_WFS_OpenData/guest"
buildings_type = "ave:GebaeudeBauwerk"
land_use_type = "ave:Nutzung"
```

What the tables say:

- `[adapter]`: the id again, a revision that starts at 1, the oldest freemapper that reads the file, the CRS of
  a map in the region (EPSG:25832, UTM zone 32) and the names.
- `[net]`: the servers the adapter may ask. freemapper sends its requests there and nowhere else.
- `[boundary]`: the file of step 2. `crs = 4326` says its positions are degrees; `simplify_m = 0` says nobody
  simplified it.
- `[[osm.extract]]`: the OpenStreetMap extract of the Geofabrik server that covers the region, with its size in
  megabytes for the download estimate.
- `[attribution]`: the licence of the data and the credit line that goes onto every map. `{year}` becomes the
  year of the download. The LVermGeo prescribes the name `© GeoBasis-DE / LVermGeo ST`; the licence asks for
  its own name, for the note that the data were changed, and for the address of its text, which `note` puts
  into `ATTRIBUTION.txt`.
- `[alkis]`: the cadastre. The WFS serves the simplified ALKIS schema; the two type names are those of its
  capabilities (`…/guest?SERVICE=WFS&REQUEST=GetCapabilities`).

Now the first check. It reads the folder and asks no server:

```sh
freemapper adapters check elbaue
```

```
adapter elbaue (Elbe meadows), revision 1, hosts: www.geodatenportal.sachsen-anhalt.de
warning: /home/you/work/elbaue/manifest.toml: adapter.last_verified: the manifest does not say when its services were last checked against it [check.never-verified]
warning: /home/you/work/elbaue/manifest.toml: boundary.file: the adapter lies on the ground of saxonyanhalt: a map there gets the objects of both [check.shares-ground]
/home/you/work/elbaue passes the check, with 2 warnings
```

The adapter would load. The second warning is the one announced above: the region lies inside Saxony-Anhalt.
An adapter for a region freemapper does not know yet has no such warning.

## 4. The rasters

A map needs an orthophoto and a terrain model; a surface model adds the vegetation height. Saxony-Anhalt serves
all three as standard services, so the manifest can declare them. Append:

```toml
[orthophoto]
kind = "wms"
url = "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest"
layer = "lsa_lvermgeo_dop20_2"
max_px = 4000

[orthophoto.date]
kind = "wfs-geojson"
url = "https://geodatenportal.sachsen-anhalt.de/ows_lvermgeo_wfs_st_geobasis"
layer = "atkis_geobasis_bestand:truedop_kacheln"
field = "BFDATUM"
format = "DD.MM.YYYY"

[terrain]
kind = "wcs"
url = "https://www.geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DGM1"
coverage = "Coverage1"
multipart = true
envelope = [606000, 5646000, 790000, 5882000]
max_m = 4000

[surface]
kind = "wcs"
url = "https://geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DOM1"
coverage = "Coverage1"
multipart = true
envelope = [606000, 5646000, 790000, 5882000]
max_m = 4000
```

- `[orthophoto]`: a WMS, its layer, and the longest side freemapper may ask for in one request. A larger area
  is fetched in parts.
- `[orthophoto.date]`: where the flight date comes from. Here a second service has a layer with one tile per
  flight block and the date in the field `BFDATUM`.
- `[terrain]` and `[surface]`: a WCS each. `coverage` and `envelope` are in the answer to
  `…/ows_WCS_ST_DGM1?SERVICE=WCS&VERSION=2.0.1&REQUEST=DescribeCoverage&COVERAGEID=Coverage1`: the id, and the
  lower and upper corner in EPSG:25832. `multipart = true` because this server wraps the TIFF in a
  `multipart/related` answer.

Check again:

```sh
freemapper adapters check elbaue
```

```
error: /home/you/work/elbaue/manifest.toml:43: orthophoto.date.url: https://geodatenportal.sachsen-anhalt.de/ows_lvermgeo_wfs_st_geobasis: the host geodatenportal.sachsen-anhalt.de is not in `[net] hosts` [policy.host-not-declared]
error: /home/you/work/elbaue/manifest.toml:58: surface.url: https://geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DOM1: the host geodatenportal.sachsen-anhalt.de is not in `[net] hosts` [policy.host-not-declared]
/home/you/work/elbaue fails the check
```

Two of the services live on a second host, without `www.`. Each finding names the file, the line, the key and,
in brackets, a code that the reference explains. Add the host in `[net]`:

```toml
hosts = ["www.geodatenportal.sachsen-anhalt.de", "geodatenportal.sachsen-anhalt.de"]
```

## 5. The aerial photo of the area page, and who checked when

Append the preview, which the area page shows behind the map area while someone draws it:

```toml
[preview]
wms = "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest"
layer = "lsa_lvermgeo_dop20_2"
format = "image/jpeg"
attribution = "© GeoBasis-DE / LVermGeo ST, dl-de/by-2-0"
agency = "LVermGeo"
```

And add two lines to `[adapter]`, below `agency`, with your name on GitHub and today's date:

```toml
maintainers = ["github:your-name"]
last_verified = "2026-11-12"
```

```sh
freemapper adapters check elbaue
```

```
adapter elbaue (Elbe meadows), revision 1, hosts: www.geodatenportal.sachsen-anhalt.de, geodatenportal.sachsen-anhalt.de
warning: /home/you/work/elbaue/manifest.toml: boundary.file: the adapter lies on the ground of saxonyanhalt: a map there gets the objects of both [check.shares-ground]
/home/you/work/elbaue passes the check, with one warning
```

## 6. Ask the services

So far no server was asked. `--live` fetches what a map of a small box would need. The box is given in degrees
as `minLon,minLat,maxLon,maxLat` and lies inside the boundary; 150 m by 150 m is enough. This one is in the
centre of Dessau:

```sh
freemapper adapters check elbaue --live --bbox 12.2430,51.8350,12.2452,51.8364
```

```
adapter elbaue (Elbe meadows), revision 1, hosts: www.geodatenportal.sachsen-anhalt.de, geodatenportal.sachsen-anhalt.de
warning: /home/you/work/elbaue/manifest.toml: boundary.file: the adapter lies on the ground of saxonyanhalt: a map there gets the objects of both [check.shares-ground]
rasters ok: orthophoto, terrain model, surface model
alkis-buildings ok: 58 objects
alkis-land-use ok: 60 objects
nas not asked: the adapter declares none
osm ok: Saxony-Anhalt, data of 2026-11-11
/home/you/work/elbaue passes the check, with one warning
```

One line per capability: the three rasters arrived, the cadastre delivered buildings and land use, and the
OpenStreetMap extract is on the server (it is not downloaded for the check). The numbers and the date will
differ on your day. The exit code is 0 when the adapter passes, 1 with findings, 2 when the check could not
run.

## What comes after

- To use an adapter, copy its folder into your adapter folder; `freemapper adapters list` names that folder in
  its last line and then lists the adapter with "from your folder". Do not do this with `elbaue`, for the reason
  given at the top.
- To offer an adapter to everyone, see "Submit" in the [guide](en.md#9-submit).
- A region whose heights do not come from a WCS needs code; the guide says when and what.
