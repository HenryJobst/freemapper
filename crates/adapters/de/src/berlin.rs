//! ALKIS buildings and land use from the Geoportal Berlin WFS (dl-de/zero-2-0).
//!
//! Features are normalised into the LGB property schema (`oid`, `gebnutzbez`, `funktion`,
//! `rellage`, `nutzart`, `bez`), so the stored files and the build rules stay the same for both
//! states.

use std::borrow::Cow;
use std::collections::HashSet;

use chrono::{Datelike, NaiveDate};
use freemapper_adapter_api::extras::{
    ExtraEstimate, ExtraLayerInfo, ExtraNotes, ExtraSet, ExtraSetInfo, FetchedExtra,
    RasterTemplate, RasterTemplateInfo, TemplateNotes,
};
use serde_json::{Map, Value};

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, Fetcher, get_with_retries};
use crate::project::MapKind;
use crate::raster::GridSpec;
use crate::sources::alkis::invalid;
use crate::sources::region::{BERLIN, touches};
use crate::sources::wms::WmsService;

pub const BUILDINGS_SERVICE: &str = "https://gdi.berlin.de/services/wfs/alkis_gebaeude";
pub const BUILDINGS_TYPE: &str = "alkis_gebaeude:gebaeude";
pub const LAND_USE_SERVICE: &str = "https://gdi.berlin.de/services/wfs/alkis";
pub const LAND_USE_TYPE: &str = "alkis:tatsaechlichenutzungflaechen";
/// The tree registers (street trees, park trees): the stored files. They have a credit line of
/// their own.
pub const TREE_REGISTERS: [&str; 2] = ["strassenbaeume", "anlagenbaeume"];
/// The lighting cadastre and the fountain registers: the stored file and the name in the
/// credit line, in the order of the line.
pub const BERLIN_REGISTERS: [(&str, &str); 3] = [
    ("beleuchtung", "Öffentliche Beleuchtung"),
    ("zierbrunnen", "Zierbrunnen"),
    ("trinkwasserbrunnen", "Trinkwasserbrunnen"),
];

/// Whether the layer stored as `file` is a register: one of [`TREE_REGISTERS`] or of
/// [`BERLIN_REGISTERS`]. A register knows its objects better than the cadastre does.
pub fn is_register(file: &str) -> bool {
    TREE_REGISTERS.contains(&file) || BERLIN_REGISTERS.iter().any(|&(name, _)| name == file)
}
/// Features per page; a 1 km tile in the city centre holds about 1100 buildings.
const PAGE_COUNT: u32 = 5000;
/// Property that holds the WFS type of a stored feature (not `layer`: a service may have one).
pub const TYPE_KEY: &str = "freemapper:layer";

/// The date the survey layers get when a `gis_id` has none (the survey ran 2014–2016).
pub const SURVEY_DATASET_DATE: NaiveDate = NaiveDate::from_ymd_opt(2014, 1, 1).expect("a date");

/// `bez` of a Berlin square: Berlin never records what a square is used for, so it stays a
/// candidate instead of becoming a paved map object.
pub const SQUARE_WITHOUT_FUNCTION: &str = "ohne Unterart (Berlin)";

/// AAA class (`bezeich`) → LGB `nutzart`.
const LAND_USE_CLASSES: [(&str, &str); 19] = [
    ("AX_Wohnbauflaeche", "Wohnbaufläche"),
    (
        "AX_IndustrieUndGewerbeflaeche",
        "Industrie- und Gewerbefläche",
    ),
    ("AX_FlaecheGemischterNutzung", "Fläche gemischter Nutzung"),
    (
        "AX_FlaecheBesondererFunktionalerPraegung",
        "Fläche besonderer funktionaler Prägung",
    ),
    (
        "AX_SportFreizeitUndErholungsflaeche",
        "Sport-, Freizeit- und Erholungsfläche",
    ),
    ("AX_Friedhof", "Friedhof"),
    ("AX_Strassenverkehr", "Straßenverkehr"),
    ("AX_Weg", "Weg"),
    ("AX_Platz", "Platz"),
    ("AX_Bahnverkehr", "Bahnverkehr"),
    ("AX_Flugverkehr", "Flugverkehr"),
    ("AX_Schiffsverkehr", "Schiffsverkehr"),
    ("AX_Wald", "Wald"),
    ("AX_Gehoelz", "Gehölz"),
    ("AX_Landwirtschaft", "Landwirtschaft"),
    ("AX_Moor", "Moor"),
    ("AX_Fliessgewaesser", "Fließgewässer"),
    ("AX_StehendesGewaesser", "Stehendes Gewässer"),
    ("AX_Hafenbecken", "Hafenbecken"),
];

/// One WFS feature type of an [`ExtraLayer`] and the map kinds that use it.
#[derive(Debug, PartialEq)]
pub struct LayerType {
    /// `namespace:name` as the service knows it; the part after the colon is the `berlin:layer`
    /// value of a layer with `by_type` set.
    pub name: &'static str,
    pub kinds: &'static [MapKind],
}

/// One extra Berlin layer: one or more WFS feature types of one service, stored together as
/// `data/berlin/<file>.geojson`.
#[derive(Debug, PartialEq)]
pub struct ExtraLayer {
    /// Stored file name without the extension; also the `berlin:layer` value of its candidates
    /// unless `by_type` is set.
    pub file: &'static str,
    pub service: &'static str,
    /// The feature types fetched one by one into the file, in this order.
    pub types: &'static [LayerType],
    /// Each stored feature gets `layer` = the type name without its namespace, as the
    /// `berlin:layer` value (the types of the file cannot be told apart otherwise).
    pub by_type: bool,
    /// The features carry a `gis_id` with their survey date (Straßenbefahrung).
    pub survey: bool,
    /// Lamps not in operation are dropped on download.
    pub operating_only: bool,
}

// A forest map takes what a sprint map takes (spec 2026-11-13).
const BOTH_KINDS: &[MapKind] = &[MapKind::Sprint, MapKind::Schoolyard, MapKind::Forest];
const SPRINT_ONLY: &[MapKind] = &[MapKind::Sprint, MapKind::Forest];
const SCHOOLYARD_ONLY: &[MapKind] = &[MapKind::Schoolyard];
/// A forest map draws no street or park trees of the register, no street furniture and no lamps
/// (spec 2026-11-13).
const TREE_KINDS: &[MapKind] = &[MapKind::Sprint, MapKind::Schoolyard];
const FURNITURE_KINDS: &[MapKind] = &[MapKind::Sprint, MapKind::Schoolyard];
const SPRINT_FURNITURE: &[MapKind] = &[MapKind::Sprint];
/// The survey's paving: only the sprint map draws paved areas from the street parcels.
const SPRINT_PAVING: &[MapKind] = &[MapKind::Sprint];
const TREE_SERVICE: &str = "https://gdi.berlin.de/services/wfs/baumbestand";
/// The street survey of 2014–2016 (Straßenbefahrung).
const SURVEY_SERVICE: &str = "https://gdi.berlin.de/services/wfs/strassenbefahrung";

/// A layer of one plus-1 type for both kinds.
const fn plain(
    file: &'static str,
    service: &'static str,
    types: &'static [LayerType],
) -> ExtraLayer {
    ExtraLayer {
        file,
        service,
        types,
        by_type: false,
        survey: false,
        operating_only: false,
    }
}

/// A Straßenbefahrung layer.
const fn survey(file: &'static str, types: &'static [LayerType]) -> ExtraLayer {
    ExtraLayer {
        file,
        service: SURVEY_SERVICE,
        types,
        by_type: true,
        survey: true,
        operating_only: false,
    }
}

const fn ty(name: &'static str, kinds: &'static [MapKind]) -> LayerType {
    LayerType { name, kinds }
}

