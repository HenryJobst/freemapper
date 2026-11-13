//! The library's errors in the language of the run (ADR 0092, stage 2).
//!
//! The error types of the library keep their English `Display` texts for logs and tests. This
//! file gives each of them a text from the catalog; in English it equals `to_string()`. Parts
//! that other programs word (the operating system, parsers, servers) and details the library
//! formats as free text stay as they are.

use std::error::Error;
use std::path::Path;

use freemapper::area::AreaError;
use freemapper::derive::DeriveError;
use freemapper::fetch::{CurrencyCsvError, Product};
use freemapper::fetch::{FetchError, Remedy};
use freemapper::georef::GeorefError;
use freemapper::ocd::file::OcdBuildError;
use freemapper::omap::OmapError;
use freemapper::permits::PermitsError;
use freemapper::places::PlaceError;
use freemapper::project::{MAX_AREA_SIDE_M, ProjectError};
use freemapper::raster::RasterError;
use freemapper::sources::alkis::AlkisError;
use freemapper::sources::osm::OsmError;
use freemapper::space::{Part, RESERVE_BYTES, Shortfall, is_storage_full, size, size_down};
use freemapper::tiles::TileId;

use crate::lock::LockError;
use crate::pipeline::Cancelled;
use crate::text::{FluentValue, Lang, TEXTS, n, v};

/// The text of a value, for example an error, in a language.
pub trait Localize {
    /// The text in `lang`. In English it equals `to_string()` for error types.
    fn localize(&self, lang: &Lang) -> String;
}

