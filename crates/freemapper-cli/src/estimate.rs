//! The download estimate as JSON, shared by the selection page and the MCP server.

use std::path::Path;

use freemapper::fetch::Cache;
use freemapper::layers::DownloadEstimate;
use freemapper::space::RESERVE_BYTES;
use serde_json::{Value, json};

use crate::errors::Localize;
use crate::text::{Lang, TEXTS, v};

/// The download estimate as the page and the MCP tool show it: each download with its size,
/// whether it is cached and whether it fits on the disk, and under `disk` the free space of the
/// volumes of the cache and of the project folder `dir` with what the fetch needs there, the
/// messages for what does not fit and whether the fetch stops; `error` when the free space
/// cannot be read. The messages and the error are in `lang`.
pub(crate) fn estimate_json(
    estimate: &DownloadEstimate,
    cache: &Cache,
    dir: &Path,
    lang: &Lang,
) -> Value {
    let plan = freemapper::space::plan(estimate, cache.root(), dir, cache.disk_space());
    let items: Vec<Value> = estimate
        .items
        .iter()
        .map(|i| {
            let fits = i.cached || plan.as_ref().map_or(true, |plan| plan.fits(i.part));
            json!({ "what": i.what, "megabytes": i.megabytes, "cached": i.cached, "fits": fits })
        })
        .collect();
    let megabytes = |bytes: u64| bytes.div_ceil(1 << 20);
    let disk = match &plan {
        Ok(plan) => json!({
            "reserve_megabytes": megabytes(RESERVE_BYTES),
            "volumes": plan.volumes.iter().map(|v| {
                let holds: Vec<&str> = [(v.holds_cache, "cache"), (v.holds_project, "project")]
                    .into_iter()
                    .filter_map(|(holds, what)| holds.then_some(what))
                    .collect();
                json!({
                    "path": v.path,
                    "holds": holds,
                    "free_megabytes": v.available / (1 << 20),
                    "needed_megabytes": megabytes(v.needed),
                })
            }).collect::<Vec<_>>(),
            "messages": plan.short.iter().map(|s| s.localize(lang)).collect::<Vec<_>>(),
            // A part the run needs does not fit: the fetch stops before it downloads anything.
            "stops": plan.missing().is_some(),
        }),
        Err((path, e)) => json!({
            "error": TEXTS.text(
                lang,
                "cli-estimate-disk-unreadable",
                &[("path", v(path.display())), ("message", v(e))],
            ),
        }),
    };
    json!({
        "tiles": estimate.tiles,
        "items": items,
        "total_megabytes": estimate.total_megabytes,
        "disk": disk,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    use freemapper::layers::EstimateItem;
    use freemapper::space::{DiskSpace, Part, Store, Volume};

    use crate::text::TEXTS;

    fn german() -> Lang {
        TEXTS.parse_lang_arg("de").unwrap()
    }

    /// The cache on one volume with `cache_mb` free, everything else on another with
    /// `project_mb`.
    #[derive(Debug)]
    struct TwoDisks {
        cache: PathBuf,
        cache_mb: u64,
        project_mb: u64,
    }

    impl DiskSpace for TwoDisks {
        fn volume(&self, path: &Path) -> std::io::Result<Volume> {
            let (id, mb) = if path.starts_with(&self.cache) {
                ("ssd", self.cache_mb)
            } else {
                ("usb", self.project_mb)
            };
            Ok(Volume {
                id: id.to_string(),
                available: mb << 20,
            })
        }
    }

    #[test]
    fn estimate_shows_free_space() {
        let item = |what: &str, megabytes, cached, part, store| EstimateItem {
            what: what.to_string(),
            megabytes,
            cached,
            part,
            store,
        };
        let estimate = DownloadEstimate {
            tiles: Vec::new(),
            items: vec![
                item("OSM extract", 300.0, false, Part::MapData, Store::Cache),
                item("orthophoto", 20.0, false, Part::Orthophoto, Store::Project),
                item("surface model", 200.0, false, Part::Surface, Store::Cache),
                item("terrain model", 10.0, true, Part::Terrain, Store::Cache),
            ],
            total_megabytes: 520.0,
            in_place: Vec::new(),
        };
        let cache = Cache::new(PathBuf::from("/ssd/cache")).with_disk_space(Arc::new(TwoDisks {
            cache: PathBuf::from("/ssd"),
            cache_mb: 1024 + 400,
            project_mb: 5000,
        }));
        let json = estimate_json(
            &estimate,
            &cache,
            Path::new("/usb/school"),
            &Lang::english(),
        );
        let disk = &json["disk"];
        assert_eq!(disk["reserve_megabytes"], 1024);
        assert_eq!(
            disk["volumes"],
            json!([
                { "path": "/ssd/cache", "holds": ["cache"], "free_megabytes": 1424, "needed_megabytes": 500 },
                { "path": "/usb/school", "holds": ["project"], "free_megabytes": 5000, "needed_megabytes": 140 },
            ])
        );
        let fits: Vec<bool> = json["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["fits"].as_bool().unwrap())
            .collect();
        assert_eq!(fits, [true, true, false, true]);
        assert_eq!(
            disk["messages"],
            json!([
                "freemapper skips the surface model for the nDSM template for lack of disk space. It needs 200 MB, but /ssd/cache has only 100 MB free beyond the 1.0 GB reserve. Free some space and run `freemapper fetch` again"
            ])
        );
        // Leaving out the surface model lets the fetch go on.
        assert_eq!(disk["stops"], false);
        // Without room for the OSM extract it stops.
        let full = Cache::new(PathBuf::from("/ssd/cache")).with_disk_space(Arc::new(TwoDisks {
            cache: PathBuf::from("/ssd"),
            cache_mb: 1024 + 100,
            project_mb: 5000,
        }));
        let json = estimate_json(&estimate, &full, Path::new("/usb/school"), &Lang::english());
        assert_eq!(json["disk"]["stops"], true);
        assert!(
            json["disk"]["messages"][0]
                .as_str()
                .unwrap()
                .starts_with("not enough disk space for the ALKIS and OSM map data."),
            "{json}"
        );
    }

    #[test]
    fn estimate_says_when_free_space_cannot_be_read() {
        #[derive(Debug)]
        struct Unreadable;
        impl DiskSpace for Unreadable {
            fn volume(&self, _path: &Path) -> std::io::Result<Volume> {
                Err(std::io::Error::other("not supported"))
            }
        }
        let estimate = DownloadEstimate {
            tiles: Vec::new(),
            items: Vec::new(),
            total_megabytes: 0.0,
            in_place: Vec::new(),
        };
        let cache = Cache::new(PathBuf::from("/net/cache")).with_disk_space(Arc::new(Unreadable));
        let json = estimate_json(
            &estimate,
            &cache,
            Path::new("/net/school"),
            &Lang::english(),
        );
        assert_eq!(
            json["disk"]["error"],
            "cannot read the free disk space of /net/cache (not supported)"
        );
        let json = estimate_json(&estimate, &cache, Path::new("/net/school"), &german());
        assert_eq!(
            json["disk"]["error"],
            "Der freie Speicherplatz von /net/cache lässt sich nicht lesen (not supported)"
        );
    }

    /// The selection page asks in its own language, which need not be the program's.
    #[test]
    fn estimate_speaks_the_language_it_is_given() {
        let estimate = DownloadEstimate {
            tiles: Vec::new(),
            items: vec![EstimateItem {
                what: "surface model".to_string(),
                megabytes: 200.0,
                cached: false,
                part: Part::Surface,
                store: Store::Cache,
            }],
            total_megabytes: 200.0,
            in_place: Vec::new(),
        };
        let cache = Cache::new(PathBuf::from("/ssd/cache")).with_disk_space(Arc::new(TwoDisks {
            cache: PathBuf::from("/ssd"),
            cache_mb: 1024 + 100,
            project_mb: 5000,
        }));
        let json = estimate_json(&estimate, &cache, Path::new("/usb/school"), &german());
        let message = json["disk"]["messages"][0].as_str().unwrap();
        assert!(
            message.starts_with("Aus Platzmangel lässt freemapper ")
                && message.contains("/ssd/cache"),
            "{message}"
        );
    }
}
