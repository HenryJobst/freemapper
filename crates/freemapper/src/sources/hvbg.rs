//! Moved to `freemapper-adapters-de` as `hvbg` (ADR 0082); this module keeps the old paths working.

pub use freemapper_adapters_de::hvbg::*;

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::crs::BoundingBox;
    use crate::fetch::test_support::MapFetcher;
    use crate::fetch::{Cache, FetchMode};

    const SCHOOL: BoundingBox = BoundingBox {
        min_e: 476_800.0,
        min_n: 5_552_500.0,
        max_e: 477_100.0,
        max_n: 5_552_600.0,
    };

    #[test]
    fn synthetic_hessen_serves_both_models() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        crate::fetch::test_support::serve_synthetic_hessen(&fetcher, &SCHOOL);
        let mut rasters = Vec::new();
        for model in [Model::Dgm1, Model::Dom1] {
            let out = dir.path().join(format!("{}.tif", model.code()));
            let fetched = fetch_model(
                &fetcher,
                &cache,
                FetchMode::Online,
                model,
                &SCHOOL,
                out.parent().unwrap(),
                &out,
            )
            .unwrap();
            assert_eq!(fetched.missing, []);
            assert_eq!(fetched.date.as_deref(), Some("2023-02-28"));
            rasters.push(crate::derive::terrain::read_float_raster(&out).unwrap());
        }
        // The list is linked from the DGM1 tree only and read once for both models.
        let reads = |url: &str| fetcher.log.borrow().iter().filter(|u| *u == url).count();
        assert_eq!(reads(METADATA_URL), 1);
        fetch_model(
            &fetcher,
            &cache,
            FetchMode::Offline,
            Model::Dom1,
            &SCHOOL,
            dir.path(),
            &dir.path().join("o.tif"),
        )
        .unwrap();
        // E 476 810: 810 m into the tile.
        assert!((rasters[0].get(10, 50) - 108.1).abs() < 1e-3);
        assert!((rasters[1].get(10, 50) - rasters[0].get(10, 50) - 12.0).abs() < 1e-3);
    }
}
