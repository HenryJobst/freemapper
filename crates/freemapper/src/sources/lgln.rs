//! LGLN Niedersachsen open data (CC BY 4.0, "© GeoBasis-DE/LGLN"), all in EPSG:25832: the
//! orthophoto from the WMS, DGM1 and DOM1 from the MapServer WCS, and the dates of the height
//! models and the orthophoto from the STAC APIs (the newest item at the centre of the area).
//!
//! The STAC answers are not cached: a date reaches the project record only.

use crate::note::{Note, note};
use std::path::Path;

use chrono::{Datelike as _, NaiveDate};
use serde_json::Value;

use crate::crs::{BoundingBox, Crs, LatLon, ProjectedPoint};
use crate::fetch::{FetchError, FetchMode, Fetcher, get_with_retries, one_line};
use crate::raster::{GridSpec, Raster};
use crate::sources::float_tiff::{require_stored, write_float_raster};
use crate::sources::height_model::HeightModel;
use crate::sources::wcs::WcsService;
use crate::sources::wms::WmsService;

/// The orthophoto WMS: the newest DOP20 (the same pixels as the tiles). MaxWidth/MaxHeight is
/// 15000, but MapServer answers only once the whole image is ready, so the pieces stay at 4000 px
/// (≈9 s each). An oversize request is an HTTP 200 XML exception. Bremen is left white.
pub const WMS: WmsService = WmsService {
    url: "https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms",
    layer: "ni_dop20",
    crs: Crs::Utm32,
    extra: "",
    max_px: 4000,
};

/// The STAC API of the orthophoto (collection `DOP`, every vintage).
pub const DOP_STAC: &str = "https://dop.stac.lgln.niedersachsen.de";
/// The STAC API of DGM1 (collection `dgm1`).
pub const DGM_STAC: &str = "https://dgm.stac.lgln.niedersachsen.de";
/// The STAC API of DOM1 (collection `dom1`).
pub const DOM_STAC: &str = "https://dom.stac.lgln.niedersachsen.de";

/// The extent of `ni_dgm1` (DescribeCoverage); DOM1 has the same tiles.
pub const ENVELOPE: BoundingBox = BoundingBox {
    min_e: 341_000.0,
    min_n: 5_681_992.0,
    max_e: 676_040.0,
    max_n: 5_973_000.0,
};

/// DGM1 from the WCS: a plain Float32 TIFF (LZW with the floating point predictor), NoData −9999.
pub const DGM1_WCS: WcsService = WcsService {
    url: "https://opendata.geoservices.lgln.niedersachsen.de/dgm_wcs",
    coverage: "ni_dgm1",
    extra: "",
    multipart: false,
    envelope: ENVELOPE,
    max_m: WCS_MAX_M,
};

/// DOM1 from the WCS, like [`DGM1_WCS`].
pub const DOM1_WCS: WcsService = WcsService {
    url: "https://opendata.geoservices.lgln.niedersachsen.de/dom_wcs",
    coverage: "ni_dom1",
    ..DGM1_WCS
};

/// Largest side of one GetCoverage. The WCS has no limit up to 5 km, but MapServer answers
/// only once the whole image is ready (2 km: 18 s, 4 km: 49 s), so 2 km pieces stay well inside
/// the response timeout.
const WCS_MAX_M: f64 = 2000.0;
/// Items per STAC page (the server caps `limit` at 1000).
const STAC_LIMIT: u32 = 100;
/// Pages of one STAC search; a search at one point finds a dozen items.
const STAC_MAX_PAGES: usize = 10;

/// A height model served by the WCS at 1 m.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// DGM1, the terrain.
    Dgm1,
    /// DOM1, the (laser) surface.
    Dom1,
}

impl Model {
    pub fn wcs(self) -> WcsService {
        match self {
            Self::Dgm1 => DGM1_WCS,
            Self::Dom1 => DOM1_WCS,
        }
    }

    /// The STAC API with the model's tiles and dates.
    pub fn stac(self) -> &'static str {
        match self {
            Self::Dgm1 => DGM_STAC,
            Self::Dom1 => DOM_STAC,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (LGLN DGM1)",
            Self::Dom1 => "surface model (LGLN DOM1)",
        }
    }
}

/// A STAC item: its id and the date of its `datetime`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StacItem {
    pub id: String,
    pub date: NaiveDate,
}

