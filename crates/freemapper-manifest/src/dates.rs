//! `date` of a raster table: where the date of the raster is asked.

use chrono::NaiveDate;
use freemapper_adapter_api::fetch::Fetcher;
use freemapper_georef::crs::{BoundingBox, Crs};
use freemapper_sources::dates::{
    day_month_year, feature_info_date, newest_stac_item, wfs_feature_date,
};

use crate::model::{DateFormat, DateSource};

/// The date of a raster at the centre of `bbox` (in `crs`), written `DD.MM.YYYY` as the records
/// keep it; `None` when the table names no lookup or the service does not tell. `wms` is the
/// address and the CRS of the WMS of the orthophoto, which `wms-featureinfo` asks.
pub fn lookup(
    date: &DateSource,
    fetcher: &dyn Fetcher,
    wms: Option<(&str, Crs)>,
    crs: Crs,
    bbox: &BoundingBox,
) -> Option<String> {
    match date {
        DateSource::None => None,
        DateSource::WfsGeoJson {
            url,
            layer,
            field,
            format,
        } => {
            let value = wfs_feature_date(fetcher, (url, layer, field), crs, bbox)?;
            match format {
                // As the service writes it; the records of the states that have such a service
                // keep it so.
                DateFormat::DayMonthYear => Some(value),
                DateFormat::Iso => value
                    .get(..10)
                    .and_then(|day| NaiveDate::parse_from_str(day, "%Y-%m-%d").ok())
                    .map(day_month_year),
            }
        }
        DateSource::StacNewest { url } => newest_stac_item(fetcher, url, bbox, crs)
            .ok()
            .flatten()
            .map(|item| day_month_year(item.date)),
        DateSource::WmsFeatureInfo { layer, field } => {
            let (url, wms_crs) = wms?;
            feature_info_date(fetcher, (url, layer, field), wms_crs, bbox)
        }
    }
}