/// `error` and its causes in `lang`, e.g. "reading area.json: No such file". An error type this
/// file does not know keeps its own text.
pub fn message(error: &(dyn Error + 'static), lang: &Lang) -> String {
    let mut message = line(error, lang);
    let mut source = error.source();
    while let Some(cause) = source {
        let line = line(cause, lang);
        if !message.contains(&line) {
            message = format!("{message}: {line}");
        }
        source = cause.source();
    }
    message
}

/// One error without its causes.
fn line(error: &(dyn Error + 'static), lang: &Lang) -> String {
    macro_rules! known {
        ($($type:ty),* $(,)?) => {
            $(if let Some(e) = error.downcast_ref::<$type>() {
                return e.localize(lang);
            })*
        };
    }
    known!(
        ProjectError,
        FetchError,
        PlaceError,
        AreaError,
        DeriveError,
        RasterError,
        GeorefError,
        OmapError,
        OcdBuildError,
        CurrencyCsvError,
        AlkisError,
        OsmError,
        Shortfall,
        Cancelled,
        LockError,
        PermitsError,
    );
    error.to_string()
}

fn text(lang: &Lang, id: &str, args: &[(&str, FluentValue)]) -> String {
    TEXTS.text(lang, id, args)
}

fn path(path: &Path) -> (&'static str, FluentValue<'static>) {
    ("path", v(path.display()))
}

fn tile_names(tiles: &[TileId]) -> String {
    tiles
        .iter()
        .map(TileId::name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// An error reading or writing `file`: plain words when the disk is full, else the path and the
/// system's message.
fn io(lang: &Lang, file: &Path, e: &std::io::Error) -> String {
    if is_storage_full(e) {
        text(lang, "error-io-disk-full", &[path(file)])
    } else {
        text(lang, "error-io", &[path(file), ("message", v(e))])
    }
}

/// The hint `MapKind::check_scale` adds for a sprint map at 1:3000.
const ENLARGE_HINT: &str = ". Draw at 1:4000, freemapper-print enlarges to 1:3000 when printing";

impl Localize for ProjectError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            ProjectError::Io(file, e) => io(lang, file, e),
            ProjectError::AlreadyExists(dir) => {
                text(lang, "error-project-already-exists", &[path(dir)])
            }
            ProjectError::MapExists(map) => text(lang, "error-project-map-exists", &[path(map)]),
            ProjectError::InvalidScale(scale) => {
                text(lang, "error-project-invalid-scale", &[("scale", v(scale))])
            }
            ProjectError::ScaleNotAllowed {
                kind,
                scale,
                allowed,
                hint,
            } => {
                let kind = match *kind {
                    "schoolyard" => text(lang, "word-kind-schoolyard", &[]),
                    "sprint" => text(lang, "word-kind-sprint", &[]),
                    other => other.to_owned(),
                };
                let args = [
                    ("scale", v(scale)),
                    ("kind", v(kind)),
                    ("allowed", v(allowed)),
                ];
                if *hint == ENLARGE_HINT {
                    text(lang, "error-project-scale-not-allowed-enlarge", &args)
                } else {
                    // A hint without a catalog text stays as it is.
                    text(lang, "error-project-scale-not-allowed", &args) + hint
                }
            }
            ProjectError::Toml(message) => {
                text(lang, "error-project-toml", &[("message", v(message))])
            }
            ProjectError::UnknownAdapter(adapter) => text(
                lang,
                "error-project-unknown-adapter",
                &[("adapter", v(adapter))],
            ),
            ProjectError::Area(e) => e.localize(lang),
            ProjectError::Georef(e) => e.localize(lang),
            ProjectError::Omap(e) => e.localize(lang),
            ProjectError::Fetch(e) => e.localize(lang),
            ProjectError::Derive(e) => e.localize(lang),
            ProjectError::NotFetched => text(lang, "error-project-not-fetched", &[]),
            ProjectError::Cancelled => text(lang, "error-cancelled", &[]),
            ProjectError::Layer(detail) => detail.text(lang, &message),
            ProjectError::Ocd(e) => text(
                lang,
                "error-project-ocd",
                &[("message", v(e.localize(lang)))],
            ),
            ProjectError::TemplateOutdated(file) => {
                text(lang, "error-project-template-outdated", &[path(file)])
            }
            ProjectError::TilesOutdated(tiles) => text(
                lang,
                "error-project-tiles-outdated",
                &[("tiles", v(tile_names(tiles)))],
            ),
            ProjectError::BuildingsNotFetched => {
                text(lang, "error-project-buildings-not-fetched", &[])
            }
            ProjectError::OrthophotoNotFetched => {
                text(lang, "error-project-orthophoto-not-fetched", &[])
            }
            ProjectError::TerrainNotFetched => text(lang, "error-project-terrain-not-fetched", &[]),
            ProjectError::CandidatesOutdated => {
                text(lang, "error-project-candidates-outdated", &[])
            }
            ProjectError::StoredCandidates {
                path: file,
                message,
            }
            | ProjectError::StoredBuildings {
                path: file,
                message,
            } => text(
                lang,
                "error-project-stored",
                &[path(file), ("message", v(message))],
            ),
            ProjectError::Osm(e) => e.localize(lang),
            ProjectError::AreaTooLarge { width_m, height_m } => text(
                lang,
                "error-project-area-too-large",
                &[
                    ("width", v(format!("{width_m:.0}"))),
                    ("height", v(format!("{height_m:.0}"))),
                    ("limit", v(format!("{MAX_AREA_SIDE_M:.0}"))),
                ],
            ),
            ProjectError::OutsideCoverage => text(
                lang,
                "error-project-outside-coverage",
                &[("states", v(crate::text::states(lang)))],
            ),
            ProjectError::NotEnoughSpace(shortfall) => shortfall.localize(lang),
        }
    }
}

impl Localize for Part {
    fn localize(&self, lang: &Lang) -> String {
        let id = match self {
            Part::MapData => "word-part-map-data",
            Part::Terrain => "word-part-terrain",
            Part::Orthophoto => "word-part-orthophoto",
            Part::Surface => "word-part-surface",
            Part::OsmUpdate => "word-part-osm-update",
            Part::K5 => "word-part-k-five",
            Part::PointCloud => "word-part-point-cloud",
            Part::BerlinLayers => "word-part-berlin-layers",
            Part::StateLayers => "word-part-state-layers",
        };
        text(lang, id, &[])
    }
}

impl Localize for Shortfall {
    fn localize(&self, lang: &Lang) -> String {
        let id = if self.part.required() {
            "error-space-required"
        } else {
            "error-space-skipped"
        };
        text(
            lang,
            id,
            &[
                ("part", v(self.part.localize(lang))),
                ("needed", v(size(self.needed))),
                path(&self.path),
                ("left", v(size_down(self.left))),
                ("reserve", v(size(RESERVE_BYTES))),
            ],
        )
    }
}

impl Localize for Product {
    fn localize(&self, lang: &Lang) -> String {
        let id = match self {
            Product::Orthophoto => "word-product-orthophoto",
            Product::Dtm => "word-product-dtm",
            Product::ImageDsm => "word-product-image-dsm",
            Product::SaxonyDtm => "word-product-saxony-dtm",
            Product::SaxonyDsm => "word-product-saxony-dsm",
        };
        text(lang, id, &[])
    }
}

/// The reasons of the rules for where requests go, with the id of each in the catalog.
const POLICY_REASONS: [(&str, &str); 12] = [
    ("not a host name", "error-reason-not-a-host-name"),
    (
        "a name outside ASCII must be written in its xn-- form",
        "error-reason-not-ascii",
    ),
    ("not a DNS name", "error-reason-not-a-dns-name"),
    (
        "a name of this computer or without a domain",
        "error-reason-no-domain",
    ),
    ("a name of the local network", "error-reason-local-name"),
    ("an IP address, not a DNS name", "error-reason-ip-address"),
    ("unexpected character", "error-reason-unexpected-character"),
    ("not https", "error-reason-not-https"),
    ("user info", "error-reason-user-info"),
    ("a port other than 443", "error-reason-port"),
    (
        "the host is not one of the declared hosts",
        "error-reason-host-not-declared",
    ),
    (
        "an address of this computer or of the local network, not of a server on the internet",
        "error-reason-local-address",
    ),
];

/// Why freemapper did not send a request, in the language of the run. The library gives the
/// reason in English: one of the reasons of the rules, alone or after what it is about
/// (`127.0.0.1: an address …`), then the address that redirected there if one did
/// (`… (https://… redirects there)`), or one of the two about redirects that lead nowhere.
pub(crate) fn not_allowed_reason(lang: &Lang, reason: &str) -> String {
    let worded = |english: &str| {
        POLICY_REASONS
            .iter()
            .find(|(known, _)| *known == english)
            .map(|(_, id)| text(lang, id, &[]))
    };
    if let Some((why, from)) = reason
        .strip_suffix(" redirects there)")
        .and_then(|rest| rest.rsplit_once(" ("))
    {
        let from = text(
            lang,
            "error-fetch-host-redirected-from",
            &[("url", v(from))],
        );
        return format!("{}{from}", not_allowed_reason(lang, why));
    }
    if let Some(count) = reason
        .strip_prefix("more than ")
        .and_then(|rest| rest.strip_suffix(" redirects"))
    {
        return text(
            lang,
            "error-reason-too-many-redirects",
            &[("count", v(count))],
        );
    }
    if let Some(status) = reason
        .strip_prefix("a redirect (HTTP ")
        .and_then(|rest| rest.strip_suffix(") to nowhere"))
    {
        return text(
            lang,
            "error-reason-redirect-nowhere",
            &[("status", v(status))],
        );
    }
    if let Some(text) = worded(reason) {
        return text;
    }
    match reason.rsplit_once(": ") {
        Some((subject, why)) => match worded(why) {
            Some(text) => format!("{subject}: {text}"),
            None => reason.to_string(),
        },
        None => reason.to_string(),
    }
}

impl Localize for FetchError {
    fn localize(&self, lang: &Lang) -> String {
        let url = |url: &String| ("url", v(url));
        match self {
            FetchError::Http { url: from, message } => text(
                lang,
                "error-fetch-http",
                &[url(from), ("message", v(message))],
            ),
            FetchError::Io(file, e) => io(lang, file, e),
            FetchError::CorruptArchive { url: from, message } => text(
                lang,
                "error-fetch-corrupt-archive",
                &[url(from), ("message", v(message))],
            ),
            FetchError::CorruptExtract { url: from, message } => text(
                lang,
                "error-fetch-corrupt-extract",
                &[url(from), ("message", v(message))],
            ),
            FetchError::TilesNotCovered(tiles) => text(
                lang,
                "error-fetch-tiles-not-covered",
                &[("count", n(tiles.len())), ("tiles", v(tile_names(tiles)))],
            ),
            FetchError::MissingOffline(product, tiles) => text(
                lang,
                "error-fetch-missing-offline",
                &[
                    ("product", v(product.localize(lang))),
                    ("count", n(tiles.len())),
                    ("tiles", v(tile_names(tiles))),
                ],
            ),
            FetchError::CurrencyCsv(e) => e.localize(lang),
            FetchError::Alkis(e) => e.localize(lang),
            FetchError::OsmExtractMissing => text(lang, "error-fetch-osm-extract-missing", &[]),
            FetchError::NotFound(name) => text(lang, "error-fetch-not-found", &[("name", v(name))]),
            FetchError::Refused {
                url: from,
                status,
                detail,
            } => match detail {
                Some(detail) => text(
                    lang,
                    "error-fetch-refused-detail",
                    &[url(from), ("status", v(status)), ("detail", v(detail))],
                ),
                None => text(
                    lang,
                    "error-fetch-refused",
                    &[url(from), ("status", v(status))],
                ),
            },
            FetchError::TooLarge {
                url: from,
                limit_mb,
            } => text(
                lang,
                "error-fetch-too-large",
                &[url(from), ("limit", v(limit_mb))],
            ),
            FetchError::NotStored { label, path: file } => text(
                lang,
                "error-fetch-not-stored",
                &[("label", v(label)), path(file)],
            ),
            FetchError::RangeIgnored {
                url: from,
                range,
                status,
            } => text(
                lang,
                "error-fetch-range-ignored",
                &[url(from), ("range", v(range)), ("status", v(status))],
            ),
            FetchError::Cancelled => text(lang, "error-cancelled", &[]),
            FetchError::NotAllowed { url: from, reason } => text(
                lang,
                "error-fetch-not-allowed",
                &[url(from), ("reason", v(not_allowed_reason(lang, reason)))],
            ),
            FetchError::HostNotNamed(refused) => {
                // The region in the language of the run; the error carries the English name.
                let name = freemapper::adapters::registry()
                    .get(&refused.adapter)
                    .filter(|_| lang.code() == "de")
                    .map_or(&refused.name, |adapter| &adapter.info().names.de);
                let from = refused.from.as_ref().map_or_else(String::new, |from| {
                    text(lang, "error-fetch-host-redirected-from", &[url(from)])
                });
                let id = match refused.remedy {
                    Remedy::AllowHost => "error-fetch-host-not-named-allow",
                    Remedy::Manifest => "error-fetch-host-not-named-manifest",
                };
                text(
                    lang,
                    id,
                    &[
                        ("target", v(&refused.target)),
                        ("host", v(&refused.host)),
                        ("adapter", v(&refused.adapter)),
                        ("name", v(name)),
                        ("from", v(from)),
                    ],
                )
            }
        }
    }
}

impl Localize for CurrencyCsvError {
    fn localize(&self, lang: &Lang) -> String {
        text(
            lang,
            "error-fetch-currency-csv",
            &[("line", v(self.line)), ("message", v(&self.message))],
        )
    }
}

impl Localize for AlkisError {
    fn localize(&self, lang: &Lang) -> String {
        text(lang, "error-alkis", &[("message", v(&self.message))])
    }
}

impl Localize for OsmError {
    fn localize(&self, lang: &Lang) -> String {
        text(lang, "error-osm", &[("message", v(&self.message))])
    }
}

impl Localize for PlaceError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            PlaceError::Fetch(e) => e.localize(lang),
            PlaceError::Area(e) => e.localize(lang),
        }
    }
}

