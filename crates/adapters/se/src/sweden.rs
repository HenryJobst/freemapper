// SPDX-License-Identifier: AGPL-3.0-or-later

//! The adapter `sweden`: Sweden as one region. Everything the manifest can say is its manifest
//! (`manifests/sweden/manifest.toml`: hosts, extract, credit, login, the addresses of the
//! catalogues); what it cannot say is code here: the terrain model ([`crate::stac_terrain`]), the
//! laser data ([`crate::copc`]) and the land cover ([`crate::marktacke`]). Whatever is asked,
//! by the manifest or by the code, stays on the hosts the manifest declares.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::alkis::AlkisSource;
use freemapper_adapter_api::auth::AuthSpec;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};
use freemapper_adapter_api::error::Remedy;
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::info::{Country, Declared, RegionInfo};
use freemapper_adapter_api::pointcloud::PointCloudBackend;
use freemapper_adapter_api::policy::UrlPolicy;
use freemapper_adapter_api::rasters::{RasterBackend, RasterConfig};
use freemapper_adapter_api::records::EstimateItem;
use freemapper_adapter_api::registry::Registry;
use freemapper_georef::crs::BoundingBox;
use freemapper_manifest::adapter::{AdapterFiles, ManifestAdapter, Parse};
use freemapper_manifest::guard::{GuardedAlkis, GuardedPointCloud};
use freemapper_manifest::model::Alkis;
use geo_types::MultiPolygon;

use crate::copc::CopcBackend;
use crate::marktacke::MarktackeSource;
use crate::stac_terrain::{LantmaterietBackend, PROVIDER};

/// The id of the adapter.
pub const SWEDEN: &str = "sweden";

/// The manifest and the boundary of the adapter, built into the program.
const MANIFEST: &str = include_str!("../manifests/sweden/manifest.toml");
const BOUNDARY: &str = include_str!("../manifests/sweden/boundary.geojson");

/// The files of the adapter folder that the program carries.
struct Files;

impl AdapterFiles for Files {
    fn read(&self, name: &str) -> Result<Cow<'static, str>, String> {
        match name {
            "boundary.geojson" => Ok(Cow::Borrowed(BOUNDARY)),
            _ => Err("the file is not built in".to_string()),
        }
    }
}

/// Sweden as one region: the manifest, and the sources its tables name but cannot read.
pub struct SwedenAdapter {
    manifest: ManifestAdapter,
    /// The facts of the manifest, with the country: a manifest says Germany for every adapter.
    info: RegionInfo,
    /// The land cover, from `[alkis] kind = "stac-gpkg"`.
    alkis: Box<dyn AlkisSource>,
    /// The laser data, from `[point_cloud] kind = "stac-copc"`.
    point_cloud: Box<dyn PointCloudBackend>,
}

impl SwedenAdapter {
    /// The adapter with the hosts in `extra` allowed besides those of the manifest: what a person
    /// allowed after a service moved (ADR 0110). `backends` holds the raster backend of the
    /// manifest (`register_backends`).
    pub fn with_hosts(backends: &Registry, extra: &[String]) -> Self {
        let broken = |findings: Vec<freemapper_manifest::ManifestError>| -> ! {
            let lines: Vec<String> = findings.iter().map(ToString::to_string).collect();
            panic!("the built-in manifest of Sweden: {}", lines.join("; "))
        };
        let manifest = freemapper_manifest::parse(MANIFEST, Path::new(SWEDEN))
            .unwrap_or_else(|findings| broken(findings));
        let manifest = ManifestAdapter::new(manifest, &Files, Parse::OnFirstUse, backends)
            .unwrap_or_else(|findings| broken(findings))
            .held_to_its_hosts(Remedy::AllowHost, extra);
        let policy = Arc::new(
            manifest
                .policy()
                .expect("the manifest was just bound to its hosts")
                .clone(),
        );
        let Some(Alkis::StacGpkg { stac, collection }) = manifest.manifest().alkis.clone() else {
            panic!("the manifest of Sweden declares its land cover as `stac-gpkg`");
        };
        let cloud = manifest
            .manifest()
            .point_cloud
            .clone()
            .expect("the manifest of Sweden declares its point cloud");
        let info = RegionInfo {
            country: Country::Se,
            ..manifest.info().clone()
        };
        Self {
            info,
            alkis: Box::new(GuardedAlkis {
                inner: Box::new(MarktackeSource::new(&stac, &collection)),
                policy: policy.clone(),
            }),
            point_cloud: Box::new(GuardedPointCloud {
                inner: Box::new(CopcBackend::new(&cloud.url)),
                policy,
            }),
            manifest,
        }
    }
}

impl RegionAdapter for SwedenAdapter {
    fn info(&self) -> &RegionInfo {
        &self.info
    }

    fn declared(&self) -> Option<&Declared> {
        self.manifest.declared()
    }

    fn policy(&self) -> Option<&UrlPolicy> {
        self.manifest.policy()
    }

    fn auth(&self) -> Option<&AuthSpec> {
        self.manifest.auth()
    }

    fn boundary(&self) -> &MultiPolygon<f64> {
        self.manifest.boundary()
    }

    fn extract_boundary(&self, index: usize) -> Option<&MultiPolygon<f64>> {
        self.manifest.extract_boundary(index)
    }

    fn bounds(&self) -> BoundingBox {
        self.manifest.bounds()
    }

    fn alkis(&self) -> Option<&dyn AlkisSource> {
        Some(self.alkis.as_ref())
    }

    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine> {
        self.manifest.attribution(used)
    }

    fn rasters(&self) -> Option<&dyn RasterBackend> {
        self.manifest.rasters()
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        self.manifest.estimate(area, env)
    }

