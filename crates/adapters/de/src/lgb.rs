// SPDX-License-Identifier: AGPL-3.0-or-later

//! Raster products of all providers (LGB Brandenburg, GeoSN Saxony; dl-de/by-2-0), tile URLs,
//! and the LGB data currency.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::NaiveDate;
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, tile_names};
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{
    DeriveEnv, DeriveInputs, ProviderInfo, RasterArea, RasterBackend, RasterConfig, RasterKind,
    StoredRasters,
};
use freemapper_adapter_api::records::{AdapterId, NdsmRecord, TileRecord};
use freemapper_sources::raster_fetch::{drop_wms_orthophoto, raster_tiles, say_kept_surface_tiles};

use crate::crs::BoundingBox;
use crate::note::{Arg, note};
use crate::tile_fetch::{
    TilePaths as _, TileProducts, fetch_ndsm_tiles_at, fetch_tiles_at, fetch_tiles_each_at,
    tile_inputs,
};
use crate::tiles::TileId;

/// GeoSN GeoCloud link shares (Nextcloud); the tokens change now and then (§2.1 of the Saxony
/// reference), so the tile index is asked first.
const GEOSN_SHARES: &str = "https://geocloud.landesvermessung.sachsen.de/public.php/dav/files";
const GEOSN_DGM1_SHARE: &str = "JCcXyifaNdLDnxZ";
const GEOSN_DOM1_SHARE: &str = "S6wwnFwX7882sZm";

/// The two folders LGB serves its tiles and their currency lists from (issue #273): where the
/// `[orthophoto]` of Brandenburg and Berlin says they are. The code keeps the names of the files
/// below them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addresses {
    /// The folder of the tile archives and the nDOM point clouds.
    pub data: String,
    /// The folder of the currency lists (CSV).
    pub currency: String,
}

impl Addresses {
    /// The addresses `config` states.
    pub fn of(config: &RasterConfig) -> Result<Self, String> {
        Ok(Self {
            data: config.require("lgb", "data")?,
            currency: config.require("lgb", "currency")?,
        })
    }

    /// The addresses the manifest of Brandenburg declares.
    pub fn builtin() -> Self {
        static BUILTIN: std::sync::OnceLock<Addresses> = std::sync::OnceLock::new();
        BUILTIN
            .get_or_init(|| {
                Self::of(&crate::states::builtin_raster_config(
                    crate::states::State::Brandenburg,
                ))
                .expect("the built-in manifest of Brandenburg names the addresses of LGB")
            })
            .clone()
    }

    /// Download URL of a tile of an LGB `product`.
    fn tile_url(&self, product: Product, t: TileId) -> String {
        let (folder, format) = match product {
            Product::Orthophoto => ("dop", "rgb_jpg"),
            Product::Dtm => ("dgm", "tif"),
            _ => ("bdom", "tif"),
        };
        format!("{}/{folder}/{format}/{}.zip", self.data, stem(product, t))
    }

    /// The currency CSV of an LGB `product`.
    fn currency_csv_url(&self, product: Product) -> String {
        let file = match product {
            Product::Orthophoto => "bb_dop_aktualitaet.csv",
            Product::Dtm => "bb_dgm_aktualitaet.csv",
            _ => "bb_bdom_aktualitaet.csv",
        };
        format!("{}/{file}", self.currency)
    }

    /// URL of the nDSM point cloud of `t` (LAS 1.2, compressed; no zip, no currency CSV).
    pub fn ndsm_tile_url(&self, t: TileId) -> String {
        format!("{}/ndom/NDOM_{}_LAS12.laz", self.data, t.name())
    }

    /// The download URL of a tile of `product`: the one of the folder `self` names for an LGB
    /// product, the fallback of the GeoSN share for a Saxon one.
    pub fn tile_url_of(&self, product: Product, t: TileId) -> String {
        match product.provider() {
            TileProvider::Lgb => self.tile_url(product, t),
            TileProvider::GeoSn => product.tile_url(t),
        }
    }

