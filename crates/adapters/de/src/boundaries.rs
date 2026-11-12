//! The boundaries of the states: the outline every question of "does this lie in the state"
//! is asked of. Each is the boundary file of the state's manifest (`manifests/<id>/boundary.geojson`),
//! embedded GeoJSON in EPSG:25833, simplified to 25 m, parsed on first use.

use std::sync::OnceLock;

use freemapper_adapter_api::geometry::{
    Outline, contains_point_in, tile_centre_near, tile_touches_in, tiles_touching,
    touches_region_in,
};
use geo_types::MultiPolygon;

use crate::crs::BoundingBox;
use crate::tiles::TileId;

// The ids of the adapters of the states.
pub const BRANDENBURG: &str = "brandenburg";
pub const BERLIN: &str = "berlin";
pub const SAXONY: &str = "saxony";
pub const SAXONY_ANHALT: &str = "saxonyanhalt";
pub const HAMBURG: &str = "hamburg";
pub const HESSEN: &str = "hessen";
pub const SCHLESWIG_HOLSTEIN: &str = "schleswigholstein";
pub const MECKLENBURG_VORPOMMERN: &str = "mecklenburgvorpommern";
pub const NORDRHEIN_WESTFALEN: &str = "nordrheinwestfalen";
pub const NIEDERSACHSEN: &str = "niedersachsen";
pub const RHEINLAND_PFALZ: &str = "rheinlandpfalz";
pub const SAARLAND: &str = "saarland";

/// Brandenburg from LGB ALKIS `verwaltungseinheit` DE12, with Berlin as a hole (dl-de/by-2-0).
const BRANDENBURG_BOUNDARY: &str = include_str!("../manifests/brandenburg/boundary.geojson");
/// Berlin boundary from ALKIS `alkis:land` (EPSG:25833, simplified 25 m; dl-de/zero-2-0).
const BERLIN_BOUNDARY: &str = include_str!("../manifests/berlin/boundary.geojson");
/// Saxony from GeoSN ALKIS `VerwaltungsEinheit` DE14 (dl-de/by-2-0).
const SAXONY_BOUNDARY: &str = include_str!("../manifests/saxony/boundary.geojson");
/// Saxony-Anhalt from LVermGeo DVG `VerwaltungsEinheit` DE15 (dl-de/by-2-0), in EPSG:25833 like
/// the others.
const SAXONY_ANHALT_BOUNDARY: &str = include_str!("../manifests/saxonyanhalt/boundary.geojson");
/// Hamburg from LGV `app:landesgrenze` (dl-de/by-2-0), with Neuwerk and Scharhörn.
const HAMBURG_BOUNDARY: &str = include_str!("../manifests/hamburg/boundary.geojson");
/// Hesse from HVBG ALKIS vereinfacht `VerwaltungsEinheit` DE06 (dl-de/zero-2-0). In EPSG:25833 it
/// has negative eastings (Hesse reaches from about -17 km to 167 km).
const HESSEN_BOUNDARY: &str = include_str!("../manifests/hessen/boundary.geojson");
/// Schleswig-Holstein from LVermGeo SH ALKIS (simplified 25 m, EPSG:25833; dl-de/by-2-0). It shares
/// about 131 km of border with Hamburg.
const SCHLESWIG_HOLSTEIN_BOUNDARY: &str =
    include_str!("../manifests/schleswigholstein/boundary.geojson");
/// Mecklenburg-Vorpommern from LAiV M-V ALKIS (simplified 25 m, EPSG:25833; CC BY 4.0). It shares
/// its southern border with Brandenburg, and stays in UTM zone 33 like it.
const MECKLENBURG_VORPOMMERN_BOUNDARY: &str =
    include_str!("../manifests/mecklenburgvorpommern/boundary.geojson");
/// North Rhine-Westphalia from Geobasis NRW ALKIS vereinfacht `VerwaltungsEinheit` DE05
/// (simplified 25 m, EPSG:25833; dl-de/zero-2-0), with the five Vennbahn exclaves. In EPSG:25833 it
/// has negative eastings (from about -140 km to 119 km).
const NORDRHEIN_WESTFALEN_BOUNDARY: &str =
    include_str!("../manifests/nordrheinwestfalen/boundary.geojson");
/// Lower Saxony from LGLN ALKIS vereinfacht `VerwaltungsEinheit` DE03 (simplified 25 m, EPSG:25833;
/// CC BY 4.0). Bremen and Bremerhaven are holes: they are not Niedersachsen.
const NIEDERSACHSEN_BOUNDARY: &str = include_str!("../manifests/niedersachsen/boundary.geojson");
/// Rhineland-Palatinate from the LVermGeo RP WFS `Verwaltungsgrenzen` (simplified 25 m, EPSG:25833;
/// dl-de/by-2-0). In EPSG:25833 it has negative eastings (84.5 % of the state). Wiesbaden and
/// Mainz-Kastel are Hesse, not part of it.
const RHEINLAND_PFALZ_BOUNDARY: &str = include_str!("../manifests/rheinlandpfalz/boundary.geojson");

/// Saarland from the LVGL WFS "ALKIS Vereinfacht", `Verwaltungseinheit` DE10 (simplified 25 m,
/// EPSG:25833; dl-de/by-2-0). In EPSG:25833 its eastings are all negative; three quarters of
/// its outline towards Rhineland-Palatinate lie on that state's outline.
const SAARLAND_BOUNDARY: &str = include_str!("../manifests/saarland/boundary.geojson");