/// The extra layers, in fetch order: ALKIS structures, relief and hedges, tree cadastre (Berlin
/// plus 1), then the street survey, lighting, fountains and building heights (Berlin plus 2).
pub const EXTRA_LAYERS: [ExtraLayer; 18] = [
    plain(
        "bauwerke-flaechen",
        LAND_USE_SERVICE,
        &[ty("alkis:bauwerkeflaechen", BOTH_KINDS)],
    ),
    plain(
        "bauwerke-linien",
        LAND_USE_SERVICE,
        &[ty("alkis:bauwerkelinien", BOTH_KINDS)],
    ),
    plain(
        "relief",
        LAND_USE_SERVICE,
        &[ty("alkis:relief", SPRINT_ONLY)],
    ),
    plain(
        "vegetation-linien",
        LAND_USE_SERVICE,
        &[ty("alkis:gewaesservegetationlinien", BOTH_KINDS)],
    ),
    plain(
        "strassenbaeume",
        TREE_SERVICE,
        &[ty("baumbestand:strassenbaeume", TREE_KINDS)],
    ),
    plain(
        "anlagenbaeume",
        TREE_SERVICE,
        &[ty("baumbestand:anlagenbaeume", TREE_KINDS)],
    ),
    survey(
        "sb-gruenflaeche",
        &[ty("strassenbefahrung:ce_gruenflaeche", SPRINT_PAVING)],
    ),
    survey(
        "sb-flaechen",
        &[
            ty("strassenbefahrung:cm_fahrbahn", SPRINT_PAVING),
            ty("strassenbefahrung:cl_gehweg", SPRINT_PAVING),
            ty("strassenbefahrung:ch_radweg", SPRINT_PAVING),
            ty("strassenbefahrung:ck_parkflaeche", SPRINT_PAVING),
            ty("strassenbefahrung:ci_oeffentlicher_platz", SPRINT_PAVING),
            ty("strassenbefahrung:cj_fussgaengerzone", SPRINT_PAVING),
            ty("strassenbefahrung:by_gehwegueberfahrt", SPRINT_PAVING),
        ],
    ),
    survey(
        "sb-bordstein",
        &[ty("strassenbefahrung:bd_bordstein", SPRINT_PAVING)],
    ),
    survey(
        "sb-treppe",
        &[ty("strassenbefahrung:cc_treppe", BOTH_KINDS)],
    ),
    survey("sb-mauer", &[ty("strassenbefahrung:bh_mauer", BOTH_KINDS)]),
    survey(
        "sb-gelaender",
        &[
            ty("strassenbefahrung:bf_gelaender", BOTH_KINDS),
            ty("strassenbefahrung:bb_verkehrsschutzgitter", BOTH_KINDS),
            ty("strassenbefahrung:bg_leitplanke", BOTH_KINDS),
        ],
    ),
    survey(
        "sb-moebel",
        &[
            ty("strassenbefahrung:bj_sitzbank", FURNITURE_KINDS),
            ty("strassenbefahrung:aq_hydrant", SCHOOLYARD_ONLY),
            ty("strassenbefahrung:av_poller", FURNITURE_KINDS),
            ty("strassenbefahrung:as_mast", FURNITURE_KINDS),
            ty("strassenbefahrung:am_fahnenmast", FURNITURE_KINDS),
            ty("strassenbefahrung:aw_spielgeraet", SCHOOLYARD_ONLY),
        ],
    ),
    survey(
        "sb-objekte",
        &[
            ty("strassenbefahrung:ag_werbesaeule", SPRINT_FURNITURE),
            ty("strassenbefahrung:br_fahrgastunterstand", BOTH_KINDS),
            ty("strassenbefahrung:af_wasserpumpen_brunnen", BOTH_KINDS),
            ty("strassenbefahrung:bv_springbrunnen_zierbrunnen", BOTH_KINDS),
            ty(
                "strassenbefahrung:ac_trinkwasserbrunnen_wasserspender",
                BOTH_KINDS,
            ),
        ],
    ),
    ExtraLayer {
        by_type: true,
        operating_only: true,
        ..plain(
            "beleuchtung",
            "https://gdi.berlin.de/services/wfs/beleuchtung",
            &[ty("beleuchtung:beleuchtung", FURNITURE_KINDS)],
        )
    },
    ExtraLayer {
        by_type: true,
        ..plain(
            "zierbrunnen",
            "https://gdi.berlin.de/services/wfs/zierbrunnen",
            &[ty("zierbrunnen:bez_zierbrunnen", BOTH_KINDS)],
        )
    },
    ExtraLayer {
        by_type: true,
        ..plain(
            "trinkwasserbrunnen",
            "https://gdi.berlin.de/services/wfs/trinkwasserbrunnen",
            &[ty("trinkwasserbrunnen:trinkwasserbrunnen", BOTH_KINDS)],
        )
    },
    plain(
        "gebaeudehoehen",
        "https://gdi.berlin.de/services/wfs/ua_gebaeudehoehen",
        &[ty("ua_gebaeudehoehen:gebaeudehoehen", BOTH_KINDS)],
    ),
];

impl ExtraLayer {
    /// The feature types `kind` uses.
    pub fn types_for(&self, kind: MapKind) -> impl Iterator<Item = &'static LayerType> {
        self.types.iter().filter(move |t| t.kinds.contains(&kind))
    }
}

/// The extra layers `kind` uses.
pub fn extra_layers(kind: MapKind) -> impl Iterator<Item = &'static ExtraLayer> {
    EXTRA_LAYERS
        .iter()
        .filter(move |l| l.types_for(kind).next().is_some())
}

/// Berlin's registers, surveys and point layers as the set of extra layers of its adapter.
pub struct Registers;

/// What the engine stores and reports of [`Registers`].
static REGISTERS: ExtraSetInfo = ExtraSetInfo {
    key: "berlin",
    crs: crate::crs::Crs::Utm33,
    notes: ExtraNotes {
        none_stored: "progress-berlin-none-stored",
        removing: "progress-berlin-removing",
        download: "progress-berlin-download",
        features: "progress-berlin-features",
        kept: "warning-berlin-layer-kept",
        dropped: "warning-berlin-layer-skipped",
    },
    // The vector layers in the centre, per km²: the ALKIS structure layers, relief, hedges and
    // the tree cadastre 4 MB (measured 3.7 to 3.8 MB in Kreuzberg and 4.6 MB in Prenzlauer Berg),
    // the street survey and point layers 5 MB (measured 3.2 MB at Viktoriapark, plus 0.8 MB per
    // km² there for the furniture and lamps that sprint maps fetch too since ADR 0077; a street
    // area with 2 392 bollards per km² takes about 1.4 MB) and the building heights 3 MB
    // (measured 2.7 MB there), live runs of 2026-11-09 and 2026-11-10. Each share is rounded up
    // to the next half MB, so the disk check errs on the safe side; Viktoriapark measured
    // 9.8 MB in all, before the furniture.
    estimate: ExtraEstimate {
        what: "Berlin ALKIS structures, tree cadastre, street survey and point layers",
        megabytes_per_km2: 12.0,
    },
};

impl ExtraSet for Registers {
    fn info(&self) -> &ExtraSetInfo {
        &REGISTERS
    }

    fn layers(&self, kind: MapKind) -> Vec<ExtraLayerInfo> {
        extra_layers(kind)
            .map(|layer| ExtraLayerInfo { key: layer.file })
            .collect()
    }

    fn has(&self, key: &str) -> bool {
        EXTRA_LAYERS.iter().any(|layer| layer.file == key)
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        region_bbox: &BoundingBox,
        kind: MapKind,
    ) -> Vec<FetchedExtra> {
        fetch_extra_layers(fetcher, region_bbox, kind)
            .into_iter()
            .map(|fetched| FetchedExtra {
                key: fetched.layer.file,
                features: fetched.features,
            })
            .collect()
    }
}

/// The Berlin 1:5000 map in colour (K5), template only: PNG pieces, not an orthophoto.
pub const K5_WMS: WmsService = WmsService {
    url: Cow::Borrowed("https://gdi.berlin.de/services/wms/k5_farbe"),
    layer: Cow::Borrowed("k5_farbe"),
    crs: crate::crs::Crs::Utm33,
    extra: Cow::Borrowed(""),
    max_px: 4000,
};
/// Pixel size of the K5 template on both map kinds: a 1:5000 map is fully resolved at this.
pub const K5_PIXEL_M: f64 = 0.25;
/// Longest side of the K5 image in pixels (about 108 MB while stitching); a larger area gets
/// coarser pixels.
const K5_MAX_SIDE_PX: f64 = 6_000.0;

/// The GetMap requests covering `grid` for the K5 map (PNG).
pub fn k5_pieces(grid: &GridSpec) -> Vec<(GridSpec, String)> {
    crate::sources::wms::png_pieces(&K5_WMS, grid)
}