    /// The URL of the currency list of `product`: a CSV of LGB, the index of GeoSN.
    pub fn currency_url_of(&self, product: Product) -> String {
        match product.provider() {
            TileProvider::Lgb => self.currency_csv_url(product),
            TileProvider::GeoSn => product.currency_csv_url().to_string(),
        }
    }
}

/// The provider of a tile product.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileProvider {
    /// LGB: Brandenburg and Berlin, 1 km tiles.
    Lgb,
    /// GeoSN: Saxony, 2 km tiles.
    GeoSn,
}

impl TileProvider {
    /// The provider id of the adapters, which is also the name of its folder in the cache.
    pub fn id(self) -> &'static str {
        match self {
            Self::Lgb => "lgb",
            Self::GeoSn => "geosn",
        }
    }
}

pub use freemapper_adapter_api::error::CurrencyCsvError;
pub use freemapper_adapter_api::records::{Product, TileCurrency};

/// What the tile fetch asks a [`Product`]: who serves it, where its tiles and its currency list
/// are, and how its files are named. The product itself lives in `freemapper-adapter-api`, because
/// the tile records name it; these answers name LGB and GeoSN and stay here until the tile
/// backends move (ADR 0082).
pub trait ProductTiles: Copy {
    /// Who serves the product.
    fn provider(self) -> TileProvider;

    /// Edge length of the product's tiles in km.
    fn grid_km(self) -> u32;

    /// Download URL of a tile. GeoSN rotates its share tokens, so its tiles are downloaded from
    /// the URL in the tile index; this one is the fallback.
    fn tile_url(self, t: TileId) -> String;

    /// The LGB currency CSV; GeoSN products take their currency from the tile index instead.
    /// An LGB product answers with the address of the built-in manifest.
    fn currency_csv_url(self) -> String;

    /// Name of the raster file inside the tile archive.
    fn raster_member(self, t: TileId) -> String;

    /// Name of the world file inside the tile archive.
    fn world_file_member(self, t: TileId) -> String;

    /// Folder of the product's provider in the cache.
    fn provider_dir_name(self) -> &'static str;

    /// Folder of this product in its provider's cache folder.
    fn cache_dir_name(self) -> &'static str;
}

impl ProductTiles for Product {
    fn provider(self) -> TileProvider {
        match self {
            Self::Orthophoto | Self::Dtm | Self::ImageDsm => TileProvider::Lgb,
            Self::SaxonyDtm | Self::SaxonyDsm => TileProvider::GeoSn,
        }
    }

    fn grid_km(self) -> u32 {
        match self.provider() {
            TileProvider::Lgb => 1,
            TileProvider::GeoSn => 2,
        }
    }

    fn tile_url(self, t: TileId) -> String {
        match self {
            Self::Orthophoto | Self::Dtm | Self::ImageDsm => Addresses::builtin().tile_url(self, t),
            Self::SaxonyDtm => format!(
                "{GEOSN_SHARES}/{GEOSN_DGM1_SHARE}/{}_tiff.zip",
                stem(self, t)
            ),
            Self::SaxonyDsm => format!(
                "{GEOSN_SHARES}/{GEOSN_DOM1_SHARE}/{}_tiff.zip",
                stem(self, t)
            ),
        }
    }

    fn currency_csv_url(self) -> String {
        match self {
            Self::Orthophoto | Self::Dtm | Self::ImageDsm => {
                Addresses::builtin().currency_csv_url(self)
            }
            Self::SaxonyDtm | Self::SaxonyDsm => crate::sources::geosn::INDEX_URL.to_string(),
        }
    }

    fn raster_member(self, t: TileId) -> String {
        let ext = match self {
            Self::Orthophoto => "jpg",
            _ => "tif",
        };
        format!("{}.{ext}", stem(self, t))
    }

    fn world_file_member(self, t: TileId) -> String {
        let ext = match self {
            Self::Orthophoto => "jgw",
            _ => "tfw",
        };
        format!("{}.{ext}", stem(self, t))
    }

    fn provider_dir_name(self) -> &'static str {
        self.provider().id()
    }

    fn cache_dir_name(self) -> &'static str {
        match self {
            Self::Orthophoto => "dop20rgb",
            Self::Dtm | Self::SaxonyDtm => "dgm1",
            Self::ImageDsm => "bdom",
            Self::SaxonyDsm => "dom1",
        }
    }
}

