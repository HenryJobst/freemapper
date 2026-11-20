// SPDX-License-Identifier: AGPL-3.0-or-later

//! The sheets of the Polish height models and orthophotos, and which one of several to use.
//!
//! GUGiK delivers per sheet (`godlo`) and flight year, and the quality differs from sheet to
//! sheet: LiDAR or photogrammetry, 1 m or 5 m, an old flight or a new one. The index of the
//! service lists every sheet of every year; [`best_per_cell`] keeps the best one per sheet.

use freemapper_georef::crs::BoundingBox;

/// What a sheet holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Product {
    /// Digital terrain model (NMT), heights of the ground.
    Terrain,
    /// Digital surface model (NMPT), heights of what covers the ground.
    Surface,
    /// Orthophoto.
    Orthophoto,
}

/// How the heights of a sheet were measured. The order is the order of preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SheetSource {
    Lidar,
    Photogrammetry,
    /// The index gives no source, or one this program does not know.
    Unknown,
}

/// One sheet of the index.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    /// The sheet number, such as `N-33-126-C-c-4-3`.
    pub godlo: String,
    pub year: u16,
    /// Pixel size in metres.
    pub resolution_m: f64,
    pub source: SheetSource,
    /// Where the file lies.
    pub url: String,
    /// The extent in PL-1992 (EPSG:2180).
    pub bbox_2180: BoundingBox,
}

/// The best sheet for every sheet number, best first: LiDAR over photogrammetry, a finer pixel
/// over a coarser one, a newer flight over an older one; what is still equal goes by the sheet
/// number and the address, so the same index always gives the same answer.
pub fn best_per_cell(sheets: &[Sheet]) -> Vec<Sheet> {
    let mut ranked: Vec<&Sheet> = sheets.iter().collect();
    ranked.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then(a.resolution_m.total_cmp(&b.resolution_m))
            .then(b.year.cmp(&a.year))
            .then_with(|| a.godlo.cmp(&b.godlo))
            .then_with(|| a.url.cmp(&b.url))
    });
    let mut seen = std::collections::HashSet::new();
    ranked
        .into_iter()
        .filter(|s| seen.insert(s.godlo.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet(godlo: &str, year: u16, resolution_m: f64, source: SheetSource) -> Sheet {
        Sheet {
            godlo: godlo.into(),
            year,
            resolution_m,
            source,
            url: format!("https://opendata.example/{godlo}/{year}/{resolution_m}"),
            bbox_2180: BoundingBox {
                min_e: 0.0,
                min_n: 0.0,
                max_e: 1.0,
                max_n: 1.0,
            },
        }
    }

    /// The year and resolution of the sheet that wins among `candidates`, all of one number.
    fn winner(candidates: &[Sheet]) -> (u16, f64) {
        let best = best_per_cell(candidates);
        assert_eq!(best.len(), 1, "{best:?}");
        (best[0].year, best[0].resolution_m)
    }

    #[test]
    fn choice_table() {
        use SheetSource::*;
        let cases: [(&str, Vec<Sheet>, (u16, f64)); 6] = [
            (
                "LiDAR 2020 over photogrammetry 2024",
                vec![
                    sheet("A", 2024, 1.0, Photogrammetry),
                    sheet("A", 2020, 1.0, Lidar),
                ],
                (2020, 1.0),
            ),
            (
                "1 m 2022 over 5 m 2025",
                vec![sheet("A", 2025, 5.0, Lidar), sheet("A", 2022, 1.0, Lidar)],
                (2022, 1.0),
            ),
            (
                "the newer of two equal ones",
                vec![sheet("A", 2019, 1.0, Lidar), sheet("A", 2023, 1.0, Lidar)],
                (2023, 1.0),
            ),
            (
                "a known source over an unknown one",
                vec![sheet("A", 2025, 1.0, Unknown), sheet("A", 2018, 1.0, Lidar)],
                (2018, 1.0),
            ),
            (
                "the source ranks above the pixel size",
                vec![
                    sheet("A", 2021, 0.5, Photogrammetry),
                    sheet("A", 2021, 1.0, Lidar),
                ],
                (2021, 1.0),
            ),
            (
                "a single candidate",
                vec![sheet("A", 2001, 5.0, Unknown)],
                (2001, 5.0),
            ),
        ];
        for (name, candidates, want) in cases {
            assert_eq!(winner(&candidates), want, "{name}");
        }
    }

    #[test]
    fn nothing_in_nothing_out() {
        assert!(best_per_cell(&[]).is_empty());
    }

    #[test]
    fn every_sheet_number_keeps_one_sheet_and_the_best_come_first() {
        use SheetSource::*;
        let all = [
            sheet("B", 2020, 1.0, Photogrammetry),
            sheet("A", 2020, 1.0, Lidar),
            sheet("B", 2022, 1.0, Lidar),
            sheet("A", 2018, 5.0, Lidar),
        ];
        let best = best_per_cell(&all);
        assert_eq!(best.len(), 2);
        // LiDAR 1 m 2022 (B) ranks above LiDAR 1 m 2020 (A).
        assert_eq!(
            best.iter()
                .map(|s| (s.godlo.as_str(), s.year))
                .collect::<Vec<_>>(),
            [("B", 2022), ("A", 2020)]
        );
    }

    #[test]
    fn a_tie_goes_by_the_address_and_does_not_depend_on_the_order_of_the_index() {
        use SheetSource::*;
        let mut a = sheet("A", 2021, 1.0, Lidar);
        let mut b = sheet("A", 2021, 1.0, Lidar);
        a.url = "https://opendata.example/a".into();
        b.url = "https://opendata.example/b".into();
        let forward = best_per_cell(&[a.clone(), b.clone()]);
        let backward = best_per_cell(&[b, a]);
        assert_eq!(forward, backward);
        assert_eq!(forward[0].url, "https://opendata.example/a");
    }
}
