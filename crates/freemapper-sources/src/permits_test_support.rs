//! What the tests of the BfN and VG250 readers share: the Strausberg box, a centre of a
//! geometry and, for the ignored live tests, a fetcher that asks with `curl`.

use std::process::Command;

use geo_types::MultiPolygon;

use crate::fetch::{FetchError, Fetcher};
use crate::geojson_wfs::LatLonBox;

/// Around the Herrensee in Strausberg: 2 NSG of the BfN, the Gemeinde Strausberg of VG250.
pub(crate) const STRAUSBERG: LatLonBox = LatLonBox {
    min_lat: 52.53,
    min_lon: 13.83,
    max_lat: 52.545,
    max_lon: 13.86,
};

/// The middle of the bounding box of `geometry`, as (x, y).
pub(crate) fn centre(geometry: &MultiPolygon<f64>) -> (f64, f64) {
    let points = geometry
        .0
        .iter()
        .flat_map(|polygon| polygon.exterior().0.iter());
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in points {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
}

/// Asks over HTTP with `curl` and the User-Agent of freemapper, nothing else.
pub(crate) struct CurlFetcher;

pub(crate) fn live_fetcher() -> CurlFetcher {
    CurlFetcher
}

impl Fetcher for CurlFetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let agent = concat!(
            "freemapper/",
            env!("CARGO_PKG_VERSION"),
            " (+",
            env!("CARGO_PKG_REPOSITORY"),
            ")"
        );
        let output = Command::new("curl")
            .args(["-sS", "-f", "--max-time", "120", "-A", agent, url])
            .output()
            .map_err(|e| FetchError::Http {
                url: url.to_string(),
                message: e.to_string(),
            })?;
        if !output.status.success() {
            return Err(FetchError::Http {
                url: url.to_string(),
                message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }
        Ok(output.stdout)
    }
}