impl Localize for AreaError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            AreaError::GeoJson(message) => {
                text(lang, "error-area-geojson", &[("message", v(message))])
            }
            AreaError::NoPolygon => text(lang, "error-area-no-polygon", &[]),
            AreaError::MultiPolygon(parts) => {
                text(lang, "error-area-multi-polygon", &[("parts", v(parts))])
            }
            AreaError::OutOfRange { lat, lon } => text(
                lang,
                "error-area-out-of-range",
                &[("lat", v(lat)), ("lon", v(lon))],
            ),
            AreaError::Place(detail) => detail.text(lang, &message),
            AreaError::NotAnArea(message) => {
                text(lang, "error-area-not-an-area", &[("message", v(message))])
            }
        }
    }
}

impl Localize for DeriveError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            DeriveError::Io(file, e) => io(lang, file, e),
            DeriveError::Zip(message) => text(lang, "error-derive-zip", &[("message", v(message))]),
            DeriveError::Image(message) => {
                text(lang, "error-derive-image", &[("message", v(message))])
            }
            DeriveError::Raster(e) => e.localize(lang),
            DeriveError::NoTiles => text(lang, "error-derive-no-tiles", &[]),
            DeriveError::AreaTooLarge { width, height } => text(
                lang,
                "error-derive-area-too-large",
                &[
                    ("width", v(width)),
                    ("height", v(height)),
                    ("limit", v(freemapper::derive::MAX_RASTER_SIDE_PX)),
                ],
            ),
        }
    }
}

