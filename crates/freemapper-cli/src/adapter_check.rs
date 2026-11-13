//! `freemapper adapters check <dir>`: whether the folder of an adapter would load from the
//! adapter folder of a person, what its author should know before passing it on, and with
//! `--live` what its services deliver for a box inside its boundary (ADR 0082, spec 3.14).
//!
//! The check reads the folder as the loader does and asks no server unless `--live` says so.
//! A live check runs the fetch of a real project with this adapter alone, so its requests are
//! held to the hosts the manifest declares, redirects included.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;

use chrono::NaiveDate;
use freemapper::adapters::{RegionAdapter, backends, builtin, install_registry};
use freemapper::area::MapArea;
use freemapper::fetch::{Cache, FetchMode, Fetcher, HttpFetcher};
use freemapper::project::{MapKind, ProjectFile, fetch_project_sources, init_project};
use freemapper::sources::osm::{extracts_for, parse_state_date};
use freemapper_manifest::ManifestError;
use freemapper_manifest::check::{Checked, check_folder, covers};
use freemapper_manifest::loader::Loaded;

use crate::text::{n, text, v};

/// The box of a live check in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiveBox {
    min_lon: f64,
    min_lat: f64,
    max_lon: f64,
    max_lat: f64,
}

impl FromStr for LiveBox {
    type Err = String;

    /// `minLon,minLat,maxLon,maxLat`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let numbers: Vec<f64> = value
            .split(',')
            .map(|part| part.trim().parse::<f64>())
            .collect::<Result<_, _>>()
            .map_err(|_| "four numbers: minLon,minLat,maxLon,maxLat".to_string())?;
        let [min_lon, min_lat, max_lon, max_lat] = numbers[..] else {
            return Err("four numbers: minLon,minLat,maxLon,maxLat".to_string());
        };
        let on_earth = (-180.0..=180.0).contains(&min_lon)
            && (-180.0..=180.0).contains(&max_lon)
            && (-90.0..=90.0).contains(&min_lat)
            && (-90.0..=90.0).contains(&max_lat);
        if !on_earth || min_lon >= max_lon || min_lat >= max_lat {
            return Err(
                "a box in degrees with minLon < maxLon and minLat < maxLat (longitude first)"
                    .to_string(),
            );
        }
        Ok(Self {
            min_lon,
            min_lat,
            max_lon,
            max_lat,
        })
    }
}

impl LiveBox {
    /// The box as a map area.
    fn area(&self) -> MapArea {
        let (w, s, e, n) = (self.min_lon, self.min_lat, self.max_lon, self.max_lat);
        let geometry = serde_json::json!({
            "type": "Polygon",
            "coordinates": [[[w, s], [e, s], [e, n], [w, n], [w, s]]],
        });
        MapArea::from_geojson(&geometry.to_string()).expect("a rectangle is a polygon")
    }
}

/// What a capability of the adapter came to in a live check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Ok,
    Failed,
    NotAsked,
}

impl State {
    fn name(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::NotAsked => "not-asked",
        }
    }
}

/// One line of a live check: a capability of the adapter and what the services delivered.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    /// `rasters`, `alkis-buildings`, `alkis-land-use`, `nas`, `trees/<id>`, `osm` or `fetch`:
    /// the same in every language.
    capability: String,
    state: State,
    detail: String,
}

/// The result of a live check.
#[derive(Debug, Default)]
struct Live {
    rows: Vec<Row>,
    /// The warnings of the fetch, each a line with its prefix.
    warnings: Vec<String>,
}

/// What the check found.
struct Report {
    folder: PathBuf,
    /// The adapter, if the folder loads.
    adapter: Option<Loaded>,
    name: String,
    errors: Vec<ManifestError>,
    warnings: Vec<ManifestError>,
    live: Option<Live>,
}

impl Report {
    fn passed(&self) -> bool {
        self.errors.is_empty()
            && self
                .live
                .as_ref()
                .is_none_or(|live| live.rows.iter().all(|row| row.state != State::Failed))
    }

