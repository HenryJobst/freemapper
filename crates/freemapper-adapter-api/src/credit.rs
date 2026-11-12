//! Credit lines: what a project used of a region, and the lines the region asks for in return.

use chrono::{Datelike, NaiveDate};

use crate::records::{AdapterId, TileRecord, WcsRecord, WmsRecord};

/// One credit line of a region, as the licence of its data prescribes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditLine {
    /// The line for the map sheet and `ATTRIBUTION.txt`.
    pub text: String,
    /// The licence text the line refers to, for the project notes (`ATTRIBUTION.txt`) only. Lines
    /// with the same note share it: it follows the last of them, once.
    pub licence: Option<String>,
}

impl CreditLine {
    /// A line whose licence needs no note.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            licence: None,
        }
    }

    /// A line with the licence text it refers to.
    pub fn with_licence(text: impl Into<String>, licence: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            licence: Some(licence.into()),
        }
    }
}

/// The lines of `credits` for the project notes: every text in order, each licence note once,
/// after the last line that refers to it.
pub fn lines_with_licences(credits: &[CreditLine]) -> Vec<String> {
    let mut lines = Vec::new();
    for (at, credit) in credits.iter().enumerate() {
        lines.push(credit.text.clone());
        if let Some(licence) = &credit.licence
            && !credits[at + 1..]
                .iter()
                .any(|later| later.licence.as_ref() == Some(licence))
        {
            lines.push(licence.clone());
        }
    }
    lines
}

/// ALKIS objects a project stores: the regions whose services delivered them and the day of the
/// fetch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlkisUse {
    pub regions: Vec<AdapterId>,
    pub fetched: NaiveDate,
}

/// What a project holds, as far as the credit depends on it.
#[derive(Debug, Clone, Default)]
pub struct UsedSources<'a> {
    /// The provider of the project's rasters: that of the region the map area lies in.
    pub raster_provider: Option<AdapterId>,
    /// The regions the area of the rasters itself overlaps (without the fetch margin).
    pub overlapped: Vec<AdapterId>,
    /// The recorded tiles of the tile providers.
    pub tiles: &'a [TileRecord],
    pub orthophoto: Option<&'a WmsRecord>,
    pub terrain: Option<&'a WcsRecord>,
    /// The stored ALKIS objects (buildings, land use).
    pub alkis: Vec<AlkisUse>,
    /// The stored extra layers: the name of each and how many features it holds.
    pub extra_layers: Vec<(&'a str, usize)>,
    /// The extra templates the project holds (the K5 map of Berlin: `k5`).
    pub extra_templates: Vec<&'a str>,
    /// The registers whose points the map draws.
    pub drawn_registers: &'a [&'a str],
}

impl UsedSources<'_> {
    /// Whether the project's rasters come from `provider`.
    pub fn rasters_from(&self, provider: &str) -> bool {
        self.raster_provider
            .as_ref()
            .is_some_and(|id| id.as_str() == provider)
    }

    /// The days ALKIS objects of `region` were fetched.
    pub fn alkis_fetches<'a>(&'a self, region: &'a str) -> impl Iterator<Item = NaiveDate> + 'a {
        self.alkis
            .iter()
            .filter(move |record| record.regions.iter().any(|id| id.as_str() == region))
            .map(|record| record.fetched)
    }

    /// Whether the project holds ALKIS objects of `region`.
    pub fn has_alkis_of(&self, region: &str) -> bool {
        self.alkis_fetches(region).next().is_some()
    }

    /// The year of the newest fetch of the data of a per-area provider: its terrain model and
    /// orthophoto (when the project's rasters come from `provider`) and its ALKIS objects of
    /// `region`. `None`: the project holds nothing of it.
    pub fn newest_fetch_year(&self, provider: &str, region: &str) -> Option<i32> {
        let raster = self.rasters_from(provider);
        [
            self.terrain.filter(|_| raster).map(|r| r.fetched),
            self.orthophoto.filter(|_| raster).map(|r| r.fetched),
        ]
        .into_iter()
        .flatten()
        .chain(self.alkis_fetches(region))
        .map(|date| date.year())
        .max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(year: i32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, 6, 15).unwrap()
    }

    #[test]
    fn a_licence_note_follows_the_last_line_that_refers_to_it() {
        let credits = [
            CreditLine::new("plain"),
            CreditLine::with_licence("first", "CC BY"),
            CreditLine::with_licence("other", "dl-de"),
            CreditLine::with_licence("second", "CC BY"),
            CreditLine::new("last"),
        ];
        assert_eq!(
            lines_with_licences(&credits),
            [
                "plain", "first", "other", "dl-de", "second", "CC BY", "last"
            ]
        );
        assert!(lines_with_licences(&[]).is_empty());
    }

    #[test]
    fn the_year_is_that_of_the_newest_fetch_of_the_provider() {
        let (terrain, orthophoto) = (
            WcsRecord {
                fetched: day(2024),
                bbox: [0.0; 4],
                date: None,
            },
            WmsRecord {
                flight: None,
                flight_label: None,
                fetched: day(2025),
                bbox: [0.0; 4],
            },
        );
        let alkis = |region: &'static str, year| AlkisUse {
            regions: vec![AdapterId::built_in(region)],
            fetched: day(year),
        };
        let used = UsedSources {
            raster_provider: Some(AdapterId::built_in("lgv")),
            terrain: Some(&terrain),
            orthophoto: Some(&orthophoto),
            alkis: vec![alkis("hamburg", 2023), alkis("niedersachsen", 2026)],
            ..UsedSources::default()
        };
        // Rasters and ALKIS of the provider: the newest of them.
        assert_eq!(used.newest_fetch_year("lgv", "hamburg"), Some(2025));
        // The rasters are another provider's: the ALKIS objects of the region alone.
        assert_eq!(used.newest_fetch_year("lgln", "niedersachsen"), Some(2026));
        // Nothing of the provider.
        assert_eq!(used.newest_fetch_year("hvbg", "hessen"), None);
        assert!(used.has_alkis_of("hamburg") && !used.has_alkis_of("hessen"));
    }
}
