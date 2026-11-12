//! What a fetch records in the project file and reports to the disk-space plan.

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;
use freemapper_georef::crs::BoundingBox;
use serde::{Deserialize, Serialize};

/// A raster product that comes as tiles: those of LGB (Brandenburg, Berlin) and GeoSN (Saxony).
/// It sits in this crate because the tile records name it; what LGB and GeoSN say about a
/// product (URLs, file names, folders) stays with them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Product {
    /// DOP20 RGB orthophoto, JPEG + world file, 0.2 m.
    Orthophoto,
    /// DGM1 digital terrain model, GeoTIFF f32 + world file, 1 m.
    Dtm,
    /// bDOM image-matched surface model, GeoTIFF f32 + world file, 0.2 m; the nDSM source
    /// where no nDOM point cloud is published (Berlin).
    ImageDsm,
    /// GeoSN DGM1 terrain model, 2 km tiles, GeoTIFF f32 + world file, 1 m.
    SaxonyDtm,
    /// GeoSN DOM1 laser surface model, 2 km tiles, GeoTIFF f32 + world file, 1 m; the nDSM
    /// source in Saxony, where no nDOM is published.
    SaxonyDsm,
}

impl Product {
    /// Human-readable name for messages.
    pub fn label(self) -> &'static str {
        match self {
            Self::Orthophoto => "orthophoto (DOP20)",
            Self::Dtm => "terrain model (DGM1)",
            Self::ImageDsm => "surface model (bDOM)",
            Self::SaxonyDtm => "terrain model (GeoSN DGM1)",
            Self::SaxonyDsm => "surface model (GeoSN DOM1)",
        }
    }
}

impl fmt::Display for Product {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Data currency of one tile from the LGB currency CSV.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileCurrency {
    pub created: NaiveDate,
    pub revised: Option<NaiveDate>,
    pub published: NaiveDate,
}

impl TileCurrency {
    /// Date the data describes: the revision date, else the creation date.
    pub fn currency(&self) -> NaiveDate {
        self.revised.unwrap_or(self.created)
    }
}

/// The grid the area of a raster record is stored on, in metres. A projected coordinate differs
/// in its last digits from one maths library to the next; on this grid the project file is the
/// same bytes on every system.
pub const STORED_METRE: f64 = 0.001;

/// `value` on the grid of `step`. Dividing by the whole number of steps in one unit gives the
/// number that is written with as few digits as the grid has; multiplying by the step would
/// leave `5816416.924000001` in the file.
pub fn on_grid(value: f64, step: f64) -> f64 {
    let per_unit = (1.0 / step).round();
    (value * per_unit).round() / per_unit
}

/// `bbox` as a raster record stores it: rounded to the millimetre.
pub fn stored_bbox(bbox: &BoundingBox) -> [f64; 4] {
    [bbox.min_e, bbox.min_n, bbox.max_e, bbox.max_n].map(|value| on_grid(value, STORED_METRE))
}

/// Whether the stored area of a raster record covers `bbox`. The rounding of [`stored_bbox`]
/// may leave it up to half a millimetre short, which counts as covered.
pub fn stored_bbox_covers(stored: [f64; 4], bbox: &BoundingBox) -> bool {
    let [min_e, min_n, max_e, max_n] = stored;
    min_e <= bbox.min_e + STORED_METRE
        && min_n <= bbox.min_n + STORED_METRE
        && max_e >= bbox.max_e - STORED_METRE
        && max_n >= bbox.max_n - STORED_METRE
}

/// The orthophoto fetched from a WMS for one area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WmsRecord {
    /// The flight date text the server gives, e.g. `19.03. + 07.04.2024`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flight: Option<String>,
    /// What the service calls the date in `flight` when it is no flight date as such
    /// (LVermGeo RP: `Aktualität (Erstellung)`, the date taken as the flight date).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flight_label: Option<String>,
    pub fetched: NaiveDate,
    /// Area in the map's CRS: min easting, min northing, max easting, max northing.
    pub bbox: [f64; 4],
}

impl WmsRecord {
    /// Whether the record's area covers `bbox`.
    pub fn covers(&self, bbox: &BoundingBox) -> bool {
        stored_bbox_covers(self.bbox, bbox)
    }
}

/// A height model fetched from a WCS for one area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WcsRecord {
    pub fetched: NaiveDate,
    /// Area in the map's CRS: min easting, min northing, max easting, max northing.
    pub bbox: [f64; 4],
    /// The date the provider gives for the data, e.g. `13.05.2024`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

impl WcsRecord {
    /// Whether the record's area covers `bbox`.
    pub fn covers(&self, bbox: &BoundingBox) -> bool {
        stored_bbox_covers(self.bbox, bbox)
    }
}

/// One fetched nDSM tile; LGB publishes no currency for it, only the server date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NdsmRecord {
    pub tile: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<NaiveDate>,
}

/// One fetched tile: where it came from and its data currency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileRecord {
    pub product: Product,
    pub tile: String,
    pub url: String,
    pub currency: NaiveDate,
    pub published: NaiveDate,
}

/// A part of a fetch: downloads freemapper loads or leaves out together. The order is the order
/// in which they are kept when space runs short.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Part {
    /// ALKIS buildings and land use, and an OSM extract not cached yet: the map objects.
    MapData,
    /// The terrain model: hillshade, slope and contours.
    Terrain,
    /// The orthophoto, the template the mapper draws on.
    Orthophoto,
    /// The extra Berlin layers (ALKIS structures, tree cadastre): candidates and a few map
    /// objects, optional like the K5 map.
    BerlinLayers,
    /// The state layers outside Berlin (full ALKIS from a NAS service, tree cadastres):
    /// candidates, optional like the Berlin layers.
    StateLayers,
    /// The surface model (nDOM, bDOM, DOM1): only the nDSM template.
    Surface,
    /// A newer OSM extract where an older one is cached; the older one serves meanwhile.
    OsmUpdate,
    /// The Berlin K5 map (1:5000), a template the mapper may draw on.
    K5,
}

