//! The region adapters of a run: those built into the program, those of the user's folder and,
//! when the person asks for them, those of a project (ADR 0082, spec 3.7).
//!
//! The registry is assembled once, before anything asks it. An adapter from a folder cannot
//! replace one the program has; a folder with a defect is left out with a warning. The MCP
//! server and the area page never load the adapters of a project: a project that was passed on
//! must not bring a source along.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use freemapper::adapters::{Registry, builtin, install_registry, registry};
use freemapper::fetch::HttpFetcher;
use freemapper_manifest::loader::{AdapterDir, AdapterWarning, Loaded, Origin, load_dirs};
use freemapper_manifest::model::Version;

use crate::text::{text, v};

/// The variable that names the folder of the user's adapters in place of the default.
pub const ADAPTER_DIR_VARIABLE: &str = "FREEMAPPER_ADAPTER_DIR";

/// The folder of the user's adapters: the one `FREEMAPPER_ADAPTER_DIR` names, else
/// `freemapper/adapters` in the configuration folder of the system.
pub fn user_dir() -> Option<PathBuf> {
    user_dir_from(std::env::var_os(ADAPTER_DIR_VARIABLE), dirs::config_dir())
}

fn user_dir_from(variable: Option<OsString>, config: Option<PathBuf>) -> Option<PathBuf> {
    match variable.filter(|value| !value.is_empty()) {
        Some(dir) => Some(PathBuf::from(dir)),
        None => config.map(|config| config.join("freemapper").join("adapters")),
    }
}

/// The folder of the adapters of the project `project`.
pub fn project_dir(project: &Path) -> PathBuf {
    project.join("adapters")
}

/// The version of this freemapper, which a manifest may ask for as its oldest.
fn running() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("the version of the package is x.y.z")
}

/// The adapters of a run and what the folders had to say.
pub struct Assembled {
    pub registry: Registry,
    /// The adapters that came from a folder, in the order they were loaded.
    pub loaded: Vec<Loaded>,
    pub warnings: Vec<AdapterWarning>,
}

/// The built-in adapters, then those of `user`, then those of `project` (the folder
/// `adapters` of a project folder), each only if given.
pub fn assemble(user: Option<&Path>, project: Option<&Path>) -> Assembled {
    let mut registry = builtin();
    let dirs: Vec<AdapterDir> = [(user, Origin::User), (project, Origin::Project)]
        .into_iter()
        .filter_map(|(path, origin)| {
            Some(AdapterDir {
                path: path?.to_path_buf(),
                origin,
            })
        })
        .collect();
    let (loaded, warnings) = load_dirs(&mut registry, &dirs, running());
    Assembled {
        registry,
        loaded,
        warnings,
    }
}

/// The warnings about the folders, one line per finding.
fn warning_lines(assembled: &Assembled) -> Vec<String> {
    let mut lines = Vec::new();
    for warning in &assembled.warnings {
        for finding in &warning.findings {
            let id = if warning.skipped {
                "cli-adapter-skipped"
            } else {
                "cli-adapter-note"
            };
            lines.push(text(
                id,
                &[
                    ("folder", v(warning.folder.display())),
                    ("finding", v(finding)),
                ],
            ));
        }
    }
    lines
}

/// The lines for the person about the folders: every warning, and every adapter of a project
/// with the hosts it asks.
fn report_lines(assembled: &Assembled) -> Vec<String> {
    let mut lines = warning_lines(assembled);
    for adapter in &assembled.loaded {
        if adapter.origin == Origin::Project {
            lines.push(text(
                "cli-adapter-from-project",
                &[
                    ("adapter", v(&adapter.id)),
                    ("hosts", v(adapter.hosts.join(", "))),
                ],
            ));
        }
    }
    lines
}

/// Makes the adapters of this run the registry of the process: the built-in ones, those of the
/// user's folder and, with `project` (a project folder), those of its folder `adapters`. What
/// the folders had to say goes to stderr. Called once, before anything asks the registry.
pub fn install(project: Option<&Path>) {
    let project = project.map(project_dir);
    let assembled = assemble(user_dir().as_deref(), project.as_deref());
    for line in report_lines(&assembled) {
        eprintln!("{line}");
    }
    // A registry that is there already (a second call) stays.
    let _ = install_registry(assembled.registry);
}

/// The fetcher of a run. With an adapter from a folder among the adapters, every request stays
/// on the hosts the adapters declare (redirects included); the program's own adapters alone
/// ask as their code asks.
pub fn fetcher() -> HttpFetcher {
    let from_folders = registry()
        .adapters()
        .any(|adapter| !is_built_in(adapter.info().id.as_str()));
    if from_folders {
        HttpFetcher::with_policy(freemapper::adapters::declared_policy())
    } else {
        HttpFetcher::new()
    }
}

/// Whether the adapter `id` is one of the program's own.
fn is_built_in(id: &str) -> bool {
    static BUILT_IN: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    BUILT_IN
        .get_or_init(|| {
            builtin()
                .adapters()
                .map(|adapter| adapter.info().id.to_string())
                .collect()
        })
        .iter()
        .any(|known| known == id)
}

