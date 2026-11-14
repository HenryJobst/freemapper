//! Permit sources that are shapefiles in a zip for a whole state (ADR 0131), and the reading
//! they share with the forest sources of North Rhine-Westphalia (ADR 0124): the zip is
//! downloaded (50 MB at most), its `.shp` and `.dbf` read, the records whose box meets the map
//! box kept and turned into GeoJSON features as longitude, latitude with only the fields their
//! reading needs. A table without a `.cpg` in Latin-1 keeps its umlauts (the shapefile reader
//! falls back to it).

use std::io::Read as _;
use std::path::Path;

use freemapper_adapter_api::permits::{
    PermitFinding, PermitSection, PermitSource, PermitSourceInfo, Wording,
};
use freemapper_sources::shapefile::{ShapeRecord, polygons_in};
use geo_types::Rect;
use serde_json::Value;

use crate::crs::{BoundingBox, Crs, LatLon, ProjectedPoint};
use crate::fetch::{FetchError, Fetcher, get_limited_with_retries};
use crate::wfs_permits::{Reading, finding, stored};

/// The largest zip read; the largest one used is NRW's water protection areas (15 MB).
const MAX_ZIP: u64 = 50 * 1024 * 1024;

/// The largest file read out of the zip; the largest is the table of NRW's Staatswald (18 MB).
const MAX_ENTRY: u64 = 100 * 1024 * 1024;

/// How far around the map box records are kept, in metres: a record whose box only nearly
/// meets it costs nothing, one cut off would be missing.
const MARGIN_M: f64 = 50.0;

/// The records of the zip at `url` (one shapefile in `crs`) whose box meets the map box `bbox`
/// (WGS 84, x = longitude). A download that is no such zip is an error with the URL.
pub(crate) fn shape_records(
    fetcher: &dyn Fetcher,
    url: &str,
    crs: Crs,
    bbox: &Rect<f64>,
    part: &Path,
) -> Result<Vec<ShapeRecord>, FetchError> {
    let cut = map_box(bbox, crs);
    get_limited_with_retries(fetcher, url, part, MAX_ZIP, |bytes| {
        records_in(bytes, &cut).map_err(|message| FetchError::Http {
            url: url.to_string(),
            message,
        })
    })
}

/// The map box (WGS 84, x = longitude) in `crs`, with [`MARGIN_M`] around it.
fn map_box(bbox: &Rect<f64>, crs: Crs) -> BoundingBox {
    let corners = [
        (bbox.min().y, bbox.min().x),
        (bbox.min().y, bbox.max().x),
        (bbox.max().y, bbox.min().x),
        (bbox.max().y, bbox.max().x),
    ]
    .map(|(lat, lon)| crs.forward(LatLon { lat, lon }));
    let fold = |pick: fn(f64, f64) -> f64, start: f64, axis: fn(&ProjectedPoint) -> f64| {
        corners.iter().map(axis).fold(start, pick)
    };
    BoundingBox {
        min_e: fold(f64::min, f64::INFINITY, |p| p.easting) - MARGIN_M,
        min_n: fold(f64::min, f64::INFINITY, |p| p.northing) - MARGIN_M,
        max_e: fold(f64::max, f64::NEG_INFINITY, |p| p.easting) + MARGIN_M,
        max_n: fold(f64::max, f64::NEG_INFINITY, |p| p.northing) + MARGIN_M,
    }
}

/// The records of the one `.shp` and `.dbf` in the zip `bytes` whose box meets `cut`.
fn records_in(bytes: &[u8], cut: &BoundingBox) -> Result<Vec<ShapeRecord>, String> {
    let mut zip =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| format!("not a zip: {e}"))?;
    let mut entry = |extension: &str| -> Result<Vec<u8>, String> {
        let index = (0..zip.len())
            .find(|&i| {
                zip.by_index_raw(i).is_ok_and(|f| {
                    f.name()
                        .is_ok_and(|n| n.to_ascii_lowercase().ends_with(extension))
                })
            })
            .ok_or_else(|| format!("the zip has no {extension} file"))?;
        let mut content = Vec::new();
        // A file larger than any of the states' is cut off and then refused below, so a
        // packed bomb never fills the memory.
        zip.by_index(index)
            .map_err(|e| e.to_string())?
            .take(MAX_ENTRY + 1)
            .read_to_end(&mut content)
            .map_err(|e| e.to_string())?;
        if content.len() as u64 > MAX_ENTRY {
            return Err(format!(
                "the {extension} file is larger than {MAX_ENTRY} bytes"
            ));
        }
        Ok(content)
    };
    let shp = entry(".shp")?;
    let dbf = entry(".dbf")?;
    polygons_in(&shp, &dbf, cut)
}

