//! `[[extra.nas]]` and `[[extra.trees]]` of a manifest as the services of the readers.

use std::borrow::Cow;

use freemapper_adapter_api::extras::{NasService, TreeDataset, TreeFields, TreeFormat};
use freemapper_adapter_api::records::AdapterId;
use freemapper_georef::crs::BoundingBox;

use crate::model::{Nas, Trees};

fn owned_list(items: &[String]) -> Cow<'static, [Cow<'static, str>]> {
    Cow::Owned(items.iter().cloned().map(Cow::Owned).collect())
}

/// The full ALKIS (NAS) service of the region `region`.
pub fn nas_service(nas: &Nas, region: &AdapterId) -> NasService {
    NasService {
        name: Cow::Owned(nas.name.clone()),
        url: Cow::Owned(nas.url.clone()),
        region: region.clone(),
        crs: nas.crs,
        max_count: nas.max_count as usize,
        max_bytes: nas.max_bytes,
        layers: nas.layers.as_deref().map(owned_list),
        namespaces: nas.namespaces.clone().map(Cow::Owned),
    }
}

/// The tree cadastre of a city in the region `region`.
pub fn tree_dataset(trees: &Trees, region: &AdapterId) -> TreeDataset {
    let [min_e, min_n, max_e, max_n] = trees.envelope;
    TreeDataset {
        id: Cow::Owned(trees.id.clone()),
        name: Cow::Owned(trees.name.clone()),
        region: region.clone(),
        crs: trees.crs,
        envelope: BoundingBox {
            min_e,
            min_n,
            max_e,
            max_n,
        },
        format: TreeFormat::OdsGeoJsonExport {
            geo_field: Cow::Owned(trees.geo_field.clone()),
        },
        endpoints: owned_list(&trees.endpoints),
        fields: TreeFields {
            kind: Cow::Owned(trees.fields.kind.clone()),
            id: Cow::Owned(trees.fields.id.clone()),
            date: Cow::Owned(trees.fields.date.clone()),
            species: trees.fields.species.clone().map(Cow::Owned),
            crown: trees.fields.crown.clone().map(Cow::Owned),
        },
        skipped_kinds: owned_list(&trees.skipped_kinds),
        attribution: Cow::Owned(trees.attribution.clone()),
        max_bytes: trees.max_bytes,
    }
}