    /// The report as lines for a person.
    fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(adapter) = &self.adapter {
            lines.push(text(
                "cli-check-adapter",
                &[
                    ("adapter", v(&adapter.id)),
                    ("name", v(&self.name)),
                    ("revision", v(adapter.revision)),
                    ("hosts", v(adapter.hosts.join(", "))),
                ],
            ));
        }
        for finding in &self.errors {
            lines.push(text("cli-check-error", &[("finding", v(finding))]));
        }
        for finding in &self.warnings {
            lines.push(text("cli-check-warning", &[("finding", v(finding))]));
        }
        if let Some(live) = &self.live {
            for row in &live.rows {
                let id = match row.state {
                    State::Ok => "cli-check-live-ok",
                    State::Failed => "cli-check-live-failed",
                    State::NotAsked => "cli-check-live-not-asked",
                };
                lines.push(text(
                    id,
                    &[
                        ("capability", v(&row.capability)),
                        ("detail", v(&row.detail)),
                    ],
                ));
            }
            lines.extend(live.warnings.iter().cloned());
        }
        let folder = ("folder", v(self.folder.display()));
        lines.push(if !self.passed() {
            text("cli-check-failed", &[folder])
        } else if self.warnings.is_empty() {
            text("cli-check-passed", &[folder])
        } else {
            text(
                "cli-check-passed-warnings",
                &[folder, ("count", n(self.warnings.len()))],
            )
        });
        lines
    }

    fn json(&self) -> String {
        let finding = |severity: &str, finding: &ManifestError| {
            serde_json::json!({
                "severity": severity,
                "code": finding.code,
                "file": finding.path.display().to_string(),
                "line": finding.line,
                "key": finding.key,
                "message": finding.message,
            })
        };
        let findings: Vec<serde_json::Value> = self
            .errors
            .iter()
            .map(|found| finding("error", found))
            .chain(self.warnings.iter().map(|found| finding("warning", found)))
            .collect();
        let live = self.live.as_ref().map(|live| {
            live.rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "capability": row.capability,
                        "state": row.state.name(),
                        "detail": row.detail,
                    })
                })
                .collect::<Vec<_>>()
        });
        serde_json::to_string_pretty(&serde_json::json!({
            "folder": self.folder.display().to_string(),
            "passed": self.passed(),
            "adapter": self.adapter.as_ref().map(|adapter| serde_json::json!({
                "id": adapter.id,
                "revision": adapter.revision,
                "hosts": adapter.hosts,
                "last_verified": adapter.last_verified.map(|day| day.to_string()),
            })),
            "findings": findings,
            "live": live,
            "live_warnings": self
                .live
                .as_ref()
                .map(|live| live.warnings.clone())
                .unwrap_or_default(),
        }))
        .expect("the report is plain data")
    }
}

/// The static part of the check: the report, and the adapter with its registry if it loads.
fn check(dir: &Path, today: NaiveDate, german: bool) -> (Report, Option<Checked>) {
    // The folder as it was named, without following a link: the name is the id.
    let folder = std::path::absolute(dir).unwrap_or_else(|_| dir.to_path_buf());
    let mut report = Report {
        folder: folder.clone(),
        adapter: None,
        name: String::new(),
        errors: Vec::new(),
        warnings: Vec::new(),
        live: None,
    };
    match check_folder(
        &folder,
        backends(),
        &builtin(),
        crate::adapters::running(),
        today,
    ) {
        Ok(checked) => {
            let names = &checked.adapter().info().names;
            report.name = if german { &names.de } else { &names.en }.clone();
            report.adapter = Some(checked.loaded.clone());
            report.warnings = checked.warnings.clone();
            (report, Some(checked))
        }
        Err(findings) => {
            report.errors = findings;
            (report, None)
        }
    }
}