fn file_prefix(product: Product) -> &'static str {
    match product {
        Product::Orthophoto => "dop",
        Product::Dtm => "dgm",
        Product::ImageDsm => "bdom",
        Product::SaxonyDtm => "dgm1",
        Product::SaxonyDsm => "dom1",
    }
}

/// File stem of a tile: `dgm_33421-5821` (LGB), `dgm1_33414_5654_2_sn` (GeoSN).
fn stem(product: Product, t: TileId) -> String {
    match product.provider() {
        TileProvider::Lgb => format!("{}_{}", file_prefix(product), t.name()),
        TileProvider::GeoSn => format!(
            "{}_33{:03}_{:04}_2_sn",
            file_prefix(product),
            t.east_km,
            t.north_km
        ),
    }
}

/// First columns of an LGB currency CSV.
const CURRENCY_CSV_HEADER: &str = "sheetnr;sheetname;creationdate;revisiondate;publicationdate";

/// Parses an LGB currency CSV (`sheetnr;sheetname;creationdate;revisiondate;publicationdate;…`).
pub fn parse_currency_csv(text: &str) -> Result<HashMap<TileId, TileCurrency>, CurrencyCsvError> {
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    let header = lines.next().unwrap_or_default();
    if !header.starts_with(CURRENCY_CSV_HEADER) {
        return Err(CurrencyCsvError {
            line: 1,
            message: format!("unexpected header {header:?}"),
        });
    }
    let mut table = HashMap::new();
    for (index, line) in lines.enumerate() {
        let line_no = index + 2;
        if line.trim().is_empty() {
            continue;
        }
        let error = |message: String| CurrencyCsvError {
            line: line_no,
            message,
        };
        let fields: Vec<&str> = line.split(';').collect();
        if fields.len() < 5 {
            return Err(error(format!("expected 5 fields, found {}", fields.len())));
        }
        let tile = TileId::parse(fields[0].trim())
            .ok_or_else(|| error(format!("invalid tile id {:?}", fields[0])))?;
        let date = |field: &str| -> Result<Option<NaiveDate>, CurrencyCsvError> {
            let field = field.trim();
            if field.is_empty() {
                return Ok(None);
            }
            field
                .parse()
                .map(Some)
                .map_err(|_| error(format!("invalid date {field:?}")))
        };
        let created = date(fields[2])?.ok_or_else(|| error("missing creation date".into()))?;
        let revised = date(fields[3])?;
        let published = date(fields[4])?.ok_or_else(|| error("missing publication date".into()))?;
        table.insert(
            tile,
            TileCurrency {
                created,
                revised,
                published,
            },
        );
    }
    Ok(table)
}

/// URL of the nDSM point cloud of `t` in the built-in folder (LAS 1.2, compressed; no zip, no
/// currency CSV).
pub fn ndsm_tile_url(t: TileId) -> String {
    Addresses::builtin().ndsm_tile_url(t)
}

/// Attribution line required by dl-de/by-2-0.
pub fn attribution(year: i32) -> String {
    format!("© GeoBasis-DE/LGB {year}, dl-de/by-2-0, Daten geändert")
}

/// The LGB attribution for tiles holding Berlin data, as the LGB licence note asks.
pub fn attribution_with_berlin(year: i32) -> String {
    format!(
        "© GeoBasis-DE/LGB {year}, dl-de/by-2-0; © Geoportal Berlin, dl-de/zero-2-0, Daten geändert"
    )
}

/// The products LGB records for every tile of an area.
pub const PRODUCTS: [Product; 2] = [Product::Orthophoto, Product::Dtm];

/// The rasters of LGB (Brandenburg and Berlin): tiles of 1 km, kept in the cache.
pub struct LgbBackend {
    info: ProviderInfo,
    /// The folders of the tiles, as the manifest says them.
    addresses: Addresses,
}

