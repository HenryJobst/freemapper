// SPDX-License-Identifier: AGPL-3.0-or-later

//! What a region supplies: the trait an adapter implements.

use freemapper_georef::crs::{BoundingBox, Crs, LatLon};
use geo_types::MultiPolygon;

use crate::alkis::AlkisSource;
use crate::auth::AuthSpec;
use crate::credit::{CreditLine, UsedSources};
use crate::estimate::{EstimateArea, EstimateEnv};
use crate::extras::{ExtraSet, NasService, RasterTemplate, TreeDataset};
use crate::info::{Declared, RegionInfo};
use crate::permits::PermitSource;
use crate::pointcloud::PointCloudBackend;
use crate::policy::UrlPolicy;
use crate::rasters::RasterBackend;
use crate::records::EstimateItem;

/// Everything one state or country supplies: who it is and where it lies, its OSM extracts, its
/// ALKIS service, its rasters, its extra sources, the credit it asks for and what a map of it
/// downloads. The engine asks the adapters of the registry and names no region itself.
pub trait RegionAdapter: Send + Sync {
    /// Identity and static facts.
    fn info(&self) -> &RegionInfo;

    /// What the manifest of the adapter declares about itself. `None`: the adapter has no
    /// manifest.
    fn declared(&self) -> Option<&Declared> {
        None
    }

    /// The hosts every request for this adapter is held to, redirects included: an adapter
    /// asks the hosts its manifest declares and no other, whether it is built into the program
    /// or came from a folder. `None`: the adapter has no manifest and asks as its code asks.
    fn policy(&self) -> Option<&UrlPolicy> {
        None
    }

    /// The login the region's downloads need: the host, the variables that hold it and what to
    /// order. `None`, as for most: the region's services are open.
    fn auth(&self) -> Option<&AuthSpec> {
        None
    }

    /// The CRS of a map whose centre lies at `centre` in this region. The default is the one CRS
    /// of the region ([`RegionInfo::crs`]); a region that spans several zones (Poland) answers by
    /// position. A map stays in this one CRS, also where its area reaches into the next zone.
    fn crs_for(&self, _centre: LatLon) -> Crs {
        self.info().crs
    }

    /// The boundary in EPSG:25833, parsed once and valid.
    fn boundary(&self) -> &MultiPolygon<f64>;

    /// The boundary (EPSG:25833) of the OSM extract `index` of [`RegionInfo::osm`], for a region
    /// with several extracts: a map reads those whose boundary lies near it. `None`: the extract
    /// has no boundary of its own and is always read.
    fn extract_boundary(&self, _index: usize) -> Option<&MultiPolygon<f64>> {
        None
    }

    /// The ALKIS service of the region: buildings and land use as features in the LGB schema.
    /// `None`: the region has no ALKIS data to offer.
    fn alkis(&self) -> Option<&dyn AlkisSource>;

    /// The NAS services of the region: its full ALKIS, for the structures ALKIS vereinfacht
    /// leaves out, and its landscape model (ADR 0135); a state may serve both. Empty: the region
    /// has none.
    fn nas_services(&self) -> &[NasService] {
        &[]
    }

    /// The tree cadastres of cities in the region.
    fn tree_datasets(&self) -> &[TreeDataset] {
        &[]
    }

    /// The sets of extra layers the adapter fetches with code of its own; empty for most.
    fn extras(&self) -> &[Box<dyn ExtraSet>] {
        &[]
    }

    /// The services of the region that give findings for the permit notes (ADR 0120); empty
    /// for most. The engine asks them only for permit notes of a map area that touches the
    /// region, behind [`Self::policy`].
    fn permit_sources(&self) -> &[Box<dyn PermitSource>] {
        &[]
    }

    /// The id of the warning for a map area on the ground of the region whose project has no
    /// building stored, a text of the catalog of this crate. `None`, as for most: the region
    /// has nothing to say then. For a region whose buildings come from a service of their own
    /// and can be missing while the rest arrived.
    fn no_buildings_warning(&self) -> Option<&'static str> {
        None
    }

    /// Whether the buildings of the region come from OpenStreetMap: it has no register of them
    /// (or the adapter does not read it yet). A fetch then keeps the `building` ways and
    /// relations of the OSM extract and stores them as the buildings of the project, and asks
    /// no ALKIS service for buildings. `false`, as for most.
    fn buildings_from_osm(&self) -> bool {
        false
    }

    /// The raster templates the adapter fetches with code of its own; empty for most.
    fn templates(&self) -> &[Box<dyn RasterTemplate>] {
        &[]
    }

    /// The credit lines for what a project holds of this region, in the order they are printed;
    /// empty before anything of it was fetched.
    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine>;

    /// The raster sources of the region. `None`: the region has ALKIS but no rasters, and no
    /// map can have its centre in it.
    fn rasters(&self) -> Option<&dyn RasterBackend>;

    /// The raster downloads of a map of `area` whose rasters come from this region (what, MB,
    /// cached, part, store), in the order they are listed.
    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem>;

    /// The classified laser point clouds of the region, for the vegetation classes. `None`: the
    /// region publishes none.
    fn point_cloud(&self) -> Option<&dyn PointCloudBackend> {
        None
    }

    /// The box around the boundary (EPSG:25833). The region functions ask it before the
    /// boundary itself: what lies outside the box cannot touch the region. An adapter that
    /// keeps the box answers faster than this default, which goes through the boundary.
    fn bounds(&self) -> BoundingBox {
        let mut bounds = BoundingBox {
            min_e: f64::INFINITY,
            min_n: f64::INFINITY,
            max_e: f64::NEG_INFINITY,
            max_n: f64::NEG_INFINITY,
        };
        for polygon in &self.boundary().0 {
            for point in polygon.exterior() {
                bounds.min_e = bounds.min_e.min(point.x);
                bounds.min_n = bounds.min_n.min(point.y);
                bounds.max_e = bounds.max_e.max(point.x);
                bounds.max_n = bounds.max_n.max(point.y);
            }
        }
        bounds
    }
}