    fn point_cloud(&self) -> Option<&dyn PointCloudBackend> {
        Some(self.point_cloud.as_ref())
    }
}

/// Makes the raster backend `lantmateriet` known to `registry`, to be built from the endpoints
/// of the manifest.
fn register_backends(
    registry: &mut Registry,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    registry.add_raster_factory(
        PROVIDER,
        Arc::new(|config: &RasterConfig| {
            LantmaterietBackend::from_config(config)
                .map(|backend| Arc::new(backend) as Arc<dyn RasterBackend>)
        }),
    )?;
    Ok(())
}

/// Registers Sweden.
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    register_with_hosts(registry, &BTreeMap::new())
}

/// [`register`] with the hosts a person allowed for the adapter, by its id.
pub fn register_with_hosts(
    registry: &mut Registry,
    extra: &BTreeMap<String, Vec<String>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    register_backends(registry)?;
    let hosts = extra.get(SWEDEN).map_or(&[][..], Vec::as_slice);
    registry.add(Arc::new(SwedenAdapter::with_hosts(registry, hosts)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use freemapper_adapter_api::adapter::RegionAdapter;
    use freemapper_adapter_api::credit::{AlkisUse, UsedSources};
    use freemapper_adapter_api::registry::Registry;
    use freemapper_georef::crs::{Crs, LatLon};
    use geo::Contains;
    use geo_types::Point;

    use super::*;

    fn registry() -> Registry {
        let mut registry = Registry::new();
        register(&mut registry).unwrap();
        registry
    }

    fn adapter(registry: &Registry) -> &dyn RegionAdapter {
        registry.get(SWEDEN).expect("sweden is registered")
    }

    fn inside(adapter: &dyn RegionAdapter, lat: f64, lon: f64) -> bool {
        let p = Crs::Utm33.forward(LatLon { lat, lon });
        adapter
            .boundary()
            .contains(&Point::new(p.easting, p.northing))
    }

    #[test]
    fn the_adapter_is_registered_with_its_hosts_and_a_policy() {
        let registry = registry();
        let adapter = adapter(&registry);
        assert_eq!(adapter.info().id.as_str(), SWEDEN);
        assert_eq!(adapter.info().crs, Crs::Utm33);
        assert_eq!(
            adapter.info().country,
            freemapper_adapter_api::info::Country::Se
        );
        let declared = adapter.declared().expect("a manifest adapter");
        assert_eq!(
            declared.hosts,
            ["api.lantmateriet.se", "dl1.lantmateriet.se"]
        );
        let policy = adapter.policy().expect("held to its hosts");
        assert!(
            policy
                .check_url("https://dl1.lantmateriet.se/hojd/x.tif")
                .is_ok()
        );
        assert!(policy.check_url("https://evil.example.net/x").is_err());
    }

    #[test]
    fn a_host_a_person_allowed_is_added_to_the_policy() {
        let mut registry = Registry::new();
        let extra = std::collections::BTreeMap::from([(
            SWEDEN.to_string(),
            vec!["mirror.example.org".to_string()],
        )]);
        register_with_hosts(&mut registry, &extra).unwrap();
        let policy = adapter(&registry).policy().unwrap().clone();
        assert!(policy.check_url("https://mirror.example.org/x").is_ok());
    }

    #[test]
    fn it_asks_for_a_login_on_the_download_host() {
        let registry = registry();
        let spec = adapter(&registry).auth().expect("a login");
        assert_eq!(spec.host, "dl1.lantmateriet.se");
        assert_eq!(spec.user_env, "FREEMAPPER_LM_USER");
        assert_eq!(spec.password_env, "FREEMAPPER_LM_PASSWORD");
        assert_eq!(spec.products.len(), 3);
    }

    #[test]
    fn the_boundary_holds_swedish_towns_and_no_neighbour_capitals() {
        let registry = registry();
        let adapter = adapter(&registry);
        for (name, lat, lon) in [
            ("Stockholm", 59.33, 18.07),
            ("Malmö", 55.60, 13.00),
            ("Kiruna", 67.85, 20.23),
            ("Visby", 57.64, 18.30),
        ] {
            assert!(inside(adapter, lat, lon), "{name}");
        }
        for (name, lat, lon) in [
            ("Oslo", 59.91, 10.75),
            ("Helsinki", 60.17, 24.94),
            ("Copenhagen", 55.68, 12.57),
            ("Berlin", 52.52, 13.40),
        ] {
            assert!(!inside(adapter, lat, lon), "{name}");
        }
    }

    #[test]
    fn the_adapter_has_terrain_cloud_and_land_cover_and_no_buildings_warning() {
        let registry = registry();
        let adapter = adapter(&registry);
        let rasters = adapter.rasters().expect("the terrain backend");
        assert_eq!(rasters.provider().id.as_str(), "lantmateriet");
        let cloud = adapter.point_cloud().expect("the point clouds");
        assert_eq!(cloud.info().epsg, 25833);
        assert!(adapter.alkis().is_some());
        assert_eq!(adapter.no_buildings_warning(), None);
    }

    #[test]
    fn the_credit_names_lantmateriet_once_something_of_it_was_fetched() {
        let registry = registry();
        let adapter = adapter(&registry);
        assert!(adapter.attribution(&UsedSources::default()).is_empty());
        let used = UsedSources {
            alkis: vec![AlkisUse {
                regions: vec![SWEDEN.parse().unwrap()],
                fetched: NaiveDate::from_ymd_opt(2026, 11, 27).unwrap(),
            }],
            ..UsedSources::default()
        };
        let lines = adapter.attribution(&used);
        assert_eq!(lines.len(), 1);
        assert!(
            format!("{lines:?}").contains("laser data: terms of Geotorget"),
            "{lines:?}"
        );
    }
}