impl Localize for RasterError {
    fn localize(&self, lang: &Lang) -> String {
        let (id, message) = match self {
            RasterError::WorldFile(message) => ("error-raster-world-file", message),
            RasterError::Misaligned(message) => ("error-raster-misaligned", message),
            RasterError::PixelSizeMismatch(message) => ("error-raster-pixel-size", message),
        };
        text(lang, id, &[("message", v(message))])
    }
}

impl Localize for GeorefError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            GeorefError::DateOutsideModel(date) => {
                text(lang, "error-georef-date", &[("date", v(date))])
            }
            GeorefError::OutsideMapRange { easting, northing } => text(
                lang,
                "error-georef-map-range",
                &[
                    ("easting", v(format!("{easting:.1}"))),
                    ("northing", v(format!("{northing:.1}"))),
                ],
            ),
        }
    }
}

impl Localize for OmapError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            OmapError::Xml(message) => text(lang, "error-omap-xml", &[("message", v(message))]),
            OmapError::MissingElement(name) => {
                text(lang, "error-omap-missing-element", &[("name", v(name))])
            }
            // The system's own words.
            OmapError::Io(e) => e.to_string(),
        }
    }
}

impl Localize for OcdBuildError {
    fn localize(&self, lang: &Lang) -> String {
        let (id, symbol) = match self {
            OcdBuildError::DuplicateSymbol(symbol) => ("error-ocd-duplicate-symbol", symbol),
            OcdBuildError::UnknownSymbol(symbol) => ("error-ocd-unknown-symbol", symbol),
            OcdBuildError::WrongType { symbol } => ("error-ocd-wrong-type", symbol),
        };
        text(lang, id, &[("symbol", v(symbol))])
    }
}