/// `freemapper adapters list`: every adapter with where it comes from, its revision, the day
/// it was last checked and the hosts it asks. `project` also reads the adapters of that
/// project folder, as `--adapters-from-project` would.
pub fn list(json: bool, project: Option<&Path>) {
    let user = user_dir();
    let project = project.map(project_dir);
    let assembled = assemble(user.as_deref(), project.as_deref());
    if json {
        println!("{}", list_json(&assembled, user.as_deref()));
        return;
    }
    for line in warning_lines(&assembled) {
        eprintln!("{line}");
    }
    for line in list_lines(&assembled, user.as_deref()) {
        println!("{line}");
    }
}

/// Where the adapter `id` comes from, as the JSON says it.
fn source_of(assembled: &Assembled, id: &str) -> &'static str {
    match assembled.loaded.iter().find(|loaded| loaded.id == id) {
        Some(loaded) if loaded.origin == Origin::Project => "project",
        Some(_) => "user",
        None => "built-in",
    }
}

fn list_lines(assembled: &Assembled, user: Option<&Path>) -> Vec<String> {
    let mut lines: Vec<String> = assembled
        .registry
        .adapters()
        .map(|adapter| {
            let info = adapter.info();
            let source = match source_of(assembled, info.id.as_str()) {
                "project" => text("cli-adapters-source-project", &[]),
                "user" => text("cli-adapters-source-user", &[]),
                _ => text("cli-adapters-source-built-in", &[]),
            };
            let declared = adapter.declared();
            let checked = declared
                .and_then(|declared| declared.last_verified)
                .map_or_else(
                    || text("cli-adapters-never-checked", &[]),
                    |day| day.to_string(),
                );
            text(
                "cli-adapters-line",
                &[
                    ("adapter", v(&info.id)),
                    ("name", v(&info.names.en)),
                    ("source", v(source)),
                    (
                        "revision",
                        v(declared.map_or(0, |declared| declared.revision)),
                    ),
                    ("checked", v(checked)),
                    (
                        "hosts",
                        v(declared.map_or_else(String::new, |declared| declared.hosts.join(", "))),
                    ),
                ],
            )
        })
        .collect();
    lines.push(match user {
        Some(dir) => text("cli-adapters-user-dir", &[("path", v(dir.display()))]),
        None => text("cli-adapters-no-user-dir", &[]),
    });
    lines
}