/// A record as a GeoJSON feature with the fields of `kept` and its polygons in `crs` as
/// longitude, latitude; `None` when a position cannot be converted.
pub(crate) fn record_feature(record: &ShapeRecord, kept: &[&str], crs: Crs) -> Option<Value> {
    let properties: serde_json::Map<String, Value> = kept
        .iter()
        .filter_map(|key| {
            let value = record.fields.get(*key)?;
            Some((key.to_string(), Value::String(value.clone())))
        })
        .collect();
    let mut polygons = Vec::with_capacity(record.polygons.len());
    for polygon in &record.polygons {
        let mut rings = Vec::with_capacity(polygon.len());
        for ring in polygon {
            let mut positions = Vec::with_capacity(ring.len());
            for &[easting, northing] in ring {
                let p = crs.inverse(ProjectedPoint { easting, northing })?;
                positions.push(serde_json::json!([p.lon, p.lat]));
            }
            rings.push(Value::Array(positions));
        }
        polygons.push(Value::Array(rings));
    }
    Some(serde_json::json!({
        "type": "Feature",
        "geometry": { "type": "MultiPolygon", "coordinates": polygons },
        "properties": properties,
    }))
}

/// A shapefile of a state read by one [`Reading`], as the table sources of WFS are.
pub struct ShapeTableSource {
    info: PermitSourceInfo,
    url: &'static str,
    crs: Crs,
    reading: Reading,
}

/// A source of the zip at `url` (one shapefile in `crs`), its one layer `layer`.
#[allow(clippy::too_many_arguments)]
pub fn source(
    key: &'static str,
    label: Wording,
    what: Wording,
    credit: &'static str,
    layer: &'static [&'static str],
    sections: &'static [PermitSection],
    url: &'static str,
    crs: Crs,
    reading: Reading,
) -> Box<dyn PermitSource> {
    debug_assert_eq!(layer.len(), 1);
    Box::new(ShapeTableSource {
        info: PermitSourceInfo {
            key,
            label,
            what,
            credit,
            note: None,
            layers: layer,
            sections,
            offices: None,
        },
        url,
        crs,
        reading,
    })
}