/// The STAC search of `host` at the centre of `bbox` (EPSG:25832): a box of ±10 m around it in
/// lon/lat, which the search wants; `None` when a corner has no lon/lat.
pub fn stac_search_url(host: &str, bbox: &BoundingBox) -> Option<String> {
    let (e, n) = (
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    );
    let corners: Vec<_> = [(-10.0, -10.0), (10.0, -10.0), (-10.0, 10.0), (10.0, 10.0)]
        .into_iter()
        .map(|(de, dn)| {
            Crs::Utm32.inverse(ProjectedPoint {
                easting: e + de,
                northing: n + dn,
            })
        })
        .collect::<Option<Vec<LatLon>>>()?;
    let min = |f: fn(&LatLon) -> f64| corners.iter().map(f).fold(f64::MAX, f64::min);
    let max = |f: fn(&LatLon) -> f64| corners.iter().map(f).fold(f64::MIN, f64::max);
    Some(format!(
        "{host}/search?bbox={:.6},{:.6},{:.6},{:.6}&limit={STAC_LIMIT}",
        min(|p| p.lon),
        min(|p| p.lat),
        max(|p| p.lon),
        max(|p| p.lat)
    ))
}

/// The newest item (by `datetime`) of the STAC search at the centre of `bbox`, following the
/// `next` links (on `host` only, up to 10 pages); `None` when the search finds nothing. Items
/// without a readable `datetime` are passed over; paging ends at an empty page or a page read
/// before.
pub fn newest_item(
    fetcher: &dyn Fetcher,
    host: &str,
    bbox: &BoundingBox,
) -> Result<Option<StacItem>, FetchError> {
    let mut newest: Option<StacItem> = None;
    let mut visited: Vec<String> = Vec::new();
    let first = stac_search_url(host, bbox).ok_or_else(|| FetchError::Http {
        url: format!("{host}/search"),
        message: "the centre of the map area has no lon/lat".to_string(),
    })?;
    let mut next = Some(first);
    while let Some(url) = next.take() {
        if visited.len() == STAC_MAX_PAGES {
            return Err(FetchError::Http {
                url,
                message: format!("the STAC search has more than {STAC_MAX_PAGES} pages"),
            });
        }
        let page = get_with_retries(fetcher, &url, |bytes| {
            serde_json::from_slice::<Value>(bytes)
                .ok()
                .filter(|page| page["features"].is_array())
                .ok_or_else(|| FetchError::Http {
                    url: url.clone(),
                    message: format!(
                        "not a STAC search answer: {}",
                        one_line(&String::from_utf8_lossy(bytes), 80)
                    ),
                })
        })?;
        let features = page["features"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default();
        for item in features.iter().filter_map(stac_item) {
            if newest.as_ref().is_none_or(|n| item.date > n.date) {
                newest = Some(item);
            }
        }
        visited.push(url);
        if features.is_empty() {
            break;
        }
        let Some(href) = page["links"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|link| link["rel"] == "next")
            .and_then(|link| link["href"].as_str())
        else {
            break;
        };
        if !href.starts_with(&format!("{host}/")) {
            return Err(FetchError::Http {
                url: visited.pop().unwrap_or_default(),
                message: format!("the STAC next link leaves {host}: {}", one_line(href, 120)),
            });
        }
        next = Some(href.to_string()).filter(|href| !visited.contains(href));
    }
    Ok(newest)
}

/// The id and `properties.datetime` (`YYYY-MM-DDT00:00:00Z`) of a STAC item.
fn stac_item(feature: &Value) -> Option<StacItem> {
    let datetime = feature["properties"]["datetime"].as_str()?;
    let date = chrono::DateTime::parse_from_rfc3339(datetime)
        .ok()?
        .date_naive();
    Some(StacItem {
        id: feature["id"].as_str().unwrap_or_default().to_string(),
        date,
    })
}

/// The date of the newest orthophoto at the centre of `bbox`, from one DOP STAC search: the
/// flight date (written `DD.MM.YYYY` like the other states') and, by its month, the leaf state.
/// `Ok(None)` when the STAC lists no item there; an error when the search failed.
pub fn orthophoto_date(
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
) -> Result<Option<NaiveDate>, FetchError> {
    Ok(newest_item(fetcher, DOP_STAC, bbox)?.map(|item| item.date))
}

/// How much older than the orthophoto the DOM1 may be before the nDSM gets a warning. The laser
/// scan behind it is mostly from 2016/2017, the orthophoto from 2024 to 2026 (§6): new buildings
/// and grown or felled trees are then missing from the nDSM, though the orthophoto shows them.
pub const SURFACE_AGE_WARNING_MONTHS: u32 = 24;

/// The warning when the surface model of `surface` is more than [`SURFACE_AGE_WARNING_MONTHS`]
/// older than the orthophoto flown on `flight`; `None` when they are close.
pub fn surface_age_warning(surface: NaiveDate, flight: NaiveDate) -> Option<String> {
    let limit = surface.checked_add_months(chrono::Months::new(SURFACE_AGE_WARNING_MONTHS))?;
    (limit < flight).then(|| {
        format!(
            "the {} is from {}, the orthophoto from {}. The nDSM may lack newer buildings and trees.",
            Model::Dom1.label(),
            surface.format("%d.%m.%Y"),
            flight.format("%d.%m.%Y")
        )
    })
}

/// What the flight month says about the trees: flights from May to October show them in leaf
/// (a quarter of the state), the spring flights bare.
pub fn leaf_state(flight: NaiveDate) -> Note {
    let date = flight.format("%d.%m.%Y").to_string();
    if (5..=10).contains(&flight.month()) {
        note!("progress-orthophoto-in-leaf", date = date)
    } else {
        note!("progress-orthophoto-bare", date = date)
    }
}

/// DGM1 or DOM1 over `bbox` (EPSG:25832) from the WCS, in pieces of at most 2 km, written as a
/// Float32 GeoTIFF `out` with its world file.
///
/// The date (`YYYY-MM-DD`) is the newest STAC item's at the centre; an empty or failed search
/// leaves it out with a warning. An area without any height (beyond the coverage's data) is an
/// error, and an HTML page in place of the TIFF (MapServer answers some faults with HTTP 200)
/// too.
///
/// Offline, the model stored at `out` is kept when it covers the area; nothing is asked (the
/// STAC is not cached, the caller keeps the recorded date).
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
) -> Result<HeightModel, FetchError> {
    let grid = GridSpec::covering(bbox, 1.0);
    let area = grid.bounds();
    if mode == FetchMode::Offline {
        require_stored(out, &area, model.label())?;
        return Ok(HeightModel::default());
    }
    let wcs = model.wcs();
    let mut raster = Raster::filled(grid, f32::NAN);
    wcs.fill(fetcher, &mut raster, &area)?;
    if raster.data.iter().all(|z| z.is_nan()) {
        return Err(FetchError::Http {
            url: wcs.get_coverage_url(&area),
            message: format!("the {} has no height in the map area", model.label()),
        });
    }
    write_float_raster(&raster, root, out, wcs.url)?;
    let mut warnings = Vec::new();
    let date = match newest_item(fetcher, model.stac(), bbox) {
        Ok(Some(item)) => Some(item.date.format("%Y-%m-%d").to_string()),
        Ok(None) => {
            warnings.push(format!(
                "{}: no date, the STAC lists no tile at the centre of the map area",
                model.label()
            ));
            None
        }
        Err(e) => {
            warnings.push(format!("{}: no date ({e})", model.label()));
            None
        }
    };
    Ok(HeightModel {
        date,
        warnings,
        ..HeightModel::default()
    })
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const WCS_DGM1: &[u8] =
        include_bytes!("../../../../tests/fixtures/niedersachsen-wcs-dgm1.tif");
    pub const STAC_DGM1: &str =
        include_str!("../../../../tests/fixtures/niedersachsen-stac-dgm1.json");
    pub const STAC_DOP: &str =
        include_str!("../../../../tests/fixtures/niedersachsen-stac-dop.json");

    /// A STAC search page with items `(id, datetime)` and an optional `next` link.
    pub fn stac_page(items: &[(&str, &str)], next: Option<&str>) -> Vec<u8> {
        let features: Vec<serde_json::Value> = items
            .iter()
            .map(|(id, datetime)| {
                serde_json::json!({
                    "type": "Feature", "id": id, "collection": "dgm1",
                    "properties": { "datetime": datetime },
                })
            })
            .collect();
        let mut links = vec![serde_json::json!({ "rel": "self", "href": "x" })];
        if let Some(href) = next {
            links.push(serde_json::json!({ "rel": "next", "href": href, "method": "GET" }));
        }
        serde_json::json!({
            "type": "FeatureCollection", "features": features, "links": links,
            "numberReturned": items.len(),
        })
        .to_string()
        .into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use crate::sources::float_tiff::decode_float_tiff;

    /// The live WCS sample: 200 × 50 m at the Kaiser-Wilhelm- und Ratsgymnasium, Hannover.
    const SAMPLE: BoundingBox = BoundingBox {
        min_e: 551_800.0,
        min_n: 5_803_350.0,
        max_e: 552_000.0,
        max_n: 5_803_400.0,
    };

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn leaf_state_by_month() {
        let leaf_state = |d: NaiveDate| super::leaf_state(d).to_string();
        let bare = |d: &str| leaf_state(date(d)).ends_with("with the trees bare");
        // The spring flights (February to April) and the winter months are bare.
        for d in [
            "2025-01-15",
            "2025-03-06",
            "2025-04-30",
            "2025-11-01",
            "2025-12-31",
        ] {
            assert!(bare(d), "{d}: {}", leaf_state(date(d)));
        }
        // May to October: in leaf.
        for d in ["2025-05-01", "2025-08-12", "2025-10-31"] {
            assert!(!bare(d), "{d}");
            assert!(
                leaf_state(date(d)).contains("with the trees in leaf"),
                "{d}"
            );
        }
        assert_eq!(
            leaf_state(date("2025-04-30")),
            "the orthophoto was flown on 30.04.2025, with the trees bare"
        );
    }

    #[test]
    fn stac_picks_the_newest_item() {
        // The search box: ±10 m around the centre, in lon/lat (the school at 9.762 E, 52.378 N).
        let url = stac_search_url(DOP_STAC, &SAMPLE).unwrap();
        let query = url
            .strip_prefix("https://dop.stac.lgln.niedersachsen.de/search?bbox=")
            .unwrap_or_else(|| panic!("{url}"));
        let corners: Vec<f64> = query
            .split('&')
            .next()
            .unwrap()
            .split(',')
            .map(|v| v.parse().unwrap())
            .collect();
        assert_eq!(corners.len(), 4, "{url}");
        assert!((9.761..9.763).contains(&corners[0]), "{url}");
        assert!((52.377..52.379).contains(&corners[1]), "{url}");
        assert!(corners[2] > corners[0] && corners[3] > corners[1], "{url}");
        assert!(
            corners[2] - corners[0] < 0.001 && corners[3] - corners[1] < 0.001,
            "{url}"
        );
        assert!(url.contains("&limit=100"), "{url}");
        // The live answer lists five vintages of the 2 km blocks and two 1 km DOP10 tiles
        // (2013 … 2025); the newest is 2025-03-06, in whatever order the items come.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DOP_STAC, STAC_DOP.as_bytes().to_vec());
        let newest = newest_item(&fetcher, DOP_STAC, &SAMPLE).unwrap().unwrap();
        assert_eq!(newest.date, date("2025-03-06"));
        assert!(newest.id.ends_with("_2025-03-06"), "{}", newest.id);
        // One DOP search gives the date for the flight date and the leaf state.
        assert_eq!(
            orthophoto_date(&fetcher, &SAMPLE).unwrap(),
            Some(date("2025-03-06"))
        );
        let mut answer: Value = serde_json::from_str(STAC_DOP).unwrap();
        answer["features"].as_array_mut().unwrap().reverse();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DOP_STAC, answer.to_string().into_bytes());
        let newest = newest_item(&fetcher, DOP_STAC, &SAMPLE).unwrap().unwrap();
        assert_eq!(newest.date, date("2025-03-06"));
        // An item without a readable date is passed over.
        let page = stac_page(
            &[
                ("a", "2016-04-18T00:00:00Z"),
                ("b", "soon"),
                ("c", "2021-02-01T00:00:00Z"),
            ],
            None,
        );
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DGM_STAC, page);
        let newest = newest_item(&fetcher, DGM_STAC, &SAMPLE).unwrap().unwrap();
        assert_eq!((newest.id.as_str(), newest.date), ("c", date("2021-02-01")));
        // A bbox outside Niedersachsen: 200 with an empty list, so no item and no date.
        let empty = stac_page(&[], None);
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DOP_STAC, empty);
        assert_eq!(newest_item(&fetcher, DOP_STAC, &SAMPLE).unwrap(), None);
        assert_eq!(orthophoto_date(&fetcher, &SAMPLE).unwrap(), None);
        // Not a search answer: an error naming it.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DOP_STAC, b"<html>\x1b[2JBad Gateway</html>".to_vec());
        let err = newest_item(&fetcher, DOP_STAC, &SAMPLE)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a STAC search answer"), "{err}");
        assert!(!err.contains('\x1b'), "{err:?}");
        // The caller tells a failed search from one without an item.
        assert!(orthophoto_date(&fetcher, &SAMPLE).is_err());
    }

    #[test]
    fn stac_search_needs_a_lon_lat_centre() {
        // A centre no lon/lat projects to gives no search box, so nothing is asked.
        let nowhere = BoundingBox {
            min_e: f64::NAN,
            ..SAMPLE
        };
        assert_eq!(stac_search_url(DGM_STAC, &nowhere), None);
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DGM_STAC, STAC_DGM1.as_bytes().to_vec());
        let err = newest_item(&fetcher, DGM_STAC, &nowhere)
            .unwrap_err()
            .to_string();
        assert!(err.contains("no lon/lat"), "{err}");
        assert!(!err.contains("1797693"), "{err}");
        assert_eq!(fetcher.calls.get(), 0);
    }

    #[test]
    fn lgln_wcs_pieces_are_2_km() {
        // A 4 x 4 km area is asked for in four GetCoverages of 2 x 2 km, for both models.
        let area = BoundingBox {
            min_e: 550_000.0,
            min_n: 5_800_000.0,
            max_e: 554_000.0,
            max_n: 5_804_000.0,
        };
        let grid = crate::raster::GridSpec::covering(&area, 1.0);
        for wcs in [DGM1_WCS, DOM1_WCS] {
            let fetcher = MapFetcher::default();
            let mut expected = Vec::new();
            for (min_e, max_e) in [(550_000.0, 552_000.0), (552_000.0, 554_000.0)] {
                for (min_n, max_n) in [(5_800_000.0, 5_802_000.0), (5_802_000.0, 5_804_000.0)] {
                    let piece = BoundingBox {
                        min_e,
                        min_n,
                        max_e,
                        max_n,
                    };
                    let url = wcs.get_coverage_url(&piece);
                    // A one-pixel answer at the piece's north-west corner is enough here.
                    let corner = crate::raster::GridSpec {
                        west: min_e,
                        north: max_n,
                        pixel_size: 1.0,
                        width: 1,
                        height: 1,
                    };
                    fetcher.insert(&url, crate::fetch::test_support::geotiff(&corner, &[50.0]));
                    expected.push(url);
                }
            }
            let mut raster = crate::raster::Raster::filled(grid, f32::NAN);
            wcs.fill(&fetcher, &mut raster, &area).unwrap();
            assert_eq!(*fetcher.log.borrow(), expected, "{}", wcs.url);
            assert_eq!(raster.get(0, 0), 50.0);
            assert_eq!(raster.get(2000, 2000), 50.0);
        }
    }

    #[test]
    fn stac_pages_are_followed() {
        let first = stac_search_url(DGM_STAC, &SAMPLE).unwrap();
        let second = format!("{DGM_STAC}/search?bbox=1,2,3,4&limit=100&token=next:dgm1:a");
        let third = format!("{DGM_STAC}/search?bbox=1,2,3,4&limit=100&token=next:dgm1:b");
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &first,
            stac_page(&[("a", "2016-04-18T00:00:00Z")], Some(&second)),
        );
        fetcher.insert(
            &second,
            stac_page(&[("b", "2025-03-03T00:00:00Z")], Some(&third)),
        );
        fetcher.insert(&third, stac_page(&[("c", "2020-01-01T00:00:00Z")], None));
        let newest = newest_item(&fetcher, DGM_STAC, &SAMPLE).unwrap().unwrap();
        assert_eq!((newest.id.as_str(), newest.date), ("b", date("2025-03-03")));
        assert_eq!(
            *fetcher.log.borrow(),
            vec![first.clone(), second.clone(), third.clone()]
        );
        // A link back to a page already read ends the search.
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &first,
            stac_page(&[("a", "2016-04-18T00:00:00Z")], Some(&second)),
        );
        fetcher.insert(
            &second,
            stac_page(&[("b", "2017-01-01T00:00:00Z")], Some(&first)),
        );
        let newest = newest_item(&fetcher, DGM_STAC, &SAMPLE).unwrap().unwrap();
        assert_eq!(newest.id, "b");
        assert_eq!(fetcher.calls.get(), 2);
        // An empty page ends it too.
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &first,
            stac_page(&[("a", "2016-04-18T00:00:00Z")], Some(&second)),
        );
        fetcher.insert(&second, stac_page(&[], Some(&third)));
        assert_eq!(
            newest_item(&fetcher, DGM_STAC, &SAMPLE)
                .unwrap()
                .unwrap()
                .id,
            "a"
        );
        assert_eq!(fetcher.calls.get(), 2);
        // A link to another host is not followed.
        let fetcher = MapFetcher::default();
        let elsewhere = "https://stac.example/search?token=next:dgm1:a";
        fetcher.insert(
            &first,
            stac_page(&[("a", "2016-04-18T00:00:00Z")], Some(elsewhere)),
        );
        fetcher.insert(elsewhere, stac_page(&[("x", "2026-01-01T00:00:00Z")], None));
        let err = newest_item(&fetcher, DGM_STAC, &SAMPLE)
            .unwrap_err()
            .to_string();
        assert!(err.contains("leaves"), "{err}");
        assert_eq!(fetcher.calls.get(), 1);
        // Endless paging stops at the page cap.
        let fetcher = MapFetcher::default();
        let page =
            |i: usize| format!("{DGM_STAC}/search?bbox=1,2,3,4&limit=100&token=next:dgm1:{i}");
        fetcher.insert(
            &first,
            stac_page(&[("0", "2016-04-18T00:00:00Z")], Some(&page(1))),
        );
        for i in 1..=STAC_MAX_PAGES + 1 {
            let id = i.to_string();
            fetcher.insert(
                &page(i),
                stac_page(&[(id.as_str(), "2016-04-18T00:00:00Z")], Some(&page(i + 1))),
            );
        }
        let err = newest_item(&fetcher, DGM_STAC, &SAMPLE)
            .unwrap_err()
            .to_string();
        assert!(err.contains("more than 10 pages"), "{err}");
        assert_eq!(fetcher.calls.get(), STAC_MAX_PAGES);
    }

    #[test]
    fn wcs_html_answer_is_an_error() {
        // MapServer answers a request it cannot serve (an unknown map) with HTTP 200 and HTML.
        let page =
            b"<HTML>\n<HEAD><TITLE>MapServer Message</TITLE></HEAD>\n<!-- MapServer version \
            8.0.1 -->\n<BODY BGCOLOR=\"#FFFFFF\">\nmsCGILoadMap(): Web application error. CGI \
            variable &quot;map&quot; not found.\n</BODY></HTML>";
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DGM1_WCS.url, page.to_vec());
        fetcher.insert_prefix(DGM_STAC, STAC_DGM1.as_bytes().to_vec());
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("dgm1.tif");
        let err = fetch_model(
            &fetcher,
            FetchMode::Online,
            Model::Dgm1,
            &SAMPLE,
            out.parent().unwrap(),
            &out,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("no TIFF in the WCS answer"), "{err}");
        assert!(err.contains("MapServer Message"), "{err}");
        assert!(!out.exists());
        // DOM1 the same.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DOM1_WCS.url, page.to_vec());
        assert!(
            fetch_model(
                &fetcher,
                FetchMode::Online,
                Model::Dom1,
                &SAMPLE,
                out.parent().unwrap(),
                &out
            )
            .is_err()
        );
        assert!(!out.exists());
    }

    #[test]
    fn lgln_dgm1_from_the_wcs() {
        assert_eq!(
            DGM1_WCS.get_coverage_url(&SAMPLE),
            "https://opendata.geoservices.lgln.niedersachsen.de/dgm_wcs?SERVICE=WCS&VERSION=2.0.1\
             &REQUEST=GetCoverage&COVERAGEID=ni_dgm1&FORMAT=image/tiff\
             &SUBSET=x(551800,552000)&SUBSET=y(5803350,5803400)"
        );
        assert_eq!(
            Model::Dom1.wcs().get_coverage_url(&SAMPLE),
            "https://opendata.geoservices.lgln.niedersachsen.de/dom_wcs?SERVICE=WCS&VERSION=2.0.1\
             &REQUEST=GetCoverage&COVERAGEID=ni_dom1&FORMAT=image/tiff\
             &SUBSET=x(551800,552000)&SUBSET=y(5803350,5803400)"
        );
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DGM1_WCS.url, WCS_DGM1.to_vec());
        fetcher.insert_prefix(DGM_STAC, STAC_DGM1.as_bytes().to_vec());
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("dgm1.tif");
        let found = fetch_model(
            &fetcher,
            FetchMode::Online,
            Model::Dgm1,
            &SAMPLE,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        // The date of the newest item at the centre (ALS of lot L1603).
        assert_eq!(
            found,
            HeightModel {
                date: Some("2016-04-18".to_string()),
                ..HeightModel::default()
            }
        );
        assert_eq!(
            fetcher.log.borrow()[0],
            DGM1_WCS.get_coverage_url(&SAMPLE),
            "one GetCoverage, then the STAC"
        );
        assert!(fetcher.log.borrow()[1].starts_with(&format!("{DGM_STAC}/search?bbox=")));
        // Written with its world file, as for the other per-area providers.
        let grid = crate::sources::float_tiff::stored_grid(&out).unwrap();
        assert_eq!((grid.width, grid.height), (200, 50));
        assert_eq!((grid.west, grid.north), (551_800.0, 5_803_400.0));
        let file = std::fs::File::open(&out).unwrap();
        let tiff::decoder::DecodingResult::F32(heights) =
            tiff::decoder::Decoder::new(std::io::BufReader::new(file))
                .unwrap()
                .read_image()
                .unwrap()
        else {
            panic!("not Float32");
        };
        // The school tile lies at 51.6–57.1 m (reference §5), without NoData; the live answer
        // decodes like the served one.
        assert!(
            heights.iter().all(|z| (51.0..58.0).contains(z)),
            "{:?}",
            heights.iter().find(|z| !(51.0..58.0).contains(*z))
        );
        assert_eq!(heights, decode_float_tiff(WCS_DGM1).unwrap().data);
        // An empty STAC answer: the model without a date, and a warning.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DGM1_WCS.url, WCS_DGM1.to_vec());
        fetcher.insert_prefix(DGM_STAC, stac_page(&[], None));
        let found = fetch_model(
            &fetcher,
            FetchMode::Online,
            Model::Dgm1,
            &SAMPLE,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        assert_eq!(found.date, None);
        assert_eq!(found.warnings.len(), 1, "{:?}", found.warnings);
        assert!(
            found.warnings[0].contains("no date"),
            "{:?}",
            found.warnings
        );
        // A failed STAC search: a warning as well.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(DGM1_WCS.url, WCS_DGM1.to_vec());
        let found = fetch_model(
            &fetcher,
            FetchMode::Online,
            Model::Dgm1,
            &SAMPLE,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        assert_eq!(found.date, None);
        assert!(
            found.warnings[0].contains("not found"),
            "{:?}",
            found.warnings
        );
        // No height at all (an area beyond the coverage's data): an error.
        let grid = crate::raster::GridSpec::covering(&SAMPLE, 1.0);
        let nodata = vec![-9999.0; (grid.width * grid.height) as usize];
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            DGM1_WCS.url,
            crate::fetch::test_support::geotiff(&grid, &nodata),
        );
        let empty = dir.path().join("empty.tif");
        let err = fetch_model(
            &fetcher,
            FetchMode::Online,
            Model::Dgm1,
            &SAMPLE,
            empty.parent().unwrap(),
            &empty,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("no height"), "{err}");
        assert!(!empty.exists());
        // Offline the stored model is kept when it covers the area; nothing is asked.
        let fetcher = MapFetcher::default();
        let found = fetch_model(
            &fetcher,
            FetchMode::Offline,
            Model::Dgm1,
            &SAMPLE,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        assert_eq!(found, HeightModel::default());
        assert_eq!(fetcher.calls.get(), 0);
        let larger = SAMPLE.buffered(10.0);
        let err = fetch_model(
            &fetcher,
            FetchMode::Offline,
            Model::Dgm1,
            &larger,
            out.parent().unwrap(),
            &out,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("run `freemapper fetch` online first"), "{err}");
    }
}