/// The states in the order of the registry, each with its boundary.
const STATES: [(&str, &str); 12] = [
    (BRANDENBURG, BRANDENBURG_BOUNDARY),
    (BERLIN, BERLIN_BOUNDARY),
    (SAXONY, SAXONY_BOUNDARY),
    (SAXONY_ANHALT, SAXONY_ANHALT_BOUNDARY),
    (HAMBURG, HAMBURG_BOUNDARY),
    (HESSEN, HESSEN_BOUNDARY),
    (SCHLESWIG_HOLSTEIN, SCHLESWIG_HOLSTEIN_BOUNDARY),
    (MECKLENBURG_VORPOMMERN, MECKLENBURG_VORPOMMERN_BOUNDARY),
    (NORDRHEIN_WESTFALEN, NORDRHEIN_WESTFALEN_BOUNDARY),
    (NIEDERSACHSEN, NIEDERSACHSEN_BOUNDARY),
    (RHEINLAND_PFALZ, RHEINLAND_PFALZ_BOUNDARY),
    (SAARLAND, SAARLAND_BOUNDARY),
];

/// The outline of a state: its boundary and the box around it.
pub struct StateOutline {
    boundary: MultiPolygon<f64>,
    bounds: BoundingBox,
}

impl Outline for StateOutline {
    fn boundary(&self) -> &MultiPolygon<f64> {
        &self.boundary
    }

    fn bounds(&self) -> BoundingBox {
        self.bounds
    }
}

/// The GeoJSON text of the boundary of the state `id`.
pub fn boundary_text(id: &str) -> Option<&'static str> {
    STATES
        .iter()
        .find(|(state, _)| *state == id)
        .map(|(_, text)| *text)
}

/// The outline of the state `id`, parsed once.
pub fn outline(id: &str) -> Option<&'static StateOutline> {
    static OUTLINES: [OnceLock<StateOutline>; 12] = [
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
    ];
    let index = STATES.iter().position(|(state, _)| *state == id)?;
    Some(OUTLINES[index].get_or_init(|| {
        use geo::BoundingRect as _;
        let boundary = parse_boundary(STATES[index].1);
        let rect = boundary.bounding_rect().expect("a boundary has an extent");
        StateOutline {
            bounds: BoundingBox {
                min_e: rect.min().x,
                min_n: rect.min().y,
                max_e: rect.max().x,
                max_n: rect.max().y,
            },
            boundary,
        }
    }))
}

fn own(id: &str) -> &'static StateOutline {
    outline(id).unwrap_or_else(|| panic!("no state {id}"))
}

/// Whether the centre of the 1 km zone-32 tile (EPSG:25832) lies in or near the state `id`, so
/// its provider must have the tile.
pub fn tile_centre_in(tile: TileId, id: &str) -> bool {
    tile_centre_near(own(id), tile)
}

/// Whether the 1 km zone-32 tile (EPSG:25832) touches the state `id` at all: every tile that
/// can exist for it.
pub fn tile_touches(tile: TileId, id: &str) -> bool {
    tile_touches_in(own(id), tile)
}

/// Whether the boundary of the state `id` lies within the region margin of `bbox` (EPSG:25833):
/// the rule by which an area needs the state.
pub fn touches(id: &str, bbox: &BoundingBox) -> bool {
    touches_region_in(own(id), bbox)
}

/// The states whose boundary lies within the region margin of `bbox` (EPSG:25833), in their
/// order.
pub fn states_touching(bbox: &BoundingBox) -> Vec<&'static str> {
    STATES
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| touches(id, bbox))
        .collect()
}

/// The tiles of a `km` grid over `bbox` (EPSG:25833) that touch one of the `states`: those the
/// provider of these states publishes.
pub fn provider_tiles(bbox: &BoundingBox, states: &[&str], km: u32) -> Vec<TileId> {
    let outlines: Vec<&StateOutline> = states.iter().map(|id| own(id)).collect();
    tiles_touching(&outlines, bbox, km)
}

/// Whether the point (EPSG:25833) lies inside the boundary of the state `id` itself.
pub fn contains_point(id: &str, easting: f64, northing: f64) -> bool {
    contains_point_in(own(id), easting, northing)
}

/// The MultiPolygon of the first feature of an embedded GeoJSON boundary asset.
pub fn parse_boundary(text: &str) -> MultiPolygon<f64> {
    let collection: geojson::FeatureCollection =
        text.parse().expect("embedded boundary is GeoJSON");
    let geometry = collection.features[0]
        .geometry
        .clone()
        .expect("embedded boundary has a geometry");
    MultiPolygon::try_from(geometry.value).expect("embedded boundary is a MultiPolygon")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_state_has_a_valid_outline() {
        for (id, _) in STATES {
            let outline = outline(id).unwrap();
            assert!(!outline.boundary().0.is_empty(), "{id}");
            let bounds = outline.bounds();
            assert!(
                bounds.min_e < bounds.max_e && bounds.min_n < bounds.max_n,
                "{id}"
            );
            assert!(boundary_text(id).is_some(), "{id}");
        }
        assert!(outline("bremen").is_none());
        // Hamburg city hall lies in Hamburg and not in Schleswig-Holstein.
        let (e, n) = (165_800.0, 5_945_300.0);
        assert_eq!(
            contains_point(HAMBURG, e, n),
            !contains_point(SCHLESWIG_HOLSTEIN, e, n)
        );
    }
}