impl PermitSource for ShapeTableSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let records = shape_records(fetcher, self.url, self.crs, bbox, part)?;
        let kept = self.reading.kept();
        Ok(vec![
            records
                .iter()
                .filter_map(|r| record_feature(r, &kept, self.crs))
                .filter_map(|f| stored(&f, self.reading))
                .collect(),
        ])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != self.info.layers[0] {
            return Vec::new();
        }
        features
            .iter()
            .filter_map(|f| finding(self.reading, f))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use freemapper_adapter_api::permits::FindingKind;
    use freemapper_sources::shapefile::test_support::{Entry, shape_files};
    use geo_types::Coord;
    use std::io::Write as _;

    /// A zip of the files of `records`.
    fn zip_of(records: &[Entry<'_>]) -> Vec<u8> {
        let (shp, dbf) = shape_files(records);
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        for (name, bytes) in [("wsg.shp", shp), ("wsg.dbf", dbf)] {
            zip.start_file(name, options).unwrap();
            zip.write_all(&bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    /// A square of 200 m at the south-west corner `(e, n)`.
    fn square(e: f64, n: f64) -> Vec<[f64; 2]> {
        vec![
            [e, n],
            [e, n + 200.0],
            [e + 200.0, n + 200.0],
            [e + 200.0, n],
            [e, n],
        ]
    }

    /// The box around 1 km of `crs` at `(e, n)` in WGS 84.
    fn around(crs: Crs, e: f64, n: f64) -> Rect<f64> {
        let corner = |e, n| {
            let p = crs
                .inverse(ProjectedPoint {
                    easting: e,
                    northing: n,
                })
                .unwrap();
            Coord { x: p.lon, y: p.lat }
        };
        Rect::new(corner(e - 500.0, n - 500.0), corner(e + 500.0, n + 500.0))
    }

    fn fetch(source: &dyn PermitSource, url: &str, zip: Vec<u8>, bbox: &Rect<f64>) -> Vec<Value> {
        let fetcher = MapFetcher::default();
        fetcher.insert(url, zip);
        let part = tempfile::tempdir().unwrap();
        let mut layers = source
            .fetch(&fetcher, bbox, &part.path().join("x.part"))
            .unwrap();
        assert_eq!(layers.len(), 1);
        layers.remove(0)
    }

    /// Brandenburg's zones keep their name, zone and ordinance, nothing else; a zone far from
    /// the map is not kept.
    #[test]
    fn brandenburg_water_zones_from_the_shapefile() {
        let near = [
            ("KLARNAME", "Beispielhausen"),
            ("SCHUTZZONE", "Zone II"),
            ("FUNDSTELLE", "GVBl.II/08, [Nr. 1], S.1"),
            ("DURCH", "Ministerium"),
            ("LANDKREIS", "Musterkreis"),
        ];
        let far = [("KLARNAME", "Fernab"), ("SCHUTZZONE", "Zone III")];
        let zip = zip_of(&[
            (&near, &[square(390_000.0, 5_800_000.0)]),
            (&far, &[square(450_000.0, 5_900_000.0)]),
        ]);
        let source = crate::lfb_permits::sources()
            .into_iter()
            .find(|s| s.info().key == "wasserschutzgebiete")
            .unwrap();
        let features = fetch(
            source.as_ref(),
            "https://data.geobasis-bb.de/geofachdaten/Wasser/Gewaesserbewirtschaftung/wsg.zip",
            zip,
            &around(Crs::Utm33, 390_100.0, 5_800_100.0),
        );
        assert_eq!(features.len(), 1, "{features:?}");
        let keys: Vec<&String> = features[0]["properties"]
            .as_object()
            .unwrap()
            .keys()
            .collect();
        assert_eq!(keys, ["FUNDSTELLE", "KLARNAME", "SCHUTZZONE"]);
        assert_eq!(
            source.findings("wasserschutzgebiete", &features)[0].kind,
            FindingKind::WaterProtection {
                name: Some("Beispielhausen".into()),
                zone: Some("Zone II".into()),
                citation: Some("GVBl.II/08, [Nr. 1], S.1".into()),
            }
        );
    }

    /// NRW's table is Latin-1 without a `.cpg`; a planned zone says so.
    #[test]
    fn nrw_water_zones_keep_umlauts_and_mark_planned_ones() {
        let set = [
            ("name", "D\u{fc}sterm\u{fc}hle"),
            ("wsg_zone", "1"),
            ("areastatus", "F"),
        ];
        let planned = [
            ("name", "Genkeltalsperre"),
            ("wsg_zone", "3B"),
            ("areastatus", "P"),
        ];
        let mut zip = zip_of(&[
            (&set, &[square(365_000.0, 5_617_000.0)]),
            (&planned, &[square(365_300.0, 5_617_000.0)]),
        ]);
        // The test tables are UTF-8; write the umlauts as the state does, one Latin-1 byte.
        let utf8 = "D\u{fc}sterm\u{fc}hle".as_bytes().to_vec();
        let latin1: Vec<u8> = "D\u{fc}sterm\u{fc}hle".chars().map(|c| c as u8).collect();
        zip = rezip_replacing(&zip, &utf8, &latin1);
        let source = crate::wuh_permits::sources()
            .into_iter()
            .find(|s| s.info().key == "wasserschutzgebiete")
            .unwrap();
        let features = fetch(
            source.as_ref(),
            "https://www.opengeodata.nrw.de/produkte/umwelt_klima/wasser/trinkwasser/wasserschutzgebiete/Wasserschutzgebiete-NRW_EPSG25832_Shape.zip",
            zip,
            &around(Crs::Utm32, 365_200.0, 5_617_100.0),
        );
        let kinds: Vec<FindingKind> = source
            .findings("wasserschutzgebiete", &features)
            .into_iter()
            .map(|f| f.kind)
            .collect();
        assert_eq!(
            kinds,
            [
                FindingKind::WaterProtection {
                    name: Some("D\u{fc}sterm\u{fc}hle".into()),
                    zone: Some("Schutzzone 1".into()),
                    citation: None,
                },
                FindingKind::WaterProtection {
                    name: Some("Genkeltalsperre".into()),
                    zone: Some("Schutzzone 3B (geplant)".into()),
                    citation: None,
                },
            ]
        );
    }

    /// The zip with `from` replaced by `to` in its `.dbf` (padded with spaces to the width).
    fn rezip_replacing(zip: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();
        let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let name = file.name().unwrap().to_string();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            if name.ends_with(".dbf") {
                let at = bytes.windows(from.len()).position(|w| w == from).unwrap();
                let mut padded = to.to_vec();
                padded.resize(from.len(), b' ');
                bytes[at..at + from.len()].copy_from_slice(&padded);
            }
            out.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            out.write_all(&bytes).unwrap();
        }
        out.finish().unwrap().into_inner()
    }

    #[test]
    fn a_download_that_is_no_zip_is_an_error() {
        let fetcher = MapFetcher::default();
        let url =
            "https://data.geobasis-bb.de/geofachdaten/Wasser/Gewaesserbewirtschaftung/wsg.zip";
        fetcher.insert(url, b"<html>moved</html>".to_vec());
        let part = tempfile::tempdir().unwrap();
        let err = shape_records(
            &fetcher,
            url,
            Crs::Utm33,
            &around(Crs::Utm33, 390_100.0, 5_800_100.0),
            &part.path().join("x.part"),
        )
        .unwrap_err();
        assert!(
            matches!(&err, FetchError::Http { url: u, .. } if u == url),
            "{err:?}"
        );
    }
}