fn list_json(assembled: &Assembled, user: Option<&Path>) -> String {
    let adapters: Vec<serde_json::Value> = assembled
        .registry
        .adapters()
        .map(|adapter| {
            let info = adapter.info();
            let declared = adapter.declared();
            serde_json::json!({
                "id": info.id.as_str(),
                "names": { "en": info.names.en, "de": info.names.de },
                "provider": info.provider.as_str(),
                "source": source_of(assembled, info.id.as_str()),
                "revision": declared.map(|declared| declared.revision),
                "last_verified": declared
                    .and_then(|declared| declared.last_verified)
                    .map(|day| day.to_string()),
                "hosts": declared.map(|declared| declared.hosts.clone()).unwrap_or_default(),
            })
        })
        .collect();
    let warnings: Vec<serde_json::Value> = assembled
        .warnings
        .iter()
        .map(|warning| {
            serde_json::json!({
                "folder": warning.folder.display().to_string(),
                "skipped": warning.skipped,
                "findings": warning.findings.iter().map(|finding| serde_json::json!({
                    "code": finding.code,
                    "file": finding.path.display().to_string(),
                    "line": finding.line,
                    "key": finding.key,
                    "message": finding.message,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "user_dir": user.map(|dir| dir.display().to_string()),
        "adapters": adapters,
        "warnings": warnings,
    }))
    .expect("the list is plain data")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE: &str = r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
        [[[400000,5800000],[401000,5800000],[401000,5801000],[400000,5801000],[400000,5800000]]]}}"#;

    /// A valid manifest for the adapter `id`, whose services are on `geo.example.org`.
    fn manifest(id: &str) -> String {
        format!(
            r#"format = 1

[adapter]
id = "{id}"
revision = 3
min_freemapper = "0.13.0"
crs = 25832
names = {{ en = "Sample {id}", de = "Beispiel {id}" }}
last_verified = "2026-10-01"

[net]
hosts = ["geo.example.org"]

[boundary]
file = "boundary.geojson"
crs = 25833

[[osm.extract]]
geofabrik = "europe/germany/hamburg"
megabytes = 54
label = "Sample"

[attribution]
licence = "dl-de/by-2-0"
text = "© Sample {{year}}, dl-de/by-2-0"

[alkis]
kind = "wfs"
url = "https://geo.example.org/alkis/wfs"
buildings_type = "ave:GebaeudeBauwerk"
land_use_type = "ave:NutzungFlurstueck"
"#
        )
    }

    fn write(dir: &Path, id: &str, manifest: &str) {
        let folder = dir.join(id);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("manifest.toml"), manifest).unwrap();
        std::fs::write(folder.join("boundary.geojson"), SQUARE).unwrap();
    }

    fn ids(assembled: &Assembled) -> Vec<String> {
        assembled
            .registry
            .adapters()
            .map(|adapter| adapter.info().id.to_string())
            .collect()
    }

    #[test]
    fn adapter_dir_env_overrides_the_config_dir() {
        let config = Some(PathBuf::from("/home/u/.config"));
        assert_eq!(
            user_dir_from(None, config.clone()),
            Some(PathBuf::from("/home/u/.config/freemapper/adapters"))
        );
        assert_eq!(
            user_dir_from(Some("/srv/adapters".into()), config.clone()),
            Some(PathBuf::from("/srv/adapters"))
        );
        // An empty variable is no folder.
        assert_eq!(
            user_dir_from(Some(OsString::new()), config),
            Some(PathBuf::from("/home/u/.config/freemapper/adapters"))
        );
        assert_eq!(user_dir_from(None, None), None);
    }

    /// The adapters of the user's folder load always, those of a project only when its folder
    /// is given: `--adapters-from-project` gives it, the MCP server and the area page never do.
    #[test]
    fn project_adapters_need_the_flag() {
        let user = tempfile::tempdir().unwrap();
        write(user.path(), "useradapter", &manifest("useradapter"));
        let project = tempfile::tempdir().unwrap();
        let adapters = project_dir(project.path());
        write(&adapters, "projectadapter", &manifest("projectadapter"));
        let without = assemble(Some(user.path()), None);
        assert_eq!(without.registry.len(), 12);
        assert_eq!(
            ids(&without).last().map(String::as_str),
            Some("useradapter")
        );
        assert!(report_lines(&without).is_empty());
        let with = assemble(Some(user.path()), Some(&adapters));
        assert_eq!(ids(&with)[11..], ["useradapter", "projectadapter"]);
        assert_eq!(
            (
                source_of(&with, "hamburg"),
                source_of(&with, "useradapter"),
                source_of(&with, "projectadapter")
            ),
            ("built-in", "user", "project")
        );
        // The person is told which adapter of the project asks which hosts.
        assert_eq!(
            report_lines(&with),
            ["adapter projectadapter from the project folder asks these hosts: geo.example.org"]
        );
        // Neither folder: the eleven states.
        assert_eq!(assemble(None, None).registry.len(), 11);
    }

    #[test]
    fn a_broken_folder_is_a_warning_and_a_built_in_adapter_stays() {
        let user = tempfile::tempdir().unwrap();
        write(user.path(), "hamburg", &manifest("hamburg"));
        write(
            user.path(),
            "broken",
            &manifest("broken").replace("https://", "http://"),
        );
        let assembled = assemble(Some(user.path()), None);
        assert_eq!(assembled.registry.len(), 11);
        assert_eq!(
            assembled.registry.get("hamburg").unwrap().info().names.en,
            "Hamburg"
        );
        let lines = report_lines(&assembled);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(
            lines[0].starts_with("warning: the adapter in "),
            "{}",
            lines[0]
        );
        assert!(lines[0].contains("is left out: "), "{}", lines[0]);
        assert!(lines[0].contains("[policy.url]"), "{}", lines[0]);
        assert!(lines[1].contains("[adapter.duplicate]"), "{}", lines[1]);
    }

    #[test]
    fn the_list_names_source_revision_hosts_and_the_last_check() {
        let user = tempfile::tempdir().unwrap();
        write(user.path(), "useradapter", &manifest("useradapter"));
        let assembled = assemble(Some(user.path()), None);
        let lines = list_lines(&assembled, Some(user.path()));
        assert_eq!(lines.len(), 13);
        assert!(
            lines[4]
                .starts_with("hamburg  Hamburg  built in  revision 1  checked 2026-11-12  hosts: "),
            "{}",
            lines[4]
        );
        assert_eq!(
            lines[11],
            "useradapter  Sample useradapter  from your folder  revision 3  checked 2026-10-01  hosts: geo.example.org"
        );
        assert_eq!(
            lines[12],
            format!("Your adapters are read from {}", user.path().display())
        );
        let json: serde_json::Value =
            serde_json::from_str(&list_json(&assembled, Some(user.path()))).unwrap();
        assert_eq!(json["adapters"].as_array().unwrap().len(), 12);
        assert_eq!(json["adapters"][0]["id"], "brandenburg");
        assert_eq!(json["adapters"][0]["source"], "built-in");
        assert_eq!(json["adapters"][0]["provider"], "lgb");
        assert_eq!(json["adapters"][11]["source"], "user");
        assert_eq!(json["adapters"][11]["revision"], 3);
        assert_eq!(json["adapters"][11]["last_verified"], "2026-10-01");
        assert_eq!(json["adapters"][11]["hosts"][0], "geo.example.org");
        assert_eq!(json["adapters"][11]["names"]["de"], "Beispiel useradapter");
        assert_eq!(json["warnings"].as_array().unwrap().len(), 0);
        assert_eq!(json["user_dir"], user.path().display().to_string());
    }
}