impl Part {
    /// Whether a run cannot go on without the part: derive needs the terrain model and the
    /// orthophoto, and the map objects come from the map data.
    pub fn required(self) -> bool {
        matches!(self, Part::MapData | Part::Terrain | Part::Orthophoto)
    }

    pub fn label(self) -> &'static str {
        match self {
            Part::MapData => "the ALKIS and OSM map data",
            Part::Terrain => "the terrain model",
            Part::Orthophoto => "the orthophoto",
            Part::Surface => "the surface model for the nDSM template",
            Part::OsmUpdate => "the update of the cached OSM extract",
            Part::K5 => "the Berlin K5 map template",
            Part::BerlinLayers => {
                "the Berlin ALKIS structures, tree cadastre, street survey and point layers"
            }
            Part::StateLayers => "the full ALKIS structures and the tree cadastres",
        }
    }
}

/// Where a download is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    /// The cache shared by all projects.
    Cache,
    /// The project folder (`data/`).
    Project,
}

/// One download of a project.
#[derive(Debug, Clone, PartialEq)]
pub struct EstimateItem {
    pub what: String,
    pub megabytes: f64,
    pub cached: bool,
    /// The part of the fetch it belongs to, kept or left out as a whole when space runs short.
    pub part: Part,
    /// Where the download is stored.
    pub store: Store,
}

/// The id of a region adapter or of a provider: a lower-case letter followed by 1 to 31
/// lower-case letters or digits (`brandenburg`, `lvermgeosh`). The project file stores it, and
/// it becomes part of folder names, so nothing else is let through.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AdapterId(Cow<'static, str>);

/// A text that is no [`AdapterId`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is no adapter id: a lower-case letter, then 1 to 31 lower-case letters or digits")]
pub struct InvalidAdapterId(pub String);

/// Whether `id` has the form of an adapter id.
const fn is_adapter_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    if bytes.len() < 2 || bytes.len() > 32 || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    let mut index = 1;
    while index < bytes.len() {
        if !(bytes[index].is_ascii_lowercase() || bytes[index].is_ascii_digit()) {
            return false;
        }
        index += 1;
    }
    true
}

impl AdapterId {
    /// The id of an adapter built into the program. A constant with a malformed id does not
    /// compile.
    pub const fn built_in(id: &'static str) -> AdapterId {
        assert!(is_adapter_id(id), "not an adapter id");
        AdapterId(Cow::Borrowed(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for AdapterId {
    type Err = InvalidAdapterId;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        if is_adapter_id(id) {
            Ok(AdapterId(Cow::Owned(id.to_string())))
        } else {
            Err(InvalidAdapterId(id.to_string()))
        }
    }
}

impl AsRef<str> for AdapterId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// An id is written as its plain text: the project file names regions and providers so.
impl serde::Serialize for AdapterId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

/// Read from its plain text; a text that has not the form of an id is refused.
impl<'de> serde::Deserialize<'de> for AdapterId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for AdapterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_adapter_id_is_stored_as_its_text() {
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct Record {
            provider: Option<AdapterId>,
            regions: Vec<AdapterId>,
        }
        let record = Record {
            provider: Some(AdapterId::built_in("lgb")),
            regions: vec![
                AdapterId::built_in("brandenburg"),
                AdapterId::built_in("berlin"),
            ],
        };
        let text = serde_json::to_string(&record).unwrap();
        assert_eq!(
            text,
            r#"{"provider":"lgb","regions":["brandenburg","berlin"]}"#
        );
        assert_eq!(serde_json::from_str::<Record>(&text).unwrap(), record);
        let error = serde_json::from_str::<Record>(r#"{"provider":"Not An Id","regions":[]}"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("is no adapter id"), "{error}");
        assert_eq!(AdapterId::built_in("lgb").as_ref(), "lgb");
    }

    #[test]
    fn adapter_id_accepts_legacy_ids_and_rejects_others() {
        let legacy = [
            "brandenburg",
            "berlin",
            "saxony",
            "saxonyanhalt",
            "hamburg",
            "hessen",
            "schleswigholstein",
            "mecklenburgvorpommern",
            "nordrheinwestfalen",
            "niedersachsen",
            "rheinlandpfalz",
            "lgb",
            "geosn",
            "lvermgeo",
            "lgv",
            "hvbg",
            "lvermgeosh",
            "laivmv",
            "geobasisnrw",
            "lgln",
            "lvermgeorp",
        ];
        assert_eq!(legacy.len(), 21);
        for id in legacy {
            let parsed: AdapterId = id.parse().unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(parsed.as_str(), id);
            assert_eq!(parsed, AdapterId::built_in(id));
            assert_eq!(parsed.to_string(), id);
        }
        let too_long = "a".repeat(33);
        for id in [
            "",
            "A",
            "a",
            "a b",
            "../x",
            "1a",
            "aB",
            "a-b",
            "bär",
            too_long.as_str(),
        ] {
            let refused = id.parse::<AdapterId>().unwrap_err();
            assert!(refused.to_string().contains("no adapter id"), "{refused}");
        }
        assert!("a".repeat(32).parse::<AdapterId>().is_ok());
    }
}