impl LgbBackend {
    /// The backend that asks the folders at `addresses`.
    pub fn with_addresses(addresses: Addresses) -> Self {
        Self {
            addresses,
            info: ProviderInfo {
                id: AdapterId::built_in(TileProvider::Lgb.id()),
                cache_dir: Some(TileProvider::Lgb.id().to_string()),
                terrain_source: "lgb-dgm1".to_string(),
                kind: RasterKind::Tiles {
                    grid_km: Product::Dtm.grid_km(),
                },
            },
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl LgbBackend {
    /// The backend with the addresses the manifest of Brandenburg declares.
    pub fn new() -> Self {
        Self::with_addresses(Addresses::builtin())
    }
}

#[cfg(any(test, feature = "testing"))]
impl Default for LgbBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend for LgbBackend {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        fetch_rasters(env, project, bbox, &self.addresses)
    }

    fn derive_inputs(
        &self,
        env: &DeriveEnv<'_>,
        stored: &StoredRasters<'_>,
        area: &RasterArea,
    ) -> DeriveInputs {
        let products = TileProducts {
            required: &PRODUCTS,
            orthophoto: Some(Product::Orthophoto),
            dtm: Product::Dtm,
            dsm: Product::ImageDsm,
        };
        tile_inputs(TileProvider::Lgb.id(), &products, env, stored, area)
    }
}

/// The LGB rasters of `bbox`: orthophoto and DTM tiles, the nDOM point clouds, and the bDOM where
/// no nDOM is published. With `skip_surface` (no disk space for it) neither surface model is
/// downloaded; the cached tiles recorded before stay.
fn fetch_rasters(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
    addresses: &Addresses,
) -> Result<(), AdapterError> {
    let (fetcher, mode, skip_surface) = (env.fetcher, env.mode, env.skip_surface);
    let cache = env.cache;
    let tiles = raster_tiles(env, bbox, TileProvider::Lgb.id(), "LGB", 1);
    // What the project recorded before this fetch.
    let old_tiles: Vec<TileRecord> = project.records().tiles.clone();
    let old_ndsm: Vec<NdsmRecord> = project.records().ndsm_tiles.clone();
    let mut records = Vec::new();
    for product in PRODUCTS {
        for fetched in fetch_tiles_at(
            addresses,
            fetcher,
            cache,
            product,
            &tiles,
            mode,
            env.progress,
        )? {
            let record = TileRecord {
                product,
                tile: fetched.tile.name(),
                url: fetched.url,
                currency: fetched.currency.currency(),
                published: fetched.currency.published,
            };
            let previous = old_tiles
                .iter()
                .find(|r| r.product == product && r.tile == record.tile);
            if let Some(previous) = previous.filter(|p| p.published != record.published) {
                (env.progress)(note!(
                    "warning-tile-changed",
                    product = format!("{product:?}"),
                    tile = &record.tile,
                    old = previous.published.to_string(),
                    new = record.published.to_string()
                ));
            }
            records.push(record);
        }
    }
    // Record the tiles first: they may have replaced older cached versions, and a failing
    // buildings download must not leave the project pointing at those.
    let previous_dsm: Vec<TileRecord> = old_tiles
        .iter()
        .filter(|r| r.product == Product::ImageDsm)
        .cloned()
        .collect();
    *project.records().tiles = records;
    project.save()?;
    if skip_surface {
        let cached = |tile: &str, path: &dyn Fn(TileId) -> PathBuf| {
            TileId::parse(tile).is_some_and(|t| tiles.contains(&t) && path(t).exists())
        };
        let r = project.records();
        r.ndsm_tiles
            .retain(|r| cached(&r.tile, &|t| cache.ndsm_path(t, r.published)));
        r.tiles.extend(
            previous_dsm
                .into_iter()
                .filter(|r| cached(&r.tile, &|t| cache.tile_path(r.product, t, r.published))),
        );
        let kept: Vec<&str> = r
            .ndsm_tiles
            .iter()
            .map(|r| r.tile.as_str())
            .chain(
                r.tiles
                    .iter()
                    .filter(|r| r.product == Product::ImageDsm)
                    .map(|r| r.tile.as_str()),
            )
            .collect();
        say_kept_surface_tiles(&kept, env.progress);
        drop_wms_orthophoto(project);
        project.save()?;
        return Ok(());
    }
    let ndsm_tiles: Vec<NdsmRecord> =
        fetch_ndsm_tiles_at(addresses, fetcher, cache, &tiles, mode, env.progress)?
            .into_iter()
            .map(|t| NdsmRecord {
                tile: t.tile.name(),
                url: t.url,
                published: t.published,
            })
            .collect();
    for record in &ndsm_tiles {
        let previous = old_ndsm.iter().find(|p| p.tile == record.tile);
        if let Some(previous) = previous.filter(|p| p.published != record.published)
            && let (Some(old), Some(new)) = (previous.published, record.published)
        {
            (env.progress)(note!(
                "warning-tile-changed",
                product = "nDSM",
                tile = &record.tile,
                old = old.to_string(),
                new = new.to_string()
            ));
        }
    }
    // Where no nDOM point cloud is published (Berlin), the nDSM comes from the bDOM surface
    // model minus the DTM. It is only a template: an unavailable bDOM costs a warning.
    let without_ndsm: Vec<TileId> = tiles
        .iter()
        .copied()
        .filter(|tile| !ndsm_tiles.iter().any(|r| r.tile == tile.name()))
        .collect();
    *project.records().ndsm_tiles = ndsm_tiles;
    if !without_ndsm.is_empty() {
        match fetch_tiles_each_at(
            addresses,
            fetcher,
            cache,
            Product::ImageDsm,
            &without_ndsm,
            mode,
            env.progress,
        ) {
            Ok(results) => {
                for (tile, result) in results {
                    let fetched = match result {
                        Ok(fetched) => fetched,
                        Err(e) => {
                            (env.progress)(note!(
                                "warning-bdom-tile-unavailable",
                                tile = tile.name(),
                                error = Arg::error(e)
                            ));
                            continue;
                        }
                    };
                    (env.progress)(note!("progress-ndsm-from-bdom", tile = tile.name()));
                    let record = TileRecord {
                        product: Product::ImageDsm,
                        tile: tile.name(),
                        url: fetched.url,
                        currency: fetched.currency.currency(),
                        published: fetched.currency.published,
                    };
                    if let Some(previous) = previous_dsm
                        .iter()
                        .find(|p| p.tile == record.tile && p.published != record.published)
                    {
                        (env.progress)(note!(
                            "warning-tile-changed",
                            product = format!("{:?}", Product::ImageDsm),
                            tile = &record.tile,
                            old = previous.published.to_string(),
                            new = record.published.to_string()
                        ));
                    }
                    project.records().tiles.push(record);
                }
            }
            Err(e) => (env.progress)(note!(
                "warning-bdom-unavailable",
                error = Arg::error(e),
                tiles = tile_names(&without_ndsm)
            )),
        }
    }
    drop_wms_orthophoto(project);
    project.save()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TILE: TileId = TileId {
        east_km: 421,
        north_km: 5821,
    };

    /// A project folder with its raster records.
    struct TestProject {
        dir: PathBuf,
        region: BoundingBox,
        tiles: Vec<TileRecord>,
        ndsm_tiles: Vec<NdsmRecord>,
        orthophoto: Option<freemapper_adapter_api::records::WmsRecord>,
        terrain: Option<freemapper_adapter_api::records::WcsRecord>,
        surface: Option<freemapper_adapter_api::records::WcsRecord>,
    }

    impl RasterProject for TestProject {
        fn records(&mut self) -> freemapper_adapter_api::project::RasterRecords<'_> {
            freemapper_adapter_api::project::RasterRecords {
                tiles: &mut self.tiles,
                ndsm_tiles: &mut self.ndsm_tiles,
                orthophoto: &mut self.orthophoto,
                terrain: &mut self.terrain,
                surface: &mut self.surface,
            }
        }

        fn save(&mut self) -> Result<(), AdapterError> {
            Ok(())
        }

        fn dir(&self) -> &std::path::Path {
            &self.dir
        }

        fn region_bbox(&self) -> BoundingBox {
            self.region
        }

        fn crs(&self) -> crate::crs::Crs {
            crate::crs::Crs::Utm33
        }
    }

    /// The currency list and the archive of `product` for `tile` at the folders of `a`.
    fn serve_tile(
        fetcher: &crate::fetch::test_support::MapFetcher,
        a: &Addresses,
        product: Product,
        tile: TileId,
    ) {
        use std::io::Write as _;
        let csv = format!(
            "\u{feff}sheetnr;sheetname;creationdate;revisiondate;publicationdate\n{};;2018-11-27;;2026-01-26;\n",
            tile.name()
        );
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file(product.raster_member(tile), options)
            .unwrap();
        zip.write_all(b"raster").unwrap();
        zip.start_file(product.world_file_member(tile), options)
            .unwrap();
        zip.write_all(b"1\n0\n0\n-1\n0.5\n0.5\n").unwrap();
        fetcher.insert(&a.currency_url_of(product), csv.into_bytes());
        fetcher.insert(
            &a.tile_url_of(product, tile),
            zip.finish().unwrap().into_inner(),
        );
    }

    fn laz_bytes() -> Vec<u8> {
        let mut writer = las::Writer::new(
            std::io::Cursor::new(Vec::new()),
            las::Builder::from((1, 2)).into_header().unwrap(),
        )
        .unwrap();
        for i in 0..5 {
            writer
                .write_point(las::Point {
                    x: f64::from(i),
                    y: 1.0,
                    z: 2.0,
                    ..Default::default()
                })
                .unwrap();
        }
        writer.into_inner().unwrap().into_inner()
    }

    /// What the backend asking the folders of `a` stored and said for the middle of `tile`.
    fn fetched(
        fetcher: &crate::fetch::test_support::MapFetcher,
        a: &Addresses,
        tile: TileId,
    ) -> (TestProject, Vec<String>) {
        let dir = tempfile::tempdir().unwrap();
        let bbox = BoundingBox {
            min_e: f64::from(tile.east_km) * 1000.0 + 400.0,
            min_n: f64::from(tile.north_km) * 1000.0 + 400.0,
            max_e: f64::from(tile.east_km) * 1000.0 + 600.0,
            max_n: f64::from(tile.north_km) * 1000.0 + 600.0,
        };
        let cache = crate::fetch::Cache::new(dir.path().join("cache"));
        std::fs::create_dir_all(dir.path().join("project")).unwrap();
        let mut project = TestProject {
            dir: dir.path().join("project"),
            region: bbox,
            tiles: Vec::new(),
            ndsm_tiles: Vec::new(),
            orthophoto: None,
            terrain: None,
            surface: None,
        };
        let mut registry = freemapper_adapter_api::registry::Registry::new();
        crate::states::register(&mut registry).unwrap();
        let mut notes = Vec::new();
        let mut env = AdapterEnv {
            fetcher,
            regions: &registry,
            cache: &cache,
            mode: crate::fetch::FetchMode::Online,
            today: "2026-11-13".parse().unwrap(),
            skip_surface: false,
            progress: &mut |n| notes.push(n.to_string()),
        };
        LgbBackend::with_addresses(a.clone())
            .fetch(&mut env, &mut project, &bbox)
            .unwrap();
        (project, notes)
    }

    /// Where a point cloud is published the bDOM is not asked, and every address is the one
    /// of the manifest: the fetcher holds nothing at the built-in addresses.
    #[test]
    fn a_tile_with_a_point_cloud_asks_no_bdom_and_every_folder_is_the_manifests() {
        let a = moved();
        let fetcher = crate::fetch::test_support::MapFetcher::default();
        serve_tile(&fetcher, &a, Product::Orthophoto, TILE);
        serve_tile(&fetcher, &a, Product::Dtm, TILE);
        let cloud = a.ndsm_tile_url(TILE);
        fetcher.insert(&cloud, laz_bytes());
        fetcher.set_modified(&cloud, date("2025-03-04"));
        let (project, notes) = fetched(&fetcher, &a, TILE);
        let urls: Vec<&str> = project.tiles.iter().map(|r| r.url.as_str()).collect();
        assert_eq!(
            urls,
            [
                "https://tiles.example.org/data/dop/rgb_jpg/dop_33421-5821.zip",
                "https://tiles.example.org/data/dgm/tif/dgm_33421-5821.zip"
            ]
        );
        assert_eq!(project.ndsm_tiles.len(), 1);
        assert_eq!(project.ndsm_tiles[0].url, cloud);
        assert!(
            !notes.iter().any(|n| n.to_lowercase().contains("bdom")),
            "{notes:?}"
        );
    }

    /// Where no point cloud is published (Berlin), the bDOM comes from the folders of the manifest.
    #[test]
    fn a_tile_without_a_point_cloud_takes_the_bdom_from_the_manifests_folders() {
        let a = moved();
        let berlin = TileId {
            east_km: 392,
            north_km: 5820,
        };
        let fetcher = crate::fetch::test_support::MapFetcher::default();
        for product in [Product::Orthophoto, Product::Dtm, Product::ImageDsm] {
            serve_tile(&fetcher, &a, product, berlin);
        }
        let (project, _) = fetched(&fetcher, &a, berlin);
        assert!(project.ndsm_tiles.is_empty());
        let bdom: Vec<&str> = project
            .tiles
            .iter()
            .filter(|r| r.product == Product::ImageDsm)
            .map(|r| r.url.as_str())
            .collect();
        assert_eq!(
            bdom,
            ["https://tiles.example.org/data/bdom/tif/bdom_33392-5820.zip"]
        );
    }

    fn date(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn moved() -> Addresses {
        Addresses {
            data: "https://tiles.example.org/data".to_string(),
            currency: "https://tiles.example.org/lists".to_string(),
        }
    }

    #[test]
    fn the_addresses_come_from_the_manifest_and_default_to_the_built_in_ones() {
        assert_eq!(
            Addresses::builtin(),
            Addresses {
                data: "https://data.geobasis-bb.de/geobasis/daten".to_string(),
                currency: "https://data.geobasis-bb.de/geobasis/information/aktualitaeten"
                    .to_string(),
            }
        );
        let mut config = RasterConfig::default();
        for missing in ["data", "currency"] {
            config.endpoints = ["data", "currency"]
                .iter()
                .filter(|n| **n != missing)
                .map(|n| (n.to_string(), format!("https://{n}.example.org")))
                .collect();
            let err = Addresses::of(&config).unwrap_err();
            assert!(err.contains(&format!("`{missing}`")), "{err}");
        }
    }

    #[test]
    fn every_url_is_built_below_the_folders_of_the_manifest() {
        let a = moved();
        let tile = |product| a.tile_url_of(product, TILE);
        assert_eq!(
            tile(Product::Orthophoto),
            "https://tiles.example.org/data/dop/rgb_jpg/dop_33421-5821.zip"
        );
        assert_eq!(
            tile(Product::Dtm),
            "https://tiles.example.org/data/dgm/tif/dgm_33421-5821.zip"
        );
        assert_eq!(
            tile(Product::ImageDsm),
            "https://tiles.example.org/data/bdom/tif/bdom_33421-5821.zip"
        );
        assert_eq!(
            a.currency_url_of(Product::Orthophoto),
            "https://tiles.example.org/lists/bb_dop_aktualitaet.csv"
        );
        assert_eq!(
            a.currency_url_of(Product::Dtm),
            "https://tiles.example.org/lists/bb_dgm_aktualitaet.csv"
        );
        assert_eq!(
            a.currency_url_of(Product::ImageDsm),
            "https://tiles.example.org/lists/bb_bdom_aktualitaet.csv"
        );
        assert_eq!(
            a.ndsm_tile_url(TILE),
            "https://tiles.example.org/data/ndom/NDOM_33421-5821_LAS12.laz"
        );
    }

    /// Saxony's products have nothing to do with the folders of LGB.
    #[test]
    fn the_saxon_products_ignore_the_folders_of_lgb() {
        let a = moved();
        for product in [Product::SaxonyDtm, Product::SaxonyDsm] {
            assert_eq!(a.tile_url_of(product, TILE), product.tile_url(TILE));
            assert_eq!(a.currency_url_of(product), product.currency_csv_url());
            assert!(!a.tile_url_of(product, TILE).contains("example.org"));
        }
    }

    #[test]
    fn bdom_url_matches_lgb_layout() {
        let tile = TileId {
            east_km: 392,
            north_km: 5820,
        };
        assert_eq!(
            Product::ImageDsm.tile_url(tile),
            "https://data.geobasis-bb.de/geobasis/daten/bdom/tif/bdom_33392-5820.zip"
        );
        assert_eq!(
            Product::ImageDsm.currency_csv_url(),
            "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_bdom_aktualitaet.csv"
        );
        assert_eq!(Product::ImageDsm.raster_member(tile), "bdom_33392-5820.tif");
        assert_eq!(
            Product::ImageDsm.world_file_member(tile),
            "bdom_33392-5820.tfw"
        );
        assert_eq!(Product::ImageDsm.cache_dir_name(), "bdom");
    }

    #[test]
    fn urls_match_lgb_layout() {
        assert_eq!(
            Product::Orthophoto.tile_url(TILE),
            "https://data.geobasis-bb.de/geobasis/daten/dop/rgb_jpg/dop_33421-5821.zip"
        );
        assert_eq!(
            Product::Dtm.tile_url(TILE),
            "https://data.geobasis-bb.de/geobasis/daten/dgm/tif/dgm_33421-5821.zip"
        );
        assert_eq!(
            Product::Orthophoto.raster_member(TILE),
            "dop_33421-5821.jpg"
        );
        assert_eq!(
            Product::Orthophoto.world_file_member(TILE),
            "dop_33421-5821.jgw"
        );
        assert_eq!(Product::Dtm.raster_member(TILE), "dgm_33421-5821.tif");
        assert_eq!(Product::Dtm.world_file_member(TILE), "dgm_33421-5821.tfw");
        assert_eq!(
            Product::Dtm.currency_csv_url(),
            "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_dgm_aktualitaet.csv"
        );
        assert_eq!(
            Product::Orthophoto.currency_csv_url(),
            "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_dop_aktualitaet.csv"
        );
        assert_eq!(Product::Orthophoto.cache_dir_name(), "dop20rgb");
        assert_eq!(Product::Dtm.cache_dir_name(), "dgm1");
    }

    #[test]
    fn currency_csv_parses_bom_and_empty_fields() {
        let csv = "\u{feff}sheetnr;sheetname;creationdate;revisiondate;publicationdate;processstepdate\n\
                   33421-5821;;2018-11-27;2023-05-04;2026-01-26;\n\
                   33421-5822;;2026-05-01;;2026-10-09;2026-06-15\n";
        let table = parse_currency_csv(csv).unwrap();
        assert_eq!(table.len(), 2);
        let dtm = table[&TILE];
        assert_eq!(dtm.currency(), date("2023-05-04"));
        assert_eq!(dtm.published, date("2026-01-26"));
        let north = TileId {
            east_km: 421,
            north_km: 5822,
        };
        assert_eq!(table[&north].revised, None);
        assert_eq!(table[&north].currency(), date("2026-05-01"));
    }

    #[test]
    fn currency_csv_reports_bad_line() {
        let csv = "sheetnr;sheetname;creationdate;revisiondate;publicationdate;processstepdate\n\
                   33421-5821;;2018-11-27;2023-05-04;2026-01-26;\n\
                   33421-5822;;2026-13-01;;2026-10-09;\n";
        let err = parse_currency_csv(csv).unwrap_err();
        assert_eq!(err.line, 3);
    }

    #[test]
    fn currency_csv_rejects_other_documents() {
        let err = parse_currency_csv("<html>captive portal</html>").unwrap_err();
        assert_eq!(err.line, 1);
    }

    #[test]
    fn ndsm_url_matches_lgb_layout() {
        assert_eq!(
            ndsm_tile_url(TILE),
            "https://data.geobasis-bb.de/geobasis/daten/ndom/NDOM_33421-5821_LAS12.laz"
        );
    }

    #[test]
    fn attribution_text() {
        assert_eq!(
            attribution(2026),
            "© GeoBasis-DE/LGB 2026, dl-de/by-2-0, Daten geändert"
        );
    }
}
