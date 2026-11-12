//! A checked manifest: what the tables of format 1 say, in types the builders can use.

use chrono::NaiveDate;
use freemapper_georef::crs::Crs;

/// An adapter manifest of format 1 that passed the validator.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub adapter: Adapter,
    /// `[net] hosts`: the hosts the URLs of the manifest may use, in lower case.
    pub hosts: Vec<String>,
    pub boundary: Boundary,
    /// `[[osm.extract]]`, at least one.
    pub osm: Vec<OsmExtract>,
    pub attribution: Attribution,
    pub alkis: Option<Alkis>,
    pub orthophoto: Option<Orthophoto>,
    pub terrain: Option<HeightModel>,
    pub surface: Option<HeightModel>,
    /// `[[extra.nas]]`.
    pub nas: Vec<Nas>,
    /// `[[extra.trees]]`.
    pub trees: Vec<Trees>,
    pub preview: Option<Preview>,
    pub estimate: Estimate,
}

/// A version `x.y.z` of freemapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u32, pub u32, pub u32);

impl Version {
    /// Reads `x.y.z`: three plain numbers.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('.').map(|part| {
            (!part.is_empty() && part.len() <= 9 && part.bytes().all(|b| b.is_ascii_digit()))
                .then(|| part.parse::<u32>().ok())
                .flatten()
        });
        let version = Self(parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(version)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// `[adapter]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adapter {
    pub id: String,
    /// The provider of the rasters; the id of the adapter when the manifest names none.
    pub provider: String,
    /// Counts the data changes of the adapter.
    pub revision: u32,
    /// The oldest freemapper that reads the manifest.
    pub min_freemapper: Version,
    pub names: Names,
    /// The short name of who serves the data, in messages; the English name of the region
    /// when the manifest gives none.
    pub agency: String,
    /// The CRS of a map whose centre lies in the region.
    pub crs: Crs,
    pub object_id_prefix: Option<String>,
    pub maintainers: Vec<String>,
    pub last_verified: Option<NaiveDate>,
    pub licence_of_manifest: Option<String>,
}

/// The name of the region; German falls back to English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Names {
    pub en: String,
    pub de: String,
}

/// The CRS of a boundary file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryCrs {
    /// EPSG:4326, longitude and latitude.
    Wgs84,
    /// EPSG:25833.
    Utm33,
}

/// `[boundary]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Boundary {
    /// A plain file name in the folder of the manifest.
    pub file: String,
    pub crs: BoundaryCrs,
    /// How far the boundary was simplified, in metres.
    pub simplify_m: f64,
}

/// One `[[osm.extract]]`.
#[derive(Debug, Clone, PartialEq)]
pub struct OsmExtract {
    /// The path on the Geofabrik server (`europe/germany/hessen`).
    pub geofabrik: String,
    pub megabytes: f64,
    pub label: String,
    /// The boundary file of the extract, when the adapter has several extracts.
    pub boundary: Option<String>,
    /// What the name of the cached file starts with, without the `_`.
    pub cache_prefix: String,
}

/// The licences a credit may name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Licence {
    DlDeZero20,
    DlDeBy20,
    CcBy40,
    Odbl,
    Cc010,
    /// Another licence; the manifest gives its text.
    Other,
}

impl Licence {
    pub const ALL: [Self; 6] = [
        Self::DlDeZero20,
        Self::DlDeBy20,
        Self::CcBy40,
        Self::Odbl,
        Self::Cc010,
        Self::Other,
    ];

    /// The id of the licence in a manifest.
    pub fn id(self) -> &'static str {
        match self {
            Self::DlDeZero20 => "dl-de/zero-2-0",
            Self::DlDeBy20 => "dl-de/by-2-0",
            Self::CcBy40 => "cc-by-4.0",
            Self::Odbl => "odbl",
            Self::Cc010 => "cc0-1.0",
            Self::Other => "other",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|licence| licence.id() == id)
    }
}

/// `[attribution]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribution {
    pub licence: Licence,
    /// The text of the licence, for [`Licence::Other`] only.
    pub licence_text: Option<String>,
    /// The credit line; `{year}` stands for the year of the newest fetch.
    pub text: String,
    /// A short credit for where the full one has no room.
    pub short: Option<String>,
    /// A line about the licence for the notes of the project (`ATTRIBUTION.txt`).
    pub note: Option<String>,
}

/// A WFS with "ALKIS vereinfacht".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wfs {
    /// The name of the service in messages.
    pub name: String,
    pub url: String,
    pub buildings_type: String,
    pub land_use_type: String,
    pub crs: Crs,
    /// `bez` given to squares without one.
    pub square_bez: Option<String>,
    /// Whether the service takes `SORTBY`; `sorted = false` in the manifest says it does not.
    pub sorted: bool,
}