/// Downloads the K5 map over `bbox` (EPSG:25833) from the Berlin WMS, stitched at `pixel_size`
/// (made coarser when the image would pass [`K5_MAX_SIDE_PX`]), as `out` (PNG) with its world
/// file (`.pgw`), on the template grid of `bbox`.
pub fn fetch_k5(
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
    pixel_size: f64,
    root: &std::path::Path,
    out: &std::path::Path,
) -> Result<(), FetchError> {
    crate::sources::wms::fetch_png_template(
        fetcher,
        &K5_WMS,
        bbox,
        pixel_size,
        K5_MAX_SIDE_PX,
        root,
        out,
    )
}

/// The Berlin map 1:5000 in colour (K5) as the raster template of Berlin's adapter.
pub struct K5;

/// The key of the [`K5`] template: its file, its record and its credit go by it.
pub const K5_KEY: &str = "k5";

/// What the engine stores and reports of [`K5`].
static K5_TEMPLATE: RasterTemplateInfo = RasterTemplateInfo {
    key: K5_KEY,
    crs: crate::crs::Crs::Utm33,
    // The date of the K5 dataset (catalogue `karte-von-berlin-1-5000-k5-farbausgabe-9f3ecf8d`);
    // the WMS gives no date of its own.
    dataset_date: "2026-01-02",
    dataset_label: "K5 dataset",
    notes: TemplateNotes {
        unchanged: "progress-k-five-unchanged",
        download: "progress-k-five-download",
        unavailable: "warning-k-five-unavailable",
    },
    // The PNG at 0.25 m per pixel, per km² (measured 9.1 MB around a park and 17 MB in dense
    // blocks, 2026-11-09).
    estimate: ExtraEstimate {
        what: "Berlin K5 map (1:5000) template",
        megabytes_per_km2: 15.0,
    },
};

impl RasterTemplate for K5 {
    fn info(&self) -> &RasterTemplateInfo {
        &K5_TEMPLATE
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &BoundingBox,
        root: &std::path::Path,
        out: &std::path::Path,
    ) -> Result<(), FetchError> {
        fetch_k5(fetcher, bbox, K5_PIXEL_M, root, out)
    }
}

/// One extra layer as fetched; an empty `features` is a layer without objects in the area.
#[derive(Debug)]
pub struct FetchedLayer {
    pub layer: &'static ExtraLayer,
    /// The features as served (EPSG:25833), or why the layer could not be fetched.
    pub features: Result<Vec<Value>, FetchError>,
}

/// Whether `region_bbox` (EPSG:25833) needs the Berlin services: the test of the Berlin ALKIS.
pub fn near_berlin(region_bbox: &BoundingBox) -> bool {
    touches(BERLIN, region_bbox)
}

/// The extra layers of `kind` in `region_bbox` (EPSG:25833), one request series per layer; none
/// at all, and no request, when the area is not near Berlin. A layer that fails does not stop
/// the others.
pub fn fetch_extra_layers(
    fetcher: &dyn Fetcher,
    region_bbox: &BoundingBox,
    kind: MapKind,
) -> Vec<FetchedLayer> {
    if !near_berlin(region_bbox) {
        return Vec::new();
    }
    extra_layers(kind)
        .map(|layer| FetchedLayer {
            layer,
            features: fetch_layer(fetcher, layer, region_bbox, kind),
        })
        .collect()
}

/// All features of `layer` for `kind`, the types one after the other. One type that fails fails
/// the layer (a half-filled file would pass for complete). Stored features are prepared as the
/// layer says: the type name, the survey date, MultiPoints split, lamps out of service dropped.
fn fetch_layer(
    fetcher: &dyn Fetcher,
    layer: &ExtraLayer,
    region_bbox: &BoundingBox,
    kind: MapKind,
) -> Result<Vec<Value>, FetchError> {
    let mut features = Vec::new();
    for t in layer.types_for(kind) {
        for mut feature in fetch_wfs(fetcher, layer.service, t.name, region_bbox)? {
            if layer.operating_only && !in_operation(&feature) {
                continue;
            }
            if layer.by_type {
                let local = t.name.split_once(':').map_or(t.name, |(_, name)| name);
                feature["properties"][TYPE_KEY] = local.into();
            }
            if layer.survey {
                stamp_survey_date(&mut feature);
            }
            features.extend(split_multipoint(feature));
        }
    }
    Ok(features)
}

/// Whether a lighting cadastre feature is in operation (`In Betrieb`, `In Betrieb (Provi)`).
fn in_operation(feature: &Value) -> bool {
    feature["properties"]["status"]
        .as_str()
        .is_some_and(|status| status.starts_with("In Betrieb"))
}

/// The date of the survey as `survey_date`, taken from `gis_id`; without a usable id the date of
/// the dataset and `survey_date_guessed` = `yes`.
pub fn stamp_survey_date(feature: &mut Value) {
    let parsed = feature["properties"]["gis_id"]
        .as_str()
        .and_then(survey_date);
    let date = parsed.unwrap_or(SURVEY_DATASET_DATE);
    feature["properties"]["survey_date"] = date.to_string().into();
    if parsed.is_none() {
        feature["properties"]["survey_date_guessed"] = "yes".into();
    }
}

/// The date in a Straßenbefahrung `gis_id` (`Fl_20150413_37`, `Li_25.09.2014_230`,
/// `Fl_2015-05-02_404949`, `FL_16072014_03142`): the part between the first two underscores.
/// Compact dates are YYYYMMDD, or DDMMYYYY when the first four digits are not a year of the
/// survey. `None` for anything else, and for years outside 2010–2020.
pub fn survey_date(gis_id: &str) -> Option<NaiveDate> {
    let text = gis_id.split('_').nth(1)?;
    let date = if text.contains('.') {
        NaiveDate::parse_from_str(text, "%d.%m.%Y").ok()?
    } else if text.contains('-') {
        NaiveDate::parse_from_str(text, "%Y-%m-%d").ok()?
    } else if text.len() == 8 && text.bytes().all(|b| b.is_ascii_digit()) {
        // YYYYMMDD when it is a date of the survey years, else DDMMYYYY.
        NaiveDate::parse_from_str(text, "%Y%m%d")
            .ok()
            .filter(|d| plausible_survey_year(d.year()))
            .or_else(|| NaiveDate::parse_from_str(text, "%d%m%Y").ok())?
    } else {
        return None;
    };
    plausible_survey_year(date.year()).then_some(date)
}

fn plausible_survey_year(year: i32) -> bool {
    (2010..=2020).contains(&year)
}