impl Localize for PermitsError {
    fn localize(&self, lang: &Lang) -> String {
        match self {
            PermitsError::Project(e) => e.localize(lang),
            PermitsError::Io(file, e) => io(lang, file, e),
            PermitsError::Stored {
                path: file,
                message,
            } => text(
                lang,
                "error-permits-stored",
                &[path(file), ("message", v(message))],
            ),
            PermitsError::NothingRead { missing } => {
                let missing: Vec<String> = missing
                    .iter()
                    .map(|m| match &m.reason {
                        Some(_) => m.to_string(),
                        None => text(
                            lang,
                            "error-permits-missing-not-read",
                            &[("source", v(&m.source))],
                        ),
                    })
                    .collect();
                text(
                    lang,
                    "error-permits-nothing-read",
                    &[("missing", v(missing.join("; ")))],
                )
            }
        }
    }
}

impl Localize for Cancelled {
    fn localize(&self, lang: &Lang) -> String {
        let names: Vec<&str> = self.kept.iter().map(|f| f.relative_path.as_str()).collect();
        if names.is_empty() {
            text(lang, "error-cancelled", &[])
        } else {
            text(
                lang,
                "cli-cancelled-kept",
                &[("files", v(names.join(", ")))],
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn de() -> Lang {
        TEXTS.parse_lang_arg("de").unwrap()
    }

    fn tile(east_km: u32) -> TileId {
        TileId {
            east_km,
            north_km: 5821,
        }
    }

    fn disk_full() -> std::io::Error {
        std::io::Error::from(std::io::ErrorKind::StorageFull)
    }

    fn missing() -> std::io::Error {
        std::io::Error::from(std::io::ErrorKind::NotFound)
    }

    fn shortfall(part: Part) -> Shortfall {
        Shortfall {
            part,
            path: PathBuf::from("/data/cache"),
            needed: 3 << 30,
            left: 700 << 20,
        }
    }

    const PARTS: [Part; 9] = [
        Part::MapData,
        Part::Terrain,
        Part::Orthophoto,
        Part::Surface,
        Part::OsmUpdate,
        Part::K5,
        Part::BerlinLayers,
        Part::StateLayers,
        Part::PointCloud,
    ];

    fn fetch_errors() -> Vec<FetchError> {
        let url = || "https://example.org/a.zip".to_owned();
        vec![
            FetchError::Http {
                url: url(),
                message: "timeout".into(),
            },
            FetchError::Io(PathBuf::from("a/b.zip"), missing()),
            FetchError::Io(PathBuf::from("a/b.zip"), disk_full()),
            FetchError::CorruptArchive {
                url: url(),
                message: "no end record".into(),
            },
            FetchError::CorruptExtract {
                url: url(),
                message: "short".into(),
            },
            FetchError::TilesNotCovered(vec![tile(421)]),
            FetchError::TilesNotCovered(vec![tile(421), tile(422)]),
            FetchError::MissingOffline(Product::Dtm, vec![tile(421)]),
            FetchError::MissingOffline(Product::Orthophoto, vec![tile(421), tile(422)]),
            FetchError::MissingOffline(Product::ImageDsm, vec![tile(421)]),
            FetchError::MissingOffline(Product::SaxonyDtm, vec![tile(421)]),
            FetchError::MissingOffline(Product::SaxonyDsm, vec![tile(421)]),
            FetchError::CurrencyCsv(CurrencyCsvError {
                line: 3,
                message: "no date".into(),
            }),
            FetchError::Alkis(AlkisError {
                message: "no ring".into(),
            }),
            FetchError::OsmExtractMissing,
            FetchError::NotFound("dgm_33421-5821.zip".into()),
            FetchError::Refused {
                url: url(),
                status: 403,
                detail: None,
            },
            FetchError::Refused {
                url: url(),
                status: 400,
                detail: Some("bbox too large".into()),
            },
            FetchError::TooLarge {
                url: url(),
                limit_mb: 512,
            },
            FetchError::NotStored {
                label: "terrain model".into(),
                path: PathBuf::from("data/terrain.tif"),
            },
            FetchError::RangeIgnored {
                url: url(),
                range: "0-99".into(),
                status: 200,
            },
            FetchError::Cancelled,
            FetchError::NotAllowed {
                url: url(),
                reason: "more than 3 redirects".into(),
            },
            host_not_named(Remedy::AllowHost, None),
            host_not_named(Remedy::AllowHost, Some("https://www.gds.hessen.de/a")),
            host_not_named(Remedy::Manifest, None),
            host_not_named(Remedy::Manifest, Some("https://www.gds.hessen.de/a")),
        ]
    }

    /// A request for Hesse that was not sent because Hesse's adapter does not name the host.
    fn host_not_named(remedy: Remedy, from: Option<&str>) -> FetchError {
        FetchError::HostNotNamed(Box::new(freemapper::fetch::HostNotNamed {
            target: "https://cdn.example.net/x".to_string(),
            host: "cdn.example.net".to_string(),
            from: from.map(str::to_string),
            adapter: "hessen".to_string(),
            name: "Hesse".to_string(),
            remedy,
        }))
    }

    fn area_errors() -> Vec<AreaError> {
        vec![
            AreaError::GeoJson("expected value".into()),
            AreaError::NoPolygon,
            AreaError::MultiPolygon(3),
            AreaError::OutOfRange {
                lat: 91.5,
                lon: 13.0,
            },
            AreaError::Place(
                freemapper::note::Note::new("error-place-not-an-osm-id").with("id", "\"x\""),
            ),
            AreaError::NotAnArea("the outline crosses itself".into()),
        ]
    }

    fn derive_errors() -> Vec<DeriveError> {
        vec![
            DeriveError::Io(PathBuf::from("templates/a.jpg"), missing()),
            DeriveError::Zip("bad header".into()),
            DeriveError::Image("not a JPEG".into()),
            DeriveError::Raster(RasterError::WorldFile("not a number: \"x\"".into())),
            DeriveError::Raster(RasterError::Misaligned("a vs b".into())),
            DeriveError::Raster(RasterError::PixelSizeMismatch("0.2 vs 1".into())),
            DeriveError::NoTiles,
            DeriveError::AreaTooLarge {
                width: 30_000,
                height: 100,
            },
        ]
    }

    fn georef_errors() -> Vec<GeorefError> {
        vec![
            GeorefError::DateOutsideModel(chrono::NaiveDate::from_ymd_opt(2040, 1, 2).unwrap()),
            GeorefError::OutsideMapRange {
                easting: 400_000.26,
                northing: 5_800_000.0,
            },
        ]
    }

    fn omap_errors() -> Vec<OmapError> {
        vec![
            OmapError::Xml("unexpected end".into()),
            OmapError::MissingElement("georeferencing"),
            OmapError::Io(missing()),
        ]
    }

    fn ocd_errors() -> Vec<OcdBuildError> {
        vec![
            OcdBuildError::DuplicateSymbol(521_000),
            OcdBuildError::UnknownSymbol(7),
            OcdBuildError::WrongType { symbol: 7 },
        ]
    }

    fn project_errors() -> Vec<ProjectError> {
        use freemapper::project::MapKind;
        let mut errors = vec![
            ProjectError::Io(PathBuf::from("p/freemapper.toml"), missing()),
            ProjectError::Io(PathBuf::from("p/freemapper.toml"), disk_full()),
            ProjectError::AlreadyExists(PathBuf::from("p")),
            ProjectError::MapExists(PathBuf::from("p/p.omap")),
            MapKind::Sprint.check_scale(0).unwrap_err(),
            MapKind::Sprint.check_scale(3000).unwrap_err(),
            MapKind::Sprint.check_scale(10_000).unwrap_err(),
            MapKind::Schoolyard.check_scale(4000).unwrap_err(),
            ProjectError::ScaleNotAllowed {
                kind: "gym",
                scale: 7,
                allowed: "1:200".into(),
                hint: ". A new hint",
            },
            ProjectError::Toml("missing field `crs`".into()),
            ProjectError::UnknownAdapter("czfoo".into()),
            ProjectError::NotFetched,
            ProjectError::Cancelled,
            ProjectError::Layer(
                freemapper::note::Note::new("error-layer-not-a-world-file")
                    .with("path", "templates/a.jgw"),
            ),
            ProjectError::TemplateOutdated(PathBuf::from("templates/a.jpg")),
            ProjectError::TilesOutdated(vec![tile(421), tile(422)]),
            ProjectError::BuildingsNotFetched,
            ProjectError::OrthophotoNotFetched,
            ProjectError::TerrainNotFetched,
            ProjectError::CandidatesOutdated,
            ProjectError::StoredCandidates {
                path: PathBuf::from("data/candidates.json"),
                message: "not JSON".into(),
            },
            ProjectError::StoredBuildings {
                path: PathBuf::from("data/buildings.json"),
                message: "not JSON".into(),
            },
            ProjectError::Osm(OsmError {
                message: "bad block".into(),
            }),
            ProjectError::AreaTooLarge {
                width_m: 4100.4,
                height_m: 120.6,
            },
            ProjectError::OutsideCoverage,
        ];
        errors.extend(area_errors().into_iter().map(ProjectError::Area));
        errors.extend(georef_errors().into_iter().map(ProjectError::Georef));
        errors.extend(omap_errors().into_iter().map(ProjectError::Omap));
        errors.extend(fetch_errors().into_iter().map(ProjectError::Fetch));
        errors.extend(derive_errors().into_iter().map(ProjectError::Derive));
        errors.extend(ocd_errors().into_iter().map(ProjectError::Ocd));
        errors.extend(
            PARTS
                .into_iter()
                .map(|part| ProjectError::NotEnoughSpace(shortfall(part))),
        );
        errors
    }

    #[test]
    fn english_equals_the_library_text() {
        let english = Lang::english();
        for error in project_errors() {
            assert_eq!(error.localize(&english), error.to_string(), "{error:?}");
        }
        for error in fetch_errors().into_iter().map(PlaceError::Fetch) {
            assert_eq!(error.localize(&english), error.to_string(), "{error:?}");
        }
        for error in area_errors().into_iter().map(PlaceError::Area) {
            assert_eq!(error.localize(&english), error.to_string(), "{error:?}");
        }
        for part in PARTS {
            assert_eq!(part.localize(&english), part.label());
        }
    }

    #[test]
    fn german_has_a_text_of_its_own_for_every_error() {
        // Free text of the library and the system's words pass through unchanged.
        let passes_through = |error: &ProjectError| match error {
            ProjectError::Omap(OmapError::Io(_)) => true,
            ProjectError::Io(_, e)
            | ProjectError::Fetch(FetchError::Io(_, e))
            | ProjectError::Derive(DeriveError::Io(_, e)) => !is_storage_full(e),
            _ => false,
        };
        for error in project_errors() {
            let german = error.localize(&de());
            assert!(!german.contains("error-"), "{german}");
            assert!(!german.contains("word-"), "{german}");
            if !passes_through(&error) {
                assert_ne!(german, error.to_string(), "{error:?}");
            }
        }
    }

    #[test]
    fn german_names_tiles_kinds_and_parts() {
        let de = de();
        assert_eq!(
            FetchError::TilesNotCovered(vec![tile(421)]).localize(&de),
            "Die Kachel 33421-5821 gehört nicht zu den veröffentlichten Kacheln; verkleinern Sie das Kartengebiet oder buffer_m in freemapper.toml"
        );
        assert!(
            FetchError::TilesNotCovered(vec![tile(421), tile(422)])
                .localize(&de)
                .starts_with("Die Kacheln 33421-5821, 33422-5821 gehören")
        );
        let scale = freemapper::project::MapKind::Sprint
            .check_scale(3000)
            .unwrap_err()
            .localize(&de);
        assert_eq!(
            scale,
            "Der Maßstab 1:3000 ist für Sprintkarten nicht erlaubt; nehmen Sie 1:4000, 1:5000. Zeichnen Sie in 1:4000, freemapper-print vergrößert beim Drucken auf 1:3000"
        );
        let space = ProjectError::NotEnoughSpace(shortfall(Part::Terrain)).localize(&de);
        assert!(
            space.starts_with("Der Speicherplatz reicht nicht für das Geländemodell. Gebraucht werden 3.0 GB, aber /data/cache hat über die Reserve von 1.0 GB hinaus nur 700 MB frei."),
            "{space}"
        );
    }

    #[test]
    fn causes_follow_the_error_once() {
        let english = Lang::english();
        let wrapped = anyhow::Error::new(ProjectError::NotFetched).context("while testing");
        assert_eq!(
            message(wrapped.as_ref(), &english),
            "while testing: no tiles fetched yet; run `freemapper fetch` first"
        );
        assert_eq!(
            message(wrapped.as_ref(), &de()),
            "while testing: Es sind noch keine Kacheln geladen; rufen Sie zuerst `freemapper fetch` auf"
        );
        // A cause the text already holds is not repeated.
        let ocd = ProjectError::Ocd(OcdBuildError::UnknownSymbol(7));
        assert_eq!(
            message(&ocd, &de()),
            "OCAD-Export: Ein Objekt verweist auf das Symbol 7, das in der Datei fehlt"
        );
    }

    #[test]
    fn a_cancelled_run_names_what_it_kept() {
        let none = Cancelled { kept: Vec::new() };
        assert_eq!(none.localize(&de()), "abgebrochen");
        assert_eq!(none.localize(&Lang::english()), none.to_string());
    }

    /// A request freemapper does not send says why in the language of the run: every reason of
    /// the rules has its text in both catalogs, alone, after what it is about, and inside the
    /// three sentences about redirects.
    #[test]
    fn the_reason_of_a_request_not_sent_is_worded_in_the_language() {
        let (english, german) = (Lang::english(), TEXTS.parse_lang_arg("de").unwrap());
        assert_eq!(
            POLICY_REASONS.map(|(reason, _)| reason),
            freemapper_adapter_api_reasons()
        );
        for (reason, _) in POLICY_REASONS {
            assert_eq!(not_allowed_reason(&english, reason), reason);
            assert_ne!(not_allowed_reason(&german, reason), reason, "{reason}");
        }
        let said = |reason: &str| not_allowed_reason(&german, reason);
        assert_eq!(
            said(
                "127.0.0.1: an address of this computer or of the local network, not of a server on the internet"
            ),
            "127.0.0.1: eine Adresse dieses Rechners oder des lokalen Netzes, nicht die eines Servers im Internet"
        );
        assert_eq!(
            said(
                "the host is not one of the declared hosts (https://geo.example.org/a redirects there)"
            ),
            "der Host gehört nicht zu den erklärten Hosts (https://geo.example.org/a leitet dorthin weiter)"
        );
        assert_eq!(said("more than 3 redirects"), "mehr als 3 Weiterleitungen");
        assert_eq!(
            said("a redirect (HTTP 302) to nowhere"),
            "eine Weiterleitung (HTTP 302) ohne Ziel"
        );
        // A reason this table does not know stays as it is.
        assert_eq!(said("something new"), "something new");
        let error = FetchError::NotAllowed {
            url: "https://geo.example.org/a".to_string(),
            reason: "more than 3 redirects".to_string(),
        };
        assert_eq!(
            error.localize(&german),
            "https://geo.example.org/a: freemapper sendet keine Anfragen an diese Adresse: mehr als 3 Weiterleitungen"
        );
        assert_eq!(error.localize(&english), error.to_string());
    }

    /// A host an adapter does not name: the refused address first, the state in the language
    /// of the run, then the command that allows the host and where to report it.
    #[test]
    fn a_host_an_adapter_does_not_name_is_told_with_the_command_in_german() {
        let german = TEXTS.parse_lang_arg("de").unwrap();
        assert_eq!(
            host_not_named(Remedy::AllowHost, Some("https://www.gds.hessen.de/a"))
                .localize(&german),
            "https://cdn.example.net/x: freemapper sendet keine Anfragen an diese Adresse: Der \
             Adapter für Hessen nennt den Host cdn.example.net nicht \
             (https://www.gds.hessen.de/a leitet dorthin weiter). Ist der Dienst dorthin \
             umgezogen, geben Sie den Host mit `freemapper adapters allow-host hessen \
             cdn.example.net` frei und melden Sie es unter \
             https://github.com/HenryJobst/freemapper/issues"
        );
        assert_eq!(
            host_not_named(Remedy::Manifest, None).localize(&german),
            "https://cdn.example.net/x: freemapper sendet keine Anfragen an diese Adresse: Der \
             Adapter hessen nennt den Host cdn.example.net nicht. Ist der Dienst dorthin \
             umgezogen, tragen Sie den Host in `[net] hosts` seines Manifests ein"
        );
    }

    /// The errors of the permit notes: in English the library's text, in German worded anew.
    #[test]
    fn permits_errors_follow_the_language() {
        use freemapper::permits::{Missing, PermitsError};
        let nothing = PermitsError::NothingRead {
            missing: vec![
                Missing {
                    source: "vg250".into(),
                    reason: None,
                },
                Missing {
                    source: "bfn".into(),
                    reason: Some("HTTP 503".into()),
                },
                Missing {
                    source: "osm".into(),
                    reason: None,
                },
            ],
        };
        let stored = PermitsError::Stored {
            path: PathBuf::from("/p/data/permits/sources.json"),
            message: "expected value".into(),
        };
        let io = PermitsError::Io(PathBuf::from("/p/permits.md"), missing());
        for error in [&nothing, &stored, &io] {
            assert_eq!(error.localize(&Lang::english()), error.to_string());
        }
        assert_eq!(
            nothing.localize(&de()),
            "Keine Quelle der Genehmigungshinweise ließ sich lesen (vg250: nicht gelesen; bfn: \
             HTTP 503; osm: nicht gelesen); holen Sie sie zuerst online"
        );
        assert_eq!(
            stored.localize(&de()),
            "/p/data/permits/sources.json: expected value; holen Sie die Genehmigungshinweise \
             online neu"
        );
    }

    /// The reasons as the library lists them.
    fn freemapper_adapter_api_reasons() -> [&'static str; 12] {
        freemapper::fetch::POLICY_REASONS
    }
}