/// The presets that turn the features of a service into the LGB spellings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Normalise {
    Lgb,
    HamburgAscii,
}

impl Normalise {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "lgb" => Some(Self::Lgb),
            "hamburg-ascii" => Some(Self::HamburgAscii),
            _ => None,
        }
    }
}

/// `[alkis]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Alkis {
    Wfs(Wfs),
    OgcApi {
        /// The items endpoint of the buildings collection.
        buildings: String,
        /// The items endpoint of the land use collection.
        land_use: String,
        crs: Crs,
        normalise: Normalise,
        /// The WFS asked when the API fails, in the CRS of the API.
        fallback: Option<Wfs>,
    },
}

/// How the date of a raster is found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateSource {
    /// The service tells no date.
    None,
    /// A WFS that answers GeoJSON: the `field` of the feature of `layer` at the centre of the
    /// area, written as `format` says.
    WfsGeoJson {
        url: String,
        layer: String,
        field: String,
        format: DateFormat,
    },
    /// A STAC API: the newest item at the centre of the area.
    StacNewest { url: String },
    /// GetFeatureInfo on a layer of the orthophoto's WMS.
    WmsFeatureInfo { layer: String, field: String },
}

/// How a service writes a date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFormat {
    /// `29.04.2026`.
    DayMonthYear,
    /// `2026-04-29`.
    Iso,
}

/// `[orthophoto]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Orthophoto {
    Wms {
        /// What the raster is called in the estimate and in messages.
        label: String,
        url: String,
        layer: String,
        crs: Crs,
        /// More GetMap parameters, by name.
        params: Vec<(String, String)>,
        /// The largest side of one request in pixels.
        max_px: u32,
        date: DateSource,
    },
    Wcs {
        /// What the raster is called in the estimate and in messages.
        label: String,
        url: String,
        coverage: String,
        crs: Crs,
        max_px: u32,
        date: DateSource,
    },
    /// Code of an adapter crate, by name.
    Backend { name: String },
}

/// `[terrain]` and `[surface]`.
// A manifest holds two of these at most; a box would save nothing.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum HeightModel {
    Wcs {
        /// What the model is called in the estimate and in messages.
        label: String,
        url: String,
        coverage: String,
        /// More GetCoverage parameters, by name.
        params: Vec<(String, String)>,
        /// The TIFF comes in a `multipart/related` answer.
        multipart: bool,
        /// The extent of the coverage in the CRS of the adapter: `[min_e, min_n, max_e, max_n]`.
        envelope: [f64; 4],
        /// The largest side of one request in metres.
        max_m: f64,
        date: DateSource,
        /// `[terrain]` only: the name of the model in `freemapper:source` of the contours.
        source_label: Option<String>,
    },
    /// Code of an adapter crate, by name.
    Backend { name: String },
}

/// One `[[extra.nas]]`: a WFS with the full ALKIS (NAS).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nas {
    pub name: String,
    pub url: String,
    pub crs: Crs,
    /// Features per request.
    pub max_count: u32,
    /// The largest answer read, in bytes.
    pub max_bytes: u64,
    /// The layers the service serves; `None`: all.
    pub layers: Option<Vec<String>>,
    pub namespaces: Option<String>,
}

/// One `[[extra.trees]]`: the tree cadastre of a city.
#[derive(Debug, Clone, PartialEq)]
pub struct Trees {
    pub id: String,
    pub name: String,
    pub crs: Crs,
    /// The extent of the data in `crs`: `[min_e, min_n, max_e, max_n]`.
    pub envelope: [f64; 4],
    /// The field with the position (format `ods-geojson-export`).
    pub geo_field: String,
    pub endpoints: Vec<String>,
    pub fields: TreeFields,
    pub skipped_kinds: Vec<String>,
    pub attribution: String,
    pub max_bytes: u64,
}

/// The field names of a tree cadastre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeFields {
    pub kind: String,
    pub id: String,
    pub date: String,
    pub species: Option<String>,
    pub crown: Option<String>,
}

/// `[preview]`: the aerial photo for the map of the area page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    pub wms: String,
    pub layer: String,
    /// `image/png` or `image/jpeg`.
    pub format: String,
    pub params: Vec<(String, String)>,
    /// The credit on the map; `{year}` stands for the current year.
    pub attribution: String,
    /// The short name of the agency, for the name of the layer.
    pub agency: String,
}

/// `[estimate]`: megabytes per km² where the adapter knows better than the defaults.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Estimate {
    pub orthophoto_mb_per_km2: Option<f64>,
    pub terrain_mb_per_km2: Option<f64>,
    pub surface_mb_per_km2: Option<f64>,
}