/// What the capabilities of `adapter` came to: `project` is the project file after the fetch,
/// `failure` the error that stopped it, `asked_trees` the tree cadastres the box needs.
fn rows(
    adapter: &dyn RegionAdapter,
    project: Option<&ProjectFile>,
    failure: Option<String>,
    asked_trees: &[String],
) -> Vec<Row> {
    let mut failure = failure;
    let stopped = failure.is_some();
    let mut rows = Vec::new();
    // A capability that is declared and did not arrive: the first one after a stop carries
    // the error, the later ones were never asked; without a stop a warning says why.
    let mut missing = |capability: &str| {
        let (state, detail) = match failure.take() {
            Some(error) => (State::Failed, error),
            None if stopped => (State::NotAsked, text("cli-check-detail-stopped", &[])),
            None => (State::Failed, text("cli-check-detail-see-warnings", &[])),
        };
        Row {
            capability: capability.to_string(),
            state,
            detail,
        }
    };
    let row = |capability: &str, state: State, detail: String| Row {
        capability: capability.to_string(),
        state,
        detail,
    };
    let undeclared = |capability: &str| {
        row(
            capability,
            State::NotAsked,
            text("cli-check-detail-not-declared", &[]),
        )
    };
    let objects = |count: usize| text("cli-check-detail-objects", &[("count", n(count))]);

    // The rasters: what is stored for the area, and the tiles by product.
    let mut parts = Vec::new();
    if let Some(project) = project {
        for (there, id) in [
            (project.orthophoto.is_some(), "cli-check-raster-orthophoto"),
            (project.terrain.is_some(), "cli-check-raster-terrain"),
            (project.surface.is_some(), "cli-check-raster-surface"),
        ] {
            if there {
                parts.push(text(id, &[]));
            }
        }
        let mut products: Vec<(&str, usize)> = Vec::new();
        for tile in &project.tiles {
            let label = tile.product.label();
            match products.iter_mut().find(|(known, _)| *known == label) {
                Some((_, count)) => *count += 1,
                None => products.push((label, 1)),
            }
        }
        if !project.ndsm_tiles.is_empty() {
            products.push(("nDSM", project.ndsm_tiles.len()));
        }
        for (label, count) in products {
            parts.push(text(
                "cli-check-raster-tiles",
                &[("product", v(label)), ("count", n(count))],
            ));
        }
    }
    rows.push(match (adapter.rasters().is_some(), parts.is_empty()) {
        (false, _) => undeclared("rasters"),
        (true, true) => missing("rasters"),
        (true, false) => row("rasters", State::Ok, parts.join(", ")),
    });

    let alkis = adapter.alkis().is_some();
    for (capability, record) in [
        (
            "alkis-buildings",
            project.and_then(|project| project.buildings.as_ref()),
        ),
        (
            "alkis-land-use",
            project.and_then(|project| project.land_use.as_ref()),
        ),
    ] {
        rows.push(match (alkis, record) {
            (false, _) => undeclared(capability),
            (true, Some(record)) => row(capability, State::Ok, objects(record.count)),
            (true, None) => missing(capability),
        });
    }

    let layers = project.map(|project| &project.nas_layers);
    rows.push(match (adapter.nas().is_some(), layers) {
        (false, _) => undeclared("nas"),
        (true, Some(layers)) if !layers.is_empty() => row(
            "nas",
            State::Ok,
            text(
                "cli-check-detail-layers",
                &[
                    ("layers", n(layers.len())),
                    (
                        "count",
                        n(layers.values().map(|record| record.count).sum::<usize>()),
                    ),
                ],
            ),
        ),
        (true, _) => missing("nas"),
    });

    for dataset in adapter.tree_datasets() {
        let capability = format!("trees/{}", dataset.id);
        let record = project.and_then(|project| project.tree_layers.get(dataset.id.as_ref()));
        rows.push(match record {
            Some(record) => row(
                &capability,
                State::Ok,
                text("cli-check-detail-trees", &[("count", n(record.count))]),
            ),
            None if !asked_trees.iter().any(|asked| asked.as_str() == dataset.id) => row(
                &capability,
                State::NotAsked,
                text("cli-check-detail-outside-dataset", &[]),
            ),
            None => missing(&capability),
        });
    }

    // A stop no capability stands for (the area, the disk, a file).
    if let Some(error) = failure {
        rows.push(row("fetch", State::Failed, error));
    }
    rows
}

/// Asks Geofabrik for the state of each OSM extract a map of `bbox` (EPSG:25833) reads.
fn osm_rows(fetcher: &dyn Fetcher, bbox: &freemapper::crs::BoundingBox) -> Vec<Row> {
    extracts_for(bbox, &mut |_| {})
        .into_iter()
        .map(|extract| {
            let date = fetcher.get(&extract.state_url()).map(|state| {
                parse_state_date(&String::from_utf8_lossy(&state))
                    .map_or_else(|| "?".to_string(), |date| date.to_string())
            });
            let (state, detail) = match date {
                Ok(date) => (
                    State::Ok,
                    text(
                        "cli-check-detail-extract",
                        &[("label", v(extract.label())), ("date", v(date))],
                    ),
                ),
                Err(error) => (
                    State::Failed,
                    crate::errors::message(&error, &crate::text::lang()),
                ),
            };
            Row {
                capability: "osm".to_string(),
                state,
                detail,
            }
        })
        .collect()
}

/// Why a live check cannot run.
#[derive(Debug)]
struct CannotRun(String);

