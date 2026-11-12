//! Tests of the engine's synthetic LVermGeo SH services (Schleswig-Holstein) against the
//! reader in `freemapper-adapters-de`.

use freemapper_adapters_de::lvermgeo_sh::*;

mod tests {
    use super::*;
    use crate::crs::BoundingBox;
    use crate::fetch::test_support::MapFetcher;
    use crate::fetch::{Cache, FetchMode};

    const SCHOOL: BoundingBox = BoundingBox {
        min_e: 565_100.0,
        min_n: 5_949_100.0,
        max_e: 565_300.0,
        max_n: 5_949_300.0,
    };

    #[test]
    fn synthetic_schleswig_holstein_serves_both_models() {
        let fetcher = MapFetcher::default();
        crate::fetch::test_support::serve_schleswig_holstein_models(&fetcher, &SCHOOL);
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let mut heights = Vec::new();
        for model in [Model::Dgm1, Model::Bdom] {
            let out = dir.path().join(format!("{model:?}.tif"));
            let found = fetch_model(
                &fetcher,
                &cache,
                FetchMode::Online,
                model,
                &SCHOOL,
                out.parent().unwrap(),
                &out,
                &mut |_| {},
            )
            .unwrap();
            assert!(found.missing.is_empty(), "{found:?}");
            assert!(found.date.is_some(), "{found:?}");
            heights.push(
                crate::derive::terrain::read_float_raster(&out)
                    .unwrap()
                    .get(10, 10),
            );
        }
        assert!((heights[1] - heights[0] - 12.0).abs() < 1e-3, "{heights:?}");
    }
}