/// A MultiPoint feature as one Point feature per member (`id`, then `id#2`, `id#3`, …), all
/// with the properties of the original; any other feature unchanged. An empty MultiPoint gives
/// none.
pub fn split_multipoint(feature: Value) -> Vec<Value> {
    if feature["geometry"]["type"] != "MultiPoint" {
        return vec![feature];
    }
    let members = feature["geometry"]["coordinates"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    members
        .into_iter()
        .enumerate()
        .map(|(i, coordinates)| {
            let mut point = feature.clone();
            point["geometry"] = serde_json::json!({ "type": "Point", "coordinates": coordinates });
            if i > 0
                && let Some(id) = feature["id"].as_str()
            {
                point["id"] = format!("{id}#{}", i + 1).into();
            }
            if let Some(object) = point.as_object_mut() {
                object.remove("bbox");
            }
            point
        })
        .collect()
}

// A stored layer is read and written by `freemapper-sources`, for every family of layers.
pub use freemapper_sources::layer_file::{layer_to_geojson, parse_layer};

/// URL of the first page of `type_name` features of `service` in `bbox`, rounded outward.
pub fn wfs_url(service: &str, type_name: &str, bbox: &BoundingBox) -> String {
    format!(
        "{service}?service=WFS&version=2.0.0&request=GetFeature&typeNames={type_name}\
         &outputFormat=application/json&count={PAGE_COUNT}&bbox={},{},{},{},urn:ogc:def:crs:EPSG::25833",
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil()
    )
}

/// All features of `type_name` in `bbox`, following `links[rel=next]`; a feature repeated on a
/// later page (same GeoServer `id`; not every layer has a `uuid`) is kept once.
pub fn fetch_wfs(
    fetcher: &dyn Fetcher,
    service: &str,
    type_name: &str,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    let mut features = Vec::new();
    let mut seen = HashSet::new();
    let mut visited = Vec::new();
    let mut next = Some(wfs_url(service, type_name, bbox));
    while let Some(url) = next.take() {
        let page = fetch_page(fetcher, &url)?;
        let page_features = page["features"].as_array().cloned().unwrap_or_default();
        let empty = page_features.is_empty();
        for feature in page_features {
            let new = match feature["id"].as_str() {
                Some(id) => seen.insert(id.to_string()),
                None => true,
            };
            if new {
                features.push(feature);
            }
        }
        visited.push(url);
        if empty {
            break;
        }
        next = page["links"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|link| link["rel"] == "next")
            .and_then(|link| link["href"].as_str())
            .map(str::to_string)
            .filter(|href| !visited.contains(href));
    }
    Ok(features)
}

/// One page as a GeoJSON FeatureCollection. Server errors, resets and answers that are not a
/// FeatureCollection (GeoServer sends exception reports as XML with status 200) are retried;
/// a missing service is not.
fn fetch_page(fetcher: &dyn Fetcher, url: &str) -> Result<Value, FetchError> {
    get_with_retries(fetcher, url, |bytes| {
        serde_json::from_slice::<Value>(bytes)
            .ok()
            .filter(|page| page["type"] == "FeatureCollection" && page["features"].is_array())
            .ok_or_else(|| {
                let start = String::from_utf8_lossy(&bytes[..bytes.len().min(80)]).into_owned();
                FetchError::Alkis(invalid(format!(
                    "{url}: not a GeoJSON FeatureCollection: {start}"
                )))
            })
    })
}

/// A Berlin building feature with LGB properties.
pub fn normalise_building(feature: &Value) -> Value {
    let source = &feature["properties"];
    let kind = match source["bezeich"].as_str() {
        Some("AX_Gebaeude") => "Gebäude",
        Some("AX_Bauteil") => "Bauteil",
        Some(other) => other,
        None => "",
    };
    let mut properties = Map::new();
    properties.insert("oid".into(), source["uuid"].clone());
    properties.insert("gebnutzbez".into(), kind.into());
    properties.insert("funktion".into(), source["bezgfk"].clone());
    properties.insert("rellage".into(), source["bezofl"].clone());
    // The kind of a building part (`Durchfahrt im Gebäude`, `Arkade`, …); the build ignores it,
    // the Berlin rules read it.
    if let Some(bauart) = source["bezbat"].as_str() {
        properties.insert("bauart".into(), bauart.into());
    }
    with_properties(feature, properties)
}

/// Sets `aktualit` of the normalised `buildings` from the footprint date
/// (`grundrissaktualitaet`, 2022 data of the Umweltatlas) of the `heights` feature with
/// `gml_id` = the building's `oid`. A building without a match keeps no date.
pub fn join_footprint_dates(buildings: &mut [Value], heights: &[Value]) {
    let dates: std::collections::HashMap<&str, &str> = heights
        .iter()
        .filter_map(|f| {
            let p = &f["properties"];
            let date = p["grundrissaktualitaet"].as_str()?;
            // Only a real date counts; the layer also holds empty strings.
            date.get(..10)?.parse::<NaiveDate>().ok()?;
            Some((p["gml_id"].as_str()?, date))
        })
        .collect();
    for building in buildings {
        let date = building["properties"]["oid"]
            .as_str()
            .and_then(|oid| dates.get(oid));
        if let Some(date) = date.copied() {
            building["properties"]["aktualit"] = date.into();
        }
    }
}

/// A Berlin land use feature with LGB properties.
pub fn normalise_land_use(feature: &Value) -> Value {
    let source = &feature["properties"];
    let class = source["bezeich"].as_str().unwrap_or_default();
    let nutzart = LAND_USE_CLASSES
        .iter()
        .find(|(aaa, _)| *aaa == class)
        .map_or(class, |(_, name)| name);
    let bez = source["bezfkt"]
        .as_str()
        .filter(|b| !b.is_empty())
        .or((nutzart == "Platz").then_some(SQUARE_WITHOUT_FUNCTION));
    let mut properties = Map::new();
    properties.insert("oid".into(), source["uuid"].clone());
    properties.insert("nutzart".into(), nutzart.into());
    if let Some(bez) = bez {
        properties.insert("bez".into(), bez.into());
    }
    with_properties(feature, properties)
}

/// `feature` with its id and geometry and the given properties.
fn with_properties(feature: &Value, properties: Map<String, Value>) -> Value {
    serde_json::json!({
        "type": "Feature",
        "id": properties["oid"],
        "geometry": feature["geometry"],
        "properties": properties,
    })
}

/// The Berlin buildings in `bbox`, normalised.
pub fn fetch_buildings(
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    Ok(fetch_wfs(fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, bbox)?
        .iter()
        .map(normalise_building)
        .collect())
}

/// The Berlin land use in `bbox`, normalised.
pub fn fetch_land_use(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Result<Vec<Value>, FetchError> {
    Ok(fetch_wfs(fetcher, LAND_USE_SERVICE, LAND_USE_TYPE, bbox)?
        .iter()
        .map(normalise_land_use)
        .collect())
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    pub const BUILDINGS: &str = include_str!("../../../../tests/fixtures/berlin-buildings.json");
    pub const LAND_USE: &str = include_str!("../../../../tests/fixtures/berlin-land-use.json");
    pub const BAUWERKE_FLAECHEN: &str =
        include_str!("../../../../tests/fixtures/berlin-bauwerke-flaechen.json");
    pub const BAUWERKE_LINIEN: &str =
        include_str!("../../../../tests/fixtures/berlin-bauwerke-linien.json");
    pub const RELIEF: &str = include_str!("../../../../tests/fixtures/berlin-relief.json");
    pub const VEGETATION_LINIEN: &str =
        include_str!("../../../../tests/fixtures/berlin-vegetation-linien.json");
    pub const STRASSENBAEUME: &str =
        include_str!("../../../../tests/fixtures/berlin-strassenbaeume.json");
    pub const ANLAGENBAEUME: &str =
        include_str!("../../../../tests/fixtures/berlin-anlagenbaeume.json");
    pub const BUILDING_PARTS: &str =
        include_str!("../../../../tests/fixtures/berlin-building-parts.json");
    pub const SB_GRUENFLAECHE: &str =
        include_str!("../../../../tests/fixtures/berlin-sb-gruenflaeche.json");
    pub const SB_TREPPE: &str = include_str!("../../../../tests/fixtures/berlin-sb-treppe.json");
    pub const SB_BORDSTEIN: &str =
        include_str!("../../../../tests/fixtures/berlin-sb-bordstein.json");
    pub const SB_GELAENDER: &str =
        include_str!("../../../../tests/fixtures/berlin-sb-gelaender.json");
    pub const SB_VERKEHRSSCHUTZGITTER: &str =
        include_str!("../../../../tests/fixtures/berlin-sb-verkehrsschutzgitter.json");
    pub const BELEUCHTUNG: &str =
        include_str!("../../../../tests/fixtures/berlin-beleuchtung.json");
    pub const ZIERBRUNNEN: &str =
        include_str!("../../../../tests/fixtures/berlin-zierbrunnen.json");
    pub const GEBAEUDEHOEHEN: &str =
        include_str!("../../../../tests/fixtures/berlin-gebaeudehoehen.json");
    /// What a WFS answers for a type without objects in the area.
    pub const EMPTY: &str = r#"{"type":"FeatureCollection","features":[],"totalFeatures":0}"#;

    /// The answer the tests serve for the WFS type `type_name`: its fixture, else an empty
    /// collection.
    pub fn answer(type_name: &str) -> &'static str {
        match type_name {
            "alkis:bauwerkeflaechen" => BAUWERKE_FLAECHEN,
            "alkis:bauwerkelinien" => BAUWERKE_LINIEN,
            "alkis:relief" => RELIEF,
            "alkis:gewaesservegetationlinien" => VEGETATION_LINIEN,
            "baumbestand:strassenbaeume" => STRASSENBAEUME,
            "baumbestand:anlagenbaeume" => ANLAGENBAEUME,
            "strassenbefahrung:ce_gruenflaeche" => SB_GRUENFLAECHE,
            "strassenbefahrung:cc_treppe" => SB_TREPPE,
            "strassenbefahrung:bd_bordstein" => SB_BORDSTEIN,
            "strassenbefahrung:bf_gelaender" => SB_GELAENDER,
            "strassenbefahrung:bb_verkehrsschutzgitter" => SB_VERKEHRSSCHUTZGITTER,
            "beleuchtung:beleuchtung" => BELEUCHTUNG,
            "zierbrunnen:bez_zierbrunnen" => ZIERBRUNNEN,
            "ua_gebaeudehoehen:gebaeudehoehen" => GEBAEUDEHOEHEN,
            _ => EMPTY,
        }
    }
}

#[cfg(test)]
mod forest_tests {
    use super::*;

    /// A forest map takes no street furniture, lamps or register trees (spec 2026-11-13).
    #[test]
    fn a_forest_map_takes_no_street_furniture_lamps_or_trees() {
        let left_out = [
            "strassenbefahrung:bj_sitzbank",
            "strassenbefahrung:av_poller",
            "strassenbefahrung:as_mast",
            "strassenbefahrung:am_fahnenmast",
            "strassenbefahrung:ag_werbesaeule",
            "beleuchtung:beleuchtung",
            "baumbestand:strassenbaeume",
            "baumbestand:anlagenbaeume",
            // The survey's paving: a forest map draws no paved areas from the street parcels.
            "strassenbefahrung:ce_gruenflaeche",
            "strassenbefahrung:cm_fahrbahn",
            "strassenbefahrung:cl_gehweg",
            "strassenbefahrung:ch_radweg",
            "strassenbefahrung:ck_parkflaeche",
            "strassenbefahrung:ci_oeffentlicher_platz",
            "strassenbefahrung:cj_fussgaengerzone",
            "strassenbefahrung:by_gehwegueberfahrt",
            "strassenbefahrung:bd_bordstein",
        ];
        for layer in &EXTRA_LAYERS {
            for ty in layer.types {
                let forest = ty.kinds.contains(&MapKind::Forest);
                if left_out.contains(&ty.name) {
                    assert!(!forest, "{}", ty.name);
                } else if ty.kinds.contains(&MapKind::Sprint) {
                    // Everything else of the sprint map comes along.
                    assert!(forest, "{}", ty.name);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::test_support::*;
    use super::*;

    /// Which layers are registers is said here once: the two tree registers and the three of
    /// the credit line. Each is a layer of the table, and no other layer is a register.
    #[test]
    fn the_registers_are_five_layers_of_the_table() {
        let registers: Vec<&str> = EXTRA_LAYERS
            .iter()
            .map(|layer| layer.file)
            .filter(|file| is_register(file))
            .collect();
        assert_eq!(
            registers,
            [
                "strassenbaeume",
                "anlagenbaeume",
                "beleuchtung",
                "zierbrunnen",
                "trinkwasserbrunnen"
            ]
        );
        let named: Vec<&str> = TREE_REGISTERS
            .iter()
            .copied()
            .chain(BERLIN_REGISTERS.iter().map(|&(file, _)| file))
            .collect();
        assert_eq!(named, registers);
        assert!(!is_register("sb-moebel") && !is_register("bauwerke-linien"));
    }
    use crate::fetch::test_support::MapFetcher;
    use crate::project::MapKind;
    use crate::sources::alkis::{parse_buildings, parse_land_use};

    /// Helene-Haeusler-Schule, Berlin-Prenzlauer Berg.
    const SCHOOL: BoundingBox = BoundingBox {
        min_e: 392_700.4,
        min_n: 5_820_750.0,
        max_e: 392_900.0,
        max_n: 5_820_950.6,
    };

    fn features(collection: &str) -> Vec<Value> {
        serde_json::from_str::<Value>(collection).unwrap()["features"]
            .as_array()
            .unwrap()
            .clone()
    }

    fn collection(features: &[Value]) -> String {
        serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
    }

    #[test]
    fn url_matches_the_wfs() {
        assert_eq!(
            wfs_url(BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL),
            "https://gdi.berlin.de/services/wfs/alkis_gebaeude?service=WFS&version=2.0.0\
             &request=GetFeature&typeNames=alkis_gebaeude:gebaeude&outputFormat=application/json\
             &count=5000&bbox=392700,5820750,392900,5820951,urn:ogc:def:crs:EPSG::25833"
        );
    }

    #[test]
    fn buildings_normalise_to_lgb_schema() {
        let normalised: Vec<Value> = features(BUILDINGS).iter().map(normalise_building).collect();
        let parsed = parse_buildings(&collection(&normalised)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        let by_oid = |oid: &str| parsed.buildings.iter().find(|b| b.oid == oid).unwrap();

        let gym = by_oid("DEBE03YY600004AR");
        assert_eq!(gym.kind, "Gebäude");
        assert_eq!(gym.function, "Allgemein bildende Schule");
        assert!(!gym.underground);
        assert_eq!(gym.updated, None);

        let part = by_oid("DEBE00YY13U0001t");
        assert_eq!(part.kind, "Bauteil");
        assert_eq!(part.function, "");

        assert!(by_oid("DEBE00YY2A80000K").underground, "Tiefgarage");
    }

    /// A raw ALKIS building with the given uuid, as the WFS serves it.
    fn raw_building(uuid: &str) -> Value {
        serde_json::json!({
            "type": "Feature", "id": format!("gebaeude.{uuid}"),
            "geometry": {"type": "Polygon", "coordinates": [[
                [392_800.0, 5_820_800.0], [392_810.0, 5_820_800.0], [392_810.0, 5_820_810.0],
                [392_800.0, 5_820_800.0]]]},
            "properties": {"uuid": uuid, "bezeich": "AX_Gebaeude", "bezgfk": "Wohnhaus"}
        })
    }

    #[test]
    fn building_gets_footprint_date() {
        let heights = features(GEBAEUDEHOEHEN);
        // `gml_id` of the height layer = `uuid` of the ALKIS building.
        let mut buildings = vec![
            normalise_building(&raw_building("DEBE02YY400008Kh")),
            normalise_building(&raw_building("DEBE00YY2mr0002Q")),
        ];
        join_footprint_dates(&mut buildings, &heights);
        let parsed = parse_buildings(&collection(&buildings)).unwrap();
        let dates: Vec<_> = parsed.buildings.iter().map(|b| b.updated).collect();
        assert_eq!(
            dates,
            ["2015-08-27", "2019-04-30"]
                .map(|d| d.parse().ok())
                .to_vec()
        );
    }

    #[test]
    fn unmatched_building_keeps_no_date() {
        let heights = features(GEBAEUDEHOEHEN);
        let mut buildings: Vec<Value> =
            features(BUILDINGS).iter().map(normalise_building).collect();
        buildings.push(normalise_building(&raw_building("DEBE02YY400008Kh")));
        join_footprint_dates(&mut buildings, &heights);
        let parsed = parse_buildings(&collection(&buildings)).unwrap();
        let dates: Vec<_> = parsed.buildings.iter().map(|b| b.updated).collect();
        // The fixture buildings are not in the height layer; the last one is.
        assert_eq!(dates[..dates.len() - 1], vec![None; dates.len() - 1]);
        assert_eq!(dates.last(), Some(&"2015-08-27".parse().ok()));
        // Without a height layer nothing changes.
        let mut alone = vec![normalise_building(&raw_building("DEBE02YY400008Kh"))];
        join_footprint_dates(&mut alone, &[]);
        assert!(alone[0]["properties"].get("aktualit").is_none());
    }

    #[test]
    fn land_use_classes_map_to_nutzart() {
        let normalised: Vec<Value> = features(LAND_USE).iter().map(normalise_land_use).collect();
        let land = parse_land_use(&collection(&normalised)).unwrap();
        let classes: Vec<(&str, Option<&str>)> = land
            .iter()
            .map(|l| (l.nutzart.as_str(), l.bez.as_deref()))
            .collect();
        assert_eq!(
            classes,
            [
                (
                    "Fläche besonderer funktionaler Prägung",
                    Some("Öffentliche Zwecke")
                ),
                ("Platz", Some(SQUARE_WITHOUT_FUNCTION)),
                ("Straßenverkehr", Some("Fußgängerzone")),
                ("Wohnbaufläche", None),
                ("Weg", None),
                ("Sport-, Freizeit- und Erholungsfläche", Some("Grünanlage")),
            ]
        );
        assert_eq!(land[0].oid, "DEBE01YYK0002J1d");
    }

    #[test]
    fn every_class_has_a_name() {
        for (class, nutzart) in LAND_USE_CLASSES {
            let feature = serde_json::json!({
                "type": "Feature",
                "geometry": null,
                "properties": { "uuid": "DEBE1", "bezeich": class, "bezfkt": "" }
            });
            assert_eq!(
                normalise_land_use(&feature)["properties"]["nutzart"],
                nutzart
            );
        }
    }

    #[test]
    fn unknown_class_keeps_its_aaa_name() {
        let feature = serde_json::json!({
            "type": "Feature",
            "geometry": null,
            "properties": { "uuid": "DEBE1", "bezeich": "AX_Heide", "bezfkt": null }
        });
        let normalised = normalise_land_use(&feature);
        assert_eq!(normalised["properties"]["nutzart"], "AX_Heide");
        assert_eq!(normalised["properties"]["bez"], Value::Null);
    }

    #[test]
    fn berlin_squares_stay_candidates() {
        let square = features(LAND_USE)
            .into_iter()
            .find(|f| f["properties"]["bezeich"] == "AX_Platz")
            .unwrap();
        let normalised = normalise_land_use(&square);
        assert_eq!(normalised["properties"]["bez"], SQUARE_WITHOUT_FUNCTION);
    }

    /// Strausberg, Brandenburg, far from the Berlin border.
    const STRAUSBERG: BoundingBox = BoundingBox {
        min_e: 421_490.0,
        min_n: 5_821_580.0,
        max_e: 421_736.0,
        max_n: 5_821_896.0,
    };

    /// Across the Berlin border at Schönefeld.
    const BORDER: BoundingBox = BoundingBox {
        min_e: 407_550.0,
        min_n: 5_799_224.0,
        max_e: 408_150.0,
        max_n: 5_799_824.0,
    };

    /// A fetcher that answers every extra layer type of the school area with its fixture.
    fn extra_fetcher() -> MapFetcher {
        let fetcher = MapFetcher::default();
        for layer in &EXTRA_LAYERS {
            for t in layer.types {
                fetcher.insert(
                    &wfs_url(layer.service, t.name, &SCHOOL),
                    answer(t.name).as_bytes().to_vec(),
                );
            }
        }
        fetcher
    }

    #[test]
    fn extra_layers_only_inside_berlin() {
        // Brandenburg only: no request at all, no layer.
        let fetcher = MapFetcher::default();
        assert!(fetch_extra_layers(&fetcher, &STRAUSBERG, MapKind::Sprint).is_empty());
        assert_eq!(fetcher.calls.get(), 0);

        // Inside Berlin: every layer of the map kind, in table order; relief is for sprint only.
        let fetcher = extra_fetcher();
        let sprint = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Sprint);
        let files: Vec<&str> = sprint.iter().map(|l| l.layer.file).collect();
        assert_eq!(
            files[..6],
            [
                "bauwerke-flaechen",
                "bauwerke-linien",
                "relief",
                "vegetation-linien",
                "strassenbaeume",
                "anlagenbaeume"
            ]
        );
        assert_eq!(fetcher.calls.get(), 33);
        let fetcher = extra_fetcher();
        let schoolyard = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Schoolyard);
        assert_eq!(schoolyard.len(), 14);
        assert!(schoolyard.iter().all(|l| l.layer.file != "relief"));
        assert_eq!(fetcher.calls.get(), 24);

        // A border area keeps the Berlin layers (nothing is served here, so they fail with a
        // missing service, but they are asked for).
        let fetcher = MapFetcher::default();
        let border = fetch_extra_layers(&fetcher, &BORDER, MapKind::Sprint);
        assert_eq!(border.len(), 18);
        assert!(border.iter().all(|l| l.features.is_err()));
    }

    #[test]
    fn survey_layers_only_inside_berlin() {
        // Brandenburg only: no request for any of the survey, lighting or height layers.
        let fetcher = MapFetcher::default();
        assert!(fetch_extra_layers(&fetcher, &STRAUSBERG, MapKind::Schoolyard).is_empty());
        assert_eq!(fetcher.calls.get(), 0);

        // The table: service, files and kinds of the new layers.
        let survey = "https://gdi.berlin.de/services/wfs/strassenbefahrung";
        let table: Vec<(&str, &str, Vec<&str>)> = EXTRA_LAYERS[6..]
            .iter()
            .map(|l| (l.file, l.service, l.types.iter().map(|t| t.name).collect()))
            .collect();
        let sb = |names: &[&'static str]| -> Vec<&'static str> { names.to_vec() };
        assert_eq!(
            table,
            [
                (
                    "sb-gruenflaeche",
                    survey,
                    sb(&["strassenbefahrung:ce_gruenflaeche"])
                ),
                (
                    "sb-flaechen",
                    survey,
                    sb(&[
                        "strassenbefahrung:cm_fahrbahn",
                        "strassenbefahrung:cl_gehweg",
                        "strassenbefahrung:ch_radweg",
                        "strassenbefahrung:ck_parkflaeche",
                        "strassenbefahrung:ci_oeffentlicher_platz",
                        "strassenbefahrung:cj_fussgaengerzone",
                        "strassenbefahrung:by_gehwegueberfahrt",
                    ])
                ),
                (
                    "sb-bordstein",
                    survey,
                    sb(&["strassenbefahrung:bd_bordstein"])
                ),
                ("sb-treppe", survey, sb(&["strassenbefahrung:cc_treppe"])),
                ("sb-mauer", survey, sb(&["strassenbefahrung:bh_mauer"])),
                (
                    "sb-gelaender",
                    survey,
                    sb(&[
                        "strassenbefahrung:bf_gelaender",
                        "strassenbefahrung:bb_verkehrsschutzgitter",
                        "strassenbefahrung:bg_leitplanke",
                    ])
                ),
                (
                    "sb-moebel",
                    survey,
                    sb(&[
                        "strassenbefahrung:bj_sitzbank",
                        "strassenbefahrung:aq_hydrant",
                        "strassenbefahrung:av_poller",
                        "strassenbefahrung:as_mast",
                        "strassenbefahrung:am_fahnenmast",
                        "strassenbefahrung:aw_spielgeraet",
                    ])
                ),
                (
                    "sb-objekte",
                    survey,
                    sb(&[
                        "strassenbefahrung:ag_werbesaeule",
                        "strassenbefahrung:br_fahrgastunterstand",
                        "strassenbefahrung:af_wasserpumpen_brunnen",
                        "strassenbefahrung:bv_springbrunnen_zierbrunnen",
                        "strassenbefahrung:ac_trinkwasserbrunnen_wasserspender",
                    ])
                ),
                (
                    "beleuchtung",
                    "https://gdi.berlin.de/services/wfs/beleuchtung",
                    sb(&["beleuchtung:beleuchtung"])
                ),
                (
                    "zierbrunnen",
                    "https://gdi.berlin.de/services/wfs/zierbrunnen",
                    sb(&["zierbrunnen:bez_zierbrunnen"])
                ),
                (
                    "trinkwasserbrunnen",
                    "https://gdi.berlin.de/services/wfs/trinkwasserbrunnen",
                    sb(&["trinkwasserbrunnen:trinkwasserbrunnen"])
                ),
                (
                    "gebaeudehoehen",
                    "https://gdi.berlin.de/services/wfs/ua_gebaeudehoehen",
                    sb(&["ua_gebaeudehoehen:gebaeudehoehen"])
                ),
            ]
        );

        // Sprint maps get the paved, curb and green layers, and the furniture that gives control
        // sites (bollards, masts, flag poles, benches, lamps) and the drinking fountains; they do
        // not get hydrants and play equipment. A type of a shared file only goes to its kinds.
        let fetcher = extra_fetcher();
        let files = |kind| -> Vec<&'static str> {
            fetch_extra_layers(&fetcher, &SCHOOL, kind)
                .iter()
                .map(|l| l.layer.file)
                .filter(|f| !EXTRA_LAYERS[..6].iter().any(|p| p.file == *f))
                .collect()
        };
        assert_eq!(
            files(MapKind::Sprint),
            [
                "sb-gruenflaeche",
                "sb-flaechen",
                "sb-bordstein",
                "sb-treppe",
                "sb-mauer",
                "sb-gelaender",
                "sb-moebel",
                "sb-objekte",
                "beleuchtung",
                "zierbrunnen",
                "trinkwasserbrunnen",
                "gebaeudehoehen"
            ]
        );
        assert_eq!(
            files(MapKind::Schoolyard),
            [
                "sb-treppe",
                "sb-mauer",
                "sb-gelaender",
                "sb-moebel",
                "sb-objekte",
                "beleuchtung",
                "zierbrunnen",
                "trinkwasserbrunnen",
                "gebaeudehoehen"
            ]
        );
        let requested: Vec<String> = fetcher.log.borrow().clone();
        let asked = |needle: &str| requested.iter().any(|u| u.contains(needle));
        // Both calls above ran on one fetcher: the sprint-only type was asked for once, by its
        // kind, and the drinking fountain, which both kinds use, once per kind.
        assert_eq!(
            requested
                .iter()
                .filter(|u| u.contains("ag_werbesaeule"))
                .count(),
            1
        );
        assert_eq!(
            requested
                .iter()
                .filter(|u| u.contains("ac_trinkwasserbrunnen_wasserspender"))
                .count(),
            2
        );
        assert_eq!(
            requested
                .iter()
                .filter(|u| u.contains("aq_hydrant"))
                .count(),
            1
        );
        assert!(asked("bd_bordstein") && asked("beleuchtung:beleuchtung"));
    }

    #[test]
    fn several_types_share_one_file() {
        let fetcher = extra_fetcher();
        let layers = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Schoolyard);
        let rails = layers
            .iter()
            .find(|l| l.layer.file == "sb-gelaender")
            .unwrap();
        let features = rails.features.as_ref().unwrap();
        // Two fixture features of `bf_gelaender`, two of `bb_verkehrsschutzgitter`, none of
        // `bg_leitplanke` (an empty answer is no error).
        assert_eq!(features.len(), 4);
        let layer_of = |f: &Value| f["properties"][TYPE_KEY].as_str().unwrap().to_string();
        let types: Vec<String> = features.iter().map(layer_of).collect();
        assert_eq!(
            types,
            [
                "bf_gelaender",
                "bf_gelaender",
                "bb_verkehrsschutzgitter",
                "bb_verkehrsschutzgitter"
            ]
        );
        assert!(
            features
                .iter()
                .all(|f| f["properties"][TYPE_KEY].is_string())
        );
        // Stored as one collection and read back unchanged.
        assert_eq!(parse_layer(&layer_to_geojson(features)).unwrap(), *features);
        // One request per type.
        let asked = fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.contains("typeNames=strassenbefahrung:b"))
            .count();
        assert!(asked >= 3);
        // A native `layer` property of the service stays as it is next to the marker.
        let fetcher = extra_fetcher();
        let body = serde_json::json!({ "type": "FeatureCollection", "features": [{
            "type": "Feature", "id": "bf_gelaender.1", "geometry": null,
            "properties": { "layer": "native", "gis_id": "Li_20150413_1" }
        }] });
        fetcher.insert(
            &wfs_url(
                rails.layer.service,
                "strassenbefahrung:bf_gelaender",
                &SCHOOL,
            ),
            body.to_string().into_bytes(),
        );
        let layers = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Schoolyard);
        let rails = layers
            .iter()
            .find(|l| l.layer.file == "sb-gelaender")
            .unwrap();
        let first = &rails.features.as_ref().unwrap()[0]["properties"];
        assert_eq!(
            (first["layer"].as_str(), first[TYPE_KEY].as_str()),
            (Some("native"), Some("bf_gelaender"))
        );
        // One failing type fails the file, not the others.
        let fetcher = extra_fetcher();
        let url = wfs_url(
            rails.layer.service,
            "strassenbefahrung:bg_leitplanke",
            &SCHOOL,
        );
        fetcher.failing.borrow_mut().push(url);
        let layers = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Schoolyard);
        let by_file = |file: &str| layers.iter().find(|l| l.layer.file == file).unwrap();
        assert!(by_file("sb-gelaender").features.is_err());
        assert!(by_file("sb-mauer").features.is_ok());
    }

    #[test]
    fn gis_id_dates_parse() {
        let date = |y, m, d| NaiveDate::from_ymd_opt(y, m, d);
        // The four spellings of the survey: compact, dotted, ISO and DDMMYYYY.
        assert_eq!(survey_date("Fl_20150413_37"), date(2015, 4, 13));
        assert_eq!(survey_date("Li_25.09.2014_230"), date(2014, 9, 25));
        assert_eq!(survey_date("Fl_2015-05-02_404949"), date(2015, 5, 2));
        assert_eq!(survey_date("FL_16072014_03142"), date(2014, 7, 16));
        // DDMMYYYY whose first four digits look like a year of the survey.
        assert_eq!(survey_date("FL_20102014_1"), date(2014, 10, 20));
        assert_eq!(survey_date("Fl_20141020_1"), date(2014, 10, 20));
        // A suffix after the running number does not matter.
        assert_eq!(survey_date("FL_04082014_01557_A_A_B"), date(2014, 8, 4));
        // Not a survey id, not a date, or not a plausible year.
        assert_eq!(survey_date(""), None);
        assert_eq!(survey_date("None"), None);
        assert_eq!(survey_date("Fl_99999999_1"), None);
        assert_eq!(survey_date("Fl_20150231_1"), None);
        assert_eq!(survey_date("Pk_13.04.1915_9"), None);

        // The fixtures carry all four; the date is a property of the stored feature, a missing
        // id gets the dataset date and a flag.
        let fetcher = extra_fetcher();
        let layers = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Sprint);
        let dates = |file: &str| -> Vec<String> {
            layers
                .iter()
                .find(|l| l.layer.file == file)
                .unwrap()
                .features
                .as_ref()
                .unwrap()
                .iter()
                .map(|f| f["properties"]["survey_date"].as_str().unwrap().to_string())
                .collect()
        };
        assert_eq!(
            dates("sb-gruenflaeche"),
            ["2015-04-16", "2016-08-19", "2014-08-04"]
        );
        assert_eq!(dates("sb-bordstein"), ["2015-04-16", "2014-07-16"]);
        assert_eq!(dates("sb-treppe"), ["2014-07-16"; 3]);
        let mut feature = serde_json::json!({
            "type": "Feature", "id": "x.1", "geometry": null,
            "properties": { "gis_id": "unlesbar" }
        });
        stamp_survey_date(&mut feature);
        assert_eq!(feature["properties"]["survey_date"], "2014-01-01");
        assert_eq!(feature["properties"]["survey_date_guessed"], "yes");
        // A parsed id sets no guessed flag; other layers get no date at all.
        let ok = &layers
            .iter()
            .find(|l| l.layer.file == "sb-treppe")
            .unwrap()
            .features
            .as_ref()
            .unwrap()[0];
        assert!(ok["properties"]["survey_date_guessed"].is_null());
        let trees = layers
            .iter()
            .find(|l| l.layer.file == "strassenbaeume")
            .unwrap();
        assert!(
            trees.features.as_ref().unwrap()[0]["properties"]["survey_date"].is_null(),
            "only the survey layers carry a survey date"
        );
    }

    #[test]
    fn multipoint_lamps_split() {
        let fetcher = extra_fetcher();
        let layers = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Schoolyard);
        let lamps = layers
            .iter()
            .find(|l| l.layer.file == "beleuchtung")
            .unwrap()
            .features
            .as_ref()
            .unwrap();
        // The fixture has four single-member MultiPoints, one of them not in operation: the
        // lamp is a Point, the out-of-service lamp is dropped.
        assert_eq!(lamps.len(), 3);
        assert!(lamps.iter().all(|f| f["geometry"]["type"] == "Point"));
        assert!(lamps.iter().all(|f| {
            f["properties"]["status"]
                .as_str()
                .unwrap()
                .starts_with("In Betrieb")
        }));
        assert_eq!(
            lamps[0]["geometry"]["coordinates"],
            serde_json::json!([390_475.233161, 5_816_762.706081])
        );
        // A MultiPoint with three members becomes three points with their own ids and the same
        // properties.
        let multi = serde_json::json!({
            "type": "Feature", "id": "beleuchtung.9",
            "geometry": { "type": "MultiPoint", "coordinates": [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]] },
            "properties": { "status": "In Betrieb" }
        });
        let split = split_multipoint(multi);
        let ids: Vec<&str> = split.iter().map(|f| f["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["beleuchtung.9", "beleuchtung.9#2", "beleuchtung.9#3"]);
        assert_eq!(
            split[2]["geometry"]["coordinates"],
            serde_json::json!([5.0, 6.0])
        );
        assert!(
            split
                .iter()
                .all(|f| f["properties"]["status"] == "In Betrieb")
        );
        // Other geometries pass unchanged; an empty MultiPoint gives no feature.
        let line = serde_json::json!({ "id": "a", "geometry": { "type": "LineString", "coordinates": [] } });
        assert_eq!(split_multipoint(line.clone()), [line]);
        let none = serde_json::json!({ "id": "b", "geometry": { "type": "MultiPoint", "coordinates": [] } });
        assert!(split_multipoint(none).is_empty());
    }

    #[test]
    fn extra_layer_url_matches_the_wfs() {
        let by_file = |file: &str| EXTRA_LAYERS.iter().find(|l| l.file == file).unwrap();
        let trees = by_file("strassenbaeume");
        assert_eq!(
            wfs_url(trees.service, trees.types[0].name, &SCHOOL),
            "https://gdi.berlin.de/services/wfs/baumbestand?service=WFS&version=2.0.0\
             &request=GetFeature&typeNames=baumbestand:strassenbaeume&outputFormat=application/json\
             &count=5000&bbox=392700,5820750,392900,5820951,urn:ogc:def:crs:EPSG::25833"
        );
        let lines = by_file("bauwerke-linien");
        assert_eq!(
            wfs_url(lines.service, lines.types[0].name, &SCHOOL),
            "https://gdi.berlin.de/services/wfs/alkis?service=WFS&version=2.0.0\
             &request=GetFeature&typeNames=alkis:bauwerkelinien&outputFormat=application/json\
             &count=5000&bbox=392700,5820750,392900,5820951,urn:ogc:def:crs:EPSG::25833"
        );
        // The whole table: service and type per file.
        let table: Vec<(&str, &str, &str)> = EXTRA_LAYERS
            .iter()
            .take(6)
            .map(|l| (l.file, l.service, l.types[0].name))
            .collect();
        let alkis = "https://gdi.berlin.de/services/wfs/alkis";
        let trees = "https://gdi.berlin.de/services/wfs/baumbestand";
        assert_eq!(
            table,
            [
                ("bauwerke-flaechen", alkis, "alkis:bauwerkeflaechen"),
                ("bauwerke-linien", alkis, "alkis:bauwerkelinien"),
                ("relief", alkis, "alkis:relief"),
                (
                    "vegetation-linien",
                    alkis,
                    "alkis:gewaesservegetationlinien"
                ),
                ("strassenbaeume", trees, "baumbestand:strassenbaeume"),
                ("anlagenbaeume", trees, "baumbestand:anlagenbaeume"),
            ]
        );
    }

    #[test]
    fn dedup_by_feature_id_without_uuid() {
        // Tree features have no `uuid`; the top-level GeoServer id decides.
        let trees = features(STRASSENBAEUME);
        assert!(trees.iter().all(|t| t["properties"]["uuid"].is_null()));
        let layer = EXTRA_LAYERS
            .iter()
            .find(|l| l.file == "strassenbaeume")
            .unwrap();
        let fetcher = MapFetcher::default();
        let first = wfs_url(layer.service, layer.types[0].name, &SCHOOL);
        let second = format!("{}?STARTINDEX=3", layer.service);
        let page = |fs: &[Value], next: Option<&str>| {
            let links: Vec<Value> = next
                .map(|href| serde_json::json!({ "rel": "next", "href": href }))
                .into_iter()
                .collect();
            serde_json::json!({ "type": "FeatureCollection", "features": fs, "links": links })
                .to_string()
                .into_bytes()
        };
        fetcher.insert(&first, page(&trees[..3], Some(&second)));
        fetcher.insert(&second, page(&trees[2..], None));
        let fetched = fetch_wfs(&fetcher, layer.service, layer.types[0].name, &SCHOOL).unwrap();
        assert_eq!(fetched.len(), 4);
    }

    #[test]
    fn empty_layer_is_stored_not_an_error() {
        // The hedge layer has no feature in the school area: an empty collection, not an error.
        let fetcher = extra_fetcher();
        let layers = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Sprint);
        let hedges = layers
            .iter()
            .find(|l| l.layer.file == "vegetation-linien")
            .unwrap();
        let none = hedges.features.as_ref().unwrap();
        assert_eq!(none.len(), 0);
        // Written as a collection with no feature, it reads back as an empty layer.
        assert_eq!(parse_layer(&layer_to_geojson(none)).unwrap().len(), 0);
        // And a layer with content round-trips.
        let areas = layers[0].features.as_ref().unwrap();
        assert_eq!(areas.len(), 10);
        assert_eq!(parse_layer(&layer_to_geojson(areas)).unwrap(), *areas);
    }

    #[test]
    fn building_parts_keep_bauart() {
        let parts: Vec<Value> = features(BUILDING_PARTS)
            .iter()
            .map(normalise_building)
            .collect();
        let bauart: Vec<&str> = parts
            .iter()
            .map(|p| p["properties"]["bauart"].as_str().unwrap())
            .collect();
        assert!(bauart.contains(&"Durchfahrt im Gebäude"));
        assert!(bauart.contains(&"Arkade"));
        // The no-break space of the source value is kept as it is.
        assert!(bauart.contains(&"Durchfahrt an\u{a0}überbautem Verkehrsweg"));
        // Still a Bauteil with LGB properties.
        assert!(
            parts
                .iter()
                .all(|p| p["properties"]["gebnutzbez"] == "Bauteil")
        );
        // A building without a part kind has no `bauart`.
        let whole = normalise_building(&features(BUILDINGS)[0]);
        assert!(whole["properties"]["bauart"].is_null());
        // The extra property does not disturb the LGB parser.
        let parsed = parse_buildings(&collection(&parts)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        assert_eq!(parsed.buildings.len(), 5);
    }

    #[test]
    fn pages_are_followed_and_duplicates_dropped() {
        let all = features(BUILDINGS);
        let fetcher = MapFetcher::default();
        let first = wfs_url(BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL);
        let second = format!("{BUILDINGS_SERVICE}?STARTINDEX=3");
        let page = |fs: &[Value], next: Option<&str>| {
            let links: Vec<Value> = next
                .map(|href| serde_json::json!({ "rel": "next", "href": href }))
                .into_iter()
                .collect();
            serde_json::json!({ "type": "FeatureCollection", "features": fs, "links": links })
                .to_string()
                .into_bytes()
        };
        fetcher.insert(&first, page(&all[..3], Some(&second)));
        fetcher.insert(&second, page(&all[2..], None));
        let fetched = fetch_wfs(&fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL).unwrap();
        assert_eq!(fetched.len(), 4);
    }

    /// Fails the first `failures` requests, then serves `body`.
    struct Flaky {
        failures: Cell<u32>,
        body: Vec<u8>,
        calls: RefCell<Vec<String>>,
    }

    impl Fetcher for Flaky {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.calls.borrow_mut().push(url.to_string());
            if self.failures.get() > 0 {
                self.failures.set(self.failures.get() - 1);
                return Err(FetchError::Http {
                    url: url.to_string(),
                    message: "Connection reset by peer".to_string(),
                });
            }
            Ok(self.body.clone())
        }
    }

    fn flaky(failures: u32, body: &[u8]) -> Flaky {
        Flaky {
            failures: Cell::new(failures),
            body: body.to_vec(),
            calls: RefCell::new(Vec::new()),
        }
    }

    #[test]
    fn flaky_server_is_retried() {
        let fetcher = flaky(2, BUILDINGS.as_bytes());
        let fetched = fetch_wfs(&fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL).unwrap();
        assert_eq!(fetched.len(), 4);
        assert_eq!(fetcher.calls.borrow().len(), 3);

        let fetcher = flaky(3, BUILDINGS.as_bytes());
        let err = fetch_wfs(&fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL).unwrap_err();
        assert!(err.to_string().contains("Connection reset"), "{err}");
        assert_eq!(fetcher.calls.borrow().len(), 3);
    }

    #[test]
    fn xml_answer_is_an_error() {
        let report =
            br#"<?xml version="1.0" ?><ows:ExceptionReport><ows:Exception/></ows:ExceptionReport>"#;
        let fetcher = flaky(0, report);
        let err = fetch_wfs(&fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL).unwrap_err();
        assert!(
            err.to_string().contains("not a GeoJSON FeatureCollection"),
            "{err}"
        );
        assert!(err.to_string().starts_with("invalid ALKIS data: "), "{err}");
        assert_eq!(
            fetcher.calls.borrow().len(),
            3,
            "an XML answer is retried too"
        );
    }

    #[test]
    fn missing_service_is_not_retried() {
        let fetcher = MapFetcher::default();
        let err = fetch_wfs(&fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL).unwrap_err();
        assert!(matches!(err, FetchError::NotFound(_)), "{err}");
        assert_eq!(fetcher.calls.get(), 1);
    }
}