/// The live part: a project for `live_box` in a folder of its own, fetched with the adapter
/// of `checked` alone. The registry of the process becomes that adapter's.
fn live(checked: Checked, live_box: &LiveBox, cache: &Cache) -> Result<Live, CannotRun> {
    let id = checked.loaded.id.clone();
    let area = live_box.area();
    let bbox = area.projected_bbox();
    if !covers(checked.adapter(), &bbox) {
        return Err(CannotRun(text(
            "cli-check-live-box-outside",
            &[("adapter", v(&id))],
        )));
    }
    let cannot =
        |detail: String| CannotRun(text("cli-check-live-cannot", &[("detail", v(detail))]));
    install_registry(checked.registry)
        .map_err(|_| cannot("the adapters of the program were asked before the check".into()))?;
    let adapter = freemapper::adapters::registry()
        .get(&id)
        .expect("the adapter that was checked is installed");
    let scratch = tempfile::tempdir().map_err(|e| cannot(e.to_string()))?;
    let dir = scratch.path().join(&id);
    let today = chrono::Local::now().date_naive();
    let lang = crate::text::lang();
    let mut live = Live::default();
    let mut failure = init_project(&dir, &area, MapKind::Sprint, None, today, None)
        .err()
        .map(|error| crate::errors::message(&error, &lang));
    let fetcher = HttpFetcher::new();
    if failure.is_none() {
        let fetched =
            fetch_project_sources(&dir, &fetcher, cache, FetchMode::Online, &mut |note| {
                if note.is_warning() {
                    live.warnings.push(crate::pipeline::line(&note));
                }
            });
        failure = fetched
            .err()
            .map(|error| crate::errors::message(&error, &lang));
    }
    // What the fetch stored before it ended, with or without an error.
    let project = ProjectFile::load(&dir).ok();
    let asked_trees: Vec<String> = freemapper::sources::trees::tree_datasets_for(&bbox)
        .into_iter()
        .map(|dataset| dataset.id.to_string())
        .collect();
    live.rows = rows(adapter, project.as_ref(), failure, &asked_trees);
    live.rows.extend(osm_rows(&fetcher, &bbox));
    Ok(live)
}

