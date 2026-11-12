//! Full ALKIS from a NAS service. The reader moved to `freemapper-sources` as `nas` (ADR 0082);
//! this module keeps the old paths working and holds what picks a service by state.

pub use freemapper_sources::nas::*;

use crate::adapters::{Registry, registry};
use crate::crs::BoundingBox;
use crate::fetch::Fetcher;
use crate::project::MapKind;
use crate::sources::alkis::FetchArea;
use crate::sources::region::regions_for_in;
use freemapper_adapter_api::policy::PolicyFetcher;

/// The NAS layers `kind` uses.
pub fn nas_layers(kind: MapKind) -> impl Iterator<Item = &'static NasLayer> {
    freemapper_sources::nas::nas_layers(kind.contract())
}

/// The NAS services of the states `region_bbox` (EPSG:25833) touches.
pub fn nas_services_for(region_bbox: &BoundingBox) -> Vec<&'static NasService> {
    regions_for_in(registry(), region_bbox)
        .into_iter()
        .filter_map(|adapter| adapter.nas())
        .collect()
}

/// The NAS layers of `kind` for `area`, as [`fetch_layers_from`] fetches them from the services
/// of the states the area touches; none at all, and no request, when it touches no state with a
/// NAS service.
pub fn fetch_nas_layers(
    fetcher: &dyn Fetcher,
    area: &FetchArea,
    kind: MapKind,
    part: PartFile,
) -> Vec<FetchedNasLayer> {
    fetch_nas_layers_in(registry(), fetcher, area, kind, part)
}

/// [`fetch_nas_layers`] with the adapters of `registry`. The service of an adapter that is held
/// to its declared hosts ([`crate::adapters::RegionAdapter::policy`]) is asked through that policy.
fn fetch_nas_layers_in(
    registry: &Registry,
    fetcher: &dyn Fetcher,
    area: &FetchArea,
    kind: MapKind,
    part: PartFile,
) -> Vec<FetchedNasLayer> {
    let adapters: Vec<(&NasService, Option<PolicyFetcher<'_>>)> =
        regions_for_in(registry, &area.region_bbox)
            .into_iter()
            .filter_map(|adapter| {
                let held = adapter
                    .policy()
                    .map(|policy| PolicyFetcher::new(fetcher, policy));
                Some((adapter.nas()?, held))
            })
            .collect();
    let services: Vec<(&NasService, &dyn Fetcher)> = adapters
        .iter()
        .map(|(service, held)| {
            let through = held.as_ref().map_or(fetcher, |held| held as &dyn Fetcher);
            (*service, through)
        })
        .collect();
    fetch_layers_through(&services, area, kind.contract(), part)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::test_support::*;
    use super::*;
    use crate::crs::Crs;
    use crate::fetch::FetchError;
    use crate::fetch::test_support::MapFetcher;
    #[allow(unused_imports)]
    use crate::sources::region::ids::{IdFacts as _, regions};
    use crate::sources::region::regions_for;

    #[test]
    fn nas_layers_only_in_nas_states() {
        let files = |kind: MapKind, bbox: BoundingBox| {
            let fetcher = MapFetcher::default();
            fetcher.insert_prefix(&LGB_NAS.url, EMPTY.as_bytes().to_vec());
            let scratch = tempfile::tempdir().unwrap();
            let part_path = scratch.path().join("answer.gml.part");
            let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
            let area = FetchArea::new(bbox, Crs::Utm33);
            let fetched = fetch_nas_layers(&fetcher, &area, kind, &part);
            let files: Vec<&str> = fetched
                .iter()
                .map(|l| {
                    let features = l.features.as_ref().unwrap();
                    assert!(features.features.is_empty());
                    assert_eq!(features.regions.len(), 1);
                    assert_eq!(features.regions[0].as_str(), regions::BRANDENBURG.id());
                    l.layer.file
                })
                .collect();
            (files, fetcher.calls.get())
        };
        let strausberg = BoundingBox {
            min_e: 421_000.0,
            min_n: 5_821_000.0,
            max_e: 422_000.0,
            max_n: 5_822_000.0,
        };
        let (sprint, calls) = files(MapKind::Sprint, strausberg);
        assert_eq!(
            sprint,
            [
                "sonstiges-bauwerk",
                "vegetationsmerkmal",
                "strukturlinie",
                "bauwerk-verkehr",
                "bauwerk-gewaesser",
                "untergeordnetes-gewaesser"
            ]
        );
        assert_eq!(calls, 6);
        // Embankment edges and bridges are sprint-only.
        let (schoolyard, calls) = files(MapKind::Schoolyard, strausberg);
        assert_eq!(
            schoolyard,
            [
                "sonstiges-bauwerk",
                "vegetationsmerkmal",
                "bauwerk-gewaesser",
                "untergeordnetes-gewaesser"
            ]
        );
        assert_eq!(calls, 4);
        // Berlin Mitte and Dresden: no NAS service, no request.
        let mitte = BoundingBox {
            min_e: 392_000.0,
            min_n: 5_820_000.0,
            max_e: 393_500.0,
            max_n: 5_821_000.0,
        };
        assert_eq!(regions_for(&mitte), [regions::BERLIN]);
        let dresden = BoundingBox {
            min_e: 415_800.0,
            min_n: 5_655_600.0,
            max_e: 416_150.0,
            max_n: 5_655_900.0,
        };
        for bbox in [mitte, dresden] {
            for kind in [MapKind::Sprint, MapKind::Schoolyard] {
                assert_eq!(files(kind, bbox), (Vec::new(), 0));
            }
        }
    }

    /// The NAS service of an adapter from a folder is asked through the adapter's policy: off
    /// the hosts it declares, no request leaves. A service of the program is asked as before.
    #[test]
    fn the_nas_service_of_a_folder_adapter_is_held_to_its_hosts() {
        use crate::adapters::test_support::{SQUARE, square};
        let service = NasService {
            url: "https://elsewhere.example.net/nas".into(),
            region: SQUARE,
            ..LGB_NAS
        };
        let area = FetchArea::new(
            BoundingBox {
                min_e: 4_000.0,
                min_n: 4_000.0,
                max_e: 5_000.0,
                max_n: 5_000.0,
            },
            Crs::Utm33,
        );
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("answer.gml.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        // What each layer came to, and the requests it took.
        let fetched = |hosts: Option<&[&str]>| {
            let registry = square(Some(service.clone()), hosts);
            let fetcher = MapFetcher::default();
            fetcher.insert_prefix("https://", EMPTY.as_bytes().to_vec());
            let layers = fetch_nas_layers_in(&registry, &fetcher, &area, MapKind::Sprint, &part);
            assert_eq!(layers.len(), 6);
            let refused = layers
                .iter()
                .filter(|layer| matches!(layer.features, Err(FetchError::NotAllowed { .. })))
                .count();
            let fine = layers.iter().filter(|l| l.features.is_ok()).count();
            (refused, fine, fetcher.calls.get())
        };
        assert_eq!(fetched(Some(&["geo.example.org"])), (6, 0, 0));
        assert_eq!(fetched(Some(&["elsewhere.example.net"])), (0, 6, 6));
        assert_eq!(fetched(None), (0, 6, 6));
    }
}