/// Runs the check of the adapter folder `dir` and prints the report. The exit code: 0 the
/// adapter passes (warnings or not), 1 it has findings or a service failed, 2 the check could
/// not run.
pub fn run(
    dir: &Path,
    live_box: Option<LiveBox>,
    json: bool,
    cache: impl FnOnce() -> anyhow::Result<Cache>,
) -> anyhow::Result<ExitCode> {
    let cannot_run = |message: String| {
        eprintln!("{}: {message}", text("cli-error-prefix", &[]));
        ExitCode::from(2)
    };
    if !dir.exists() {
        return Ok(cannot_run(text(
            "cli-check-no-folder",
            &[("folder", v(dir.display()))],
        )));
    }
    let today = chrono::Local::now().date_naive();
    let german = crate::text::lang().code() == "de";
    let (mut report, checked) = check(dir, today, german);
    if let (Some(checked), Some(live_box)) = (checked, live_box) {
        match live(checked, &live_box, &cache()?) {
            Ok(live) => report.live = Some(live),
            Err(CannotRun(message)) => return Ok(cannot_run(message)),
        }
    }
    if json {
        println!("{}", report.json());
    } else {
        for line in report.lines() {
            println!("{line}");
        }
    }
    Ok(if report.passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifests() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../adapters/de/manifests")
    }

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
    }

    const SQUARE: &str = r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
        [[[600000,5300000],[640000,5300000],[640000,5340000],[600000,5340000],[600000,5300000]]]}}"#;

    /// The folder of a valid adapter `mine` in `dir`, far from the built-in states.
    fn write(dir: &Path) -> PathBuf {
        let folder = dir.join("mine");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join("manifest.toml"),
            "format = 1\n\n[adapter]\nid = \"mine\"\nrevision = 2\nmin_freemapper = \"0.13.0\"\n\
             crs = 25833\nnames = { en = \"Mine\", de = \"Meins\" }\nlast_verified = \"2026-10-01\"\n\n\
             [net]\nhosts = [\"geo.example.org\"]\n\n[boundary]\nfile = \"boundary.geojson\"\n\
             crs = 25833\n\n[[osm.extract]]\ngeofabrik = \"europe/germany/bayern\"\nmegabytes = 700\n\
             label = \"Mine\"\n\n[attribution]\nlicence = \"dl-de/by-2-0\"\ntext = \"© Mine {year}\"\n\n\
             [alkis]\nkind = \"wfs\"\nurl = \"https://geo.example.org/wfs\"\n\
             buildings_type = \"ave:GebaeudeBauwerk\"\nland_use_type = \"ave:NutzungFlurstueck\"\n",
        )
        .unwrap();
        std::fs::write(folder.join("boundary.geojson"), SQUARE).unwrap();
        folder
    }

    /// The manifest folder of a built-in state passes: it loads with the backends of the
    /// program. That its id is built in is a warning, since from a folder it would not load.
    #[test]
    fn check_passes_the_hamburg_folder() {
        let (report, checked) = check(&manifests().join("hamburg"), today(), false);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(report.passed());
        let codes: Vec<&str> = report.warnings.iter().map(|w| w.code).collect();
        assert_eq!(codes, ["check.built-in-id"]);
        let lines = report.lines();
        assert!(
            lines[0].starts_with("adapter hamburg (Hamburg), revision 1, hosts: "),
            "{}",
            lines[0]
        );
        assert!(lines[1].starts_with("warning: "), "{}", lines[1]);
        assert!(lines[1].ends_with("[check.built-in-id]"), "{}", lines[1]);
        assert!(
            lines[2].ends_with("hamburg passes the check, with one warning"),
            "{}",
            lines[2]
        );
        assert_eq!(lines.len(), 3);
        // The adapter is held to its hosts like any adapter from a folder.
        assert!(checked.unwrap().adapter().policy().is_some());
    }

    /// Every built-in manifest folder passes the check it will be given before a change.
    #[test]
    fn every_built_in_manifest_folder_passes() {
        let mut ids = Vec::new();
        for entry in std::fs::read_dir(manifests()).unwrap() {
            let folder = entry.unwrap().path();
            if !folder.is_dir() {
                continue;
            }
            let (report, _) = check(&folder, today(), false);
            assert!(report.errors.is_empty(), "{:?}", report.errors);
            ids.push(report.adapter.unwrap().id);
        }
        assert_eq!(ids.len(), 13);
    }

    #[test]
    fn check_fails_a_boundary_in_the_wrong_crs_with_code_boundary_crs() {
        let dir = tempfile::tempdir().unwrap();
        let folder = write(dir.path());
        std::fs::write(
            folder.join("boundary.geojson"),
            r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
            [[[9.7,53.4],[10.3,53.4],[10.3,53.7],[9.7,53.7],[9.7,53.4]]]}}"#,
        )
        .unwrap();
        let (report, checked) = check(&folder, today(), false);
        assert!(checked.is_none() && !report.passed());
        assert_eq!(report.errors[0].code, "boundary.crs");
        let lines = report.lines();
        assert!(lines[0].starts_with("error: "), "{}", lines[0]);
        assert!(lines[0].ends_with("[boundary.crs]"), "{}", lines[0]);
        assert!(lines[1].ends_with("mine fails the check"), "{}", lines[1]);
    }

    #[cfg(unix)]
    #[test]
    fn check_fails_a_symlinked_folder() {
        let dir = tempfile::tempdir().unwrap();
        let real = write(dir.path());
        let link = dir.path().join("links").join("mine");
        std::fs::create_dir_all(link.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let (report, _) = check(&link, today(), false);
        assert_eq!(report.errors[0].code, "adapter.folder");
        assert!(!report.passed());
    }

    #[test]
    fn check_json_output_lists_codes() {
        let dir = tempfile::tempdir().unwrap();
        let folder = write(dir.path());
        let (report, _) = check(&folder, today(), true);
        assert_eq!(report.name, "Meins");
        let json: serde_json::Value = serde_json::from_str(&report.json()).unwrap();
        assert_eq!(json["passed"], true);
        assert_eq!(json["adapter"]["id"], "mine");
        assert_eq!(json["adapter"]["revision"], 2);
        assert_eq!(json["adapter"]["hosts"][0], "geo.example.org");
        assert_eq!(json["adapter"]["last_verified"], "2026-10-01");
        assert_eq!(json["findings"], serde_json::json!([]));
        assert_eq!(json["live"], serde_json::Value::Null);
        // A manifest with two defects: both codes, each with its place.
        let path = folder.join("manifest.toml");
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(
            &path,
            text.replace("https://geo.example.org/wfs", "http://geo.example.org/wfs")
                .replace("revision = 2", "revision = 2\ncolour = \"red\""),
        )
        .unwrap();
        let (report, _) = check(&folder, today(), false);
        let json: serde_json::Value = serde_json::from_str(&report.json()).unwrap();
        assert_eq!(json["passed"], false);
        assert_eq!(json["adapter"], serde_json::Value::Null);
        let findings = json["findings"].as_array().unwrap();
        assert!(!findings.is_empty());
        for finding in findings {
            assert_eq!(finding["severity"], "error");
            assert!(finding["code"].as_str().unwrap().contains('.'), "{finding}");
            assert!(
                finding["file"].as_str().unwrap().ends_with("manifest.toml"),
                "{finding}"
            );
        }
    }

    #[test]
    fn live_box_outside_the_boundary_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let folder = write(dir.path());
        let (_, checked) = check(&folder, today(), false);
        // Strausberg, far from the square of the adapter.
        let strausberg: LiveBox = "13.84,52.53,13.85,52.54".parse().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let CannotRun(message) = live(checked.unwrap(), &strausberg, &cache).unwrap_err();
        assert_eq!(
            message,
            "the box does not lie inside the boundary of the adapter mine; a live check asks \
             the services for a place the adapter covers"
        );
        // Nothing was written, and the registry of the process is untouched.
        assert!(!dir.path().join("cache").exists());
    }

    #[test]
    fn a_box_is_four_numbers_longitude_first() {
        let parsed: LiveBox = "9.97, 53.55,9.98,53.56".parse().unwrap();
        assert_eq!(
            parsed,
            LiveBox {
                min_lon: 9.97,
                min_lat: 53.55,
                max_lon: 9.98,
                max_lat: 53.56
            }
        );
        for wrong in [
            "",
            "9.97,53.55,9.98",
            "a,b,c,d",
            "9.98,53.55,9.97,53.56",
            "53.55,99.97,53.56,99.98",
        ] {
            assert!(wrong.parse::<LiveBox>().is_err(), "{wrong}");
        }
        // 0.01 degrees of longitude at Hamburg, in EPSG:25833 (turned, far from its meridian).
        let bbox = parsed.area().projected_bbox();
        let width = bbox.max_e - bbox.min_e;
        assert!((600.0..800.0).contains(&width), "{bbox:?}");
    }

    /// The lines of a live check: what arrived, what failed with the error of the fetch, and
    /// what was never asked after it.
    #[test]
    fn live_rows_say_what_arrived_and_what_the_stop_left_out() {
        let dir = tempfile::tempdir().unwrap();
        let folder = write(dir.path());
        let (_, checked) = check(&folder, today(), false);
        let checked = checked.unwrap();
        let adapter = checked.adapter();
        let line = |row: &Row| format!("{} {} {}", row.capability, row.state.name(), row.detail);
        // Nothing arrived, and the fetch stopped with an error: the buildings carry it.
        let stopped = rows(adapter, None, Some("HTTP 404".to_string()), &[]);
        assert_eq!(
            stopped.iter().map(line).collect::<Vec<_>>(),
            [
                "rasters not-asked the adapter declares none",
                "alkis-buildings failed HTTP 404",
                "alkis-land-use not-asked an earlier part failed",
                "nas not-asked the adapter declares none",
            ]
        );
        // Without a stop a missing part points at the warnings.
        let soft = rows(adapter, None, None, &[]);
        assert_eq!(line(&soft[2]), "alkis-land-use failed see the warnings");
        let report = Report {
            folder: folder.clone(),
            adapter: Some(checked.loaded.clone()),
            name: "Mine".to_string(),
            errors: Vec::new(),
            warnings: Vec::new(),
            live: Some(Live {
                rows: stopped,
                warnings: vec!["warning: slow".to_string()],
            }),
        };
        assert!(!report.passed());
        let lines = report.lines();
        assert_eq!(lines[2], "alkis-buildings FAILED: HTTP 404");
        assert_eq!(lines[3], "alkis-land-use not asked: an earlier part failed");
        assert_eq!(lines[5], "warning: slow");
        assert!(lines[6].ends_with("mine fails the check"), "{}", lines[6]);
        let json: serde_json::Value = serde_json::from_str(&report.json()).unwrap();
        assert_eq!(json["live"][1]["state"], "failed");
        assert_eq!(json["live_warnings"][0], "warning: slow");
    }
}
