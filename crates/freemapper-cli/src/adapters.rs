// SPDX-License-Identifier: AGPL-3.0-or-later

//! The region adapters of a run: those built into the program, those of the user's folder and,
//! when the person asks for them, those of a project (ADR 0082, spec 3.7).
//!
//! The registry is assembled once, before anything asks it. An adapter from the person's folder
//! replaces a built-in one with its id (ADR 0188), one from a project folder does not; a folder
//! with a defect is left out with a warning. The MCP
//! server and the area page never load the adapters of a project: a project that was passed on
//! must not bring a source along.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use freemapper::adapters::{ExtraHosts, Registry, builtin, builtin_with_hosts, install_registry};
use freemapper_manifest::loader::{AdapterDir, AdapterWarning, Loaded, Origin, load_dirs};
use freemapper_manifest::model::Version;

use crate::text::{text, v};

/// The variable that names the folder of the user's adapters in place of the default.
pub const ADAPTER_DIR_VARIABLE: &str = "FREEMAPPER_ADAPTER_DIR";

/// The folder of the user's adapters.
#[derive(Debug, PartialEq, Eq)]
pub struct UserDir {
    /// The folder that is read, if the system has one.
    pub dir: Option<PathBuf>,
    /// What `FREEMAPPER_ADAPTER_DIR` names when that is no absolute path. It is not read.
    pub ignored: Option<PathBuf>,
}

impl UserDir {
    /// The line for the person about a folder that was named and is not read.
    fn warning(&self) -> Option<String> {
        let ignored = self.ignored.as_ref()?;
        Some(text(
            "cli-adapter-dir-relative",
            &[("path", v(ignored.display()))],
        ))
    }
}

/// The folder of the user's adapters: the one `FREEMAPPER_ADAPTER_DIR` names, else
/// `freemapper/adapters` in the configuration folder of the system.
pub fn user_dir() -> UserDir {
    user_dir_from(std::env::var_os(ADAPTER_DIR_VARIABLE), dirs::config_dir())
}

/// A relative path in the variable is not used: it would name another folder in every working
/// folder, and in a project that was passed on its folder `adapters`, which loads only when
/// the person asks for it.
fn user_dir_from(variable: Option<OsString>, config: Option<PathBuf>) -> UserDir {
    let default = || config.map(|config| config.join("freemapper").join("adapters"));
    match variable
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        Some(dir) if dir.is_absolute() => UserDir {
            dir: Some(dir),
            ignored: None,
        },
        Some(relative) => UserDir {
            dir: default(),
            ignored: Some(relative),
        },
        None => UserDir {
            dir: default(),
            ignored: None,
        },
    }
}

/// The folder of the adapters of the project `project`.
pub fn project_dir(project: &Path) -> PathBuf {
    project.join("adapters")
}

/// The version of this freemapper, which a manifest may ask for as its oldest. A build before
/// a release (`x.y.z-rc.1`) counts as that release.
pub(crate) fn running() -> Version {
    Version::of_program(env!("CARGO_PKG_VERSION")).expect("the version of the package is x.y.z")
}

/// The adapters of a run and what the folders had to say.
pub struct Assembled {
    pub registry: Registry,
    /// The adapters that came from a folder, in the order they were loaded.
    pub loaded: Vec<Loaded>,
    pub warnings: Vec<AdapterWarning>,
    /// The hosts the person allowed for built-in adapters, by adapter.
    pub allowed: ExtraHosts,
}

/// The built-in adapters, then those of `user`, then those of `project` (the folder
/// `adapters` of a project folder), each only if given.
pub fn assemble(user: Option<&Path>, project: Option<&Path>) -> Assembled {
    assemble_with_hosts(user, project, &ExtraHosts::new())
}

/// [`assemble`] with the hosts in `extra` allowed for the built-in adapters they name: what
/// the person allowed with `adapters allow-host` (ADR 0110).
pub fn assemble_with_hosts(
    user: Option<&Path>,
    project: Option<&Path>,
    extra: &ExtraHosts,
) -> Assembled {
    let mut registry = builtin_with_hosts(extra);
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
        allowed: extra.clone(),
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
/// with the hosts it may ask and the adapters on whose ground it lies.
fn report_lines(assembled: &Assembled) -> Vec<String> {
    let mut lines = warning_lines(assembled);
    for adapter in &assembled.loaded {
        if adapter.origin != Origin::Project {
            continue;
        }
        lines.push(text(
            "cli-adapter-from-project",
            &[
                ("adapter", v(&adapter.id)),
                ("hosts", v(adapter.hosts.join(", "))),
            ],
        ));
        if !adapter.overlaps.is_empty() {
            let others = region_names(assembled, &adapter.overlaps, &crate::text::lang().code());
            lines.push(text(
                "cli-adapter-overlaps",
                &[
                    ("adapter", v(&adapter.id)),
                    ("others", v(others.join(", "))),
                ],
            ));
        }
    }
    lines
}

/// Makes the adapters of this run the registry of the process: the built-in ones, those of the
/// user's folder and, with `project` (a project folder), those of its folder `adapters`. What
/// the folders had to say goes to stderr and comes back, for the MCP server to pass on. Called
/// once, before anything asks the registry.
///
/// The fetcher of the run is the plain one. Every adapter asks through its own policy
/// (`RegionAdapter::policy`), so an adapter from a folder changes nothing for the others.
pub fn install(project: Option<&Path>, may_write: bool) -> Vec<String> {
    let user = user_dir();
    let project = project.map(project_dir);
    let allowed = allowed_hosts(may_write);
    let assembled = assemble_with_hosts(user.dir.as_deref(), project.as_deref(), &allowed.hosts);
    let lines: Vec<String> = user
        .warning()
        .into_iter()
        .chain(allowed.lines)
        .chain(report_lines(&assembled))
        .collect();
    for line in &lines {
        eprintln!("{line}");
    }
    let installed = install_registry(assembled.registry);
    // Something that asked the registry before this call would have fixed the built-in
    // adapters, and the folders above would go unused without a word.
    debug_assert!(
        installed.is_ok(),
        "the registry was asked before the adapters of the run were installed"
    );
    lines
}

/// The hosts the person allowed for built-in adapters, and what there is to say about them.
struct AllowedHosts {
    hosts: ExtraHosts,
    lines: Vec<String>,
}

/// Reads the hosts the person allowed (`hosts::file`). With `may_write`, an entry for a host
/// its adapter names itself by now is taken out of the file and said once; the MCP server and
/// the area page read the file and never write it.
fn allowed_hosts(may_write: bool) -> AllowedHosts {
    let built_in = builtin();
    let file = crate::hosts::file();
    let allowed = crate::hosts::load(file.as_deref(), &built_in);
    let mut lines = allowed.warnings.clone();
    if let (true, Some(file)) = (may_write, &file) {
        lines.extend(crate::hosts::drop_redundant(file, &allowed, &built_in));
    }
    AllowedHosts {
        hosts: allowed.hosts,
        lines,
    }
}

/// `freemapper adapters allow-host` and `disallow-host`: allows the built-in adapter `adapter`
/// the host `host`, or takes it back. The line that says what happened goes to stdout; why
/// nothing changed is the error.
pub fn change_allowed_host(adapter: &str, host: &str, allow: bool) -> anyhow::Result<()> {
    let built_in = builtin();
    // With the person's folder, to tell an adapter of theirs from a name nobody has.
    let with_folders = assemble(user_dir().dir.as_deref(), None).registry;
    let file = crate::hosts::file();
    let change = if allow {
        crate::hosts::allow
    } else {
        crate::hosts::disallow
    };
    let line = change(file.as_deref(), adapter, host, &built_in, &with_folders)
        .map_err(anyhow::Error::msg)?;
    println!("{line}");
    Ok(())
}

/// The line about `project` (a project folder given to `adapters list`) when it is no folder.
fn missing_project(project: Option<&Path>) -> Option<String> {
    let project = project.filter(|project| !project.is_dir())?;
    Some(text(
        "cli-adapters-no-project",
        &[("path", v(project.display()))],
    ))
}

/// `freemapper adapters list`: every adapter with where it comes from, its revision, the day
/// it was last checked and the hosts it asks. `project` also reads the adapters of that
/// project folder, as `--adapters-from-project` would.
pub fn list(json: bool, project: Option<&Path>) {
    let user = user_dir();
    for line in user.warning().into_iter().chain(missing_project(project)) {
        eprintln!("{line}");
    }
    let project = project.map(project_dir);
    // The list shows what a run would use and changes nothing: it does not write the file.
    let allowed = allowed_hosts(false);
    let mut assembled =
        assemble_with_hosts(user.dir.as_deref(), project.as_deref(), &allowed.hosts);
    assembled.allowed = allowed.hosts;
    if json {
        println!("{}", list_json(&assembled, user.dir.as_deref()));
        return;
    }
    for line in allowed.lines.into_iter().chain(warning_lines(&assembled)) {
        eprintln!("{line}");
    }
    for line in list_lines(&assembled, user.dir.as_deref(), &crate::text::lang().code()) {
        println!("{line}");
    }
}

/// `freemapper adapters export <id> <dir>`: writes the built-in adapter `id` as the folder
/// `<dir>/<id>`, which loads as an adapter that replaces the built-in one (ADR 0188). It never
/// overwrites: the folder must not exist or must be empty.
pub fn export(id: &str, dir: &Path) -> anyhow::Result<()> {
    let files = freemapper::adapters::export_files(id).ok_or_else(|| {
        anyhow::anyhow!(text("cli-adapters-export-unknown", &[("adapter", v(id))]))
    })?;
    let target = dir.join(id);
    write_export(&target, &files)?;
    println!(
        "{}",
        text(
            "cli-adapters-export-done",
            &[
                ("adapter", v(id)),
                ("count", v(files.len())),
                ("path", v(target.display())),
            ]
        )
    );
    let user = user_dir();
    if let Some(user) = user.dir {
        println!(
            "{}",
            text("cli-adapters-export-next", &[("path", v(user.display()))])
        );
    }
    Ok(())
}

/// Writes `files` into `target`, a folder that does not exist yet or is empty.
fn write_export(
    target: &Path,
    files: &[(String, std::borrow::Cow<'static, str>)],
) -> anyhow::Result<()> {
    use std::io::Write as _;
    let taken = |_| {
        anyhow::anyhow!(text(
            "cli-adapters-export-taken",
            &[("path", v(target.display()))]
        ))
    };
    match std::fs::read_dir(target) {
        Ok(mut entries) => {
            if entries.next().is_some() {
                return Err(taken(()));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(target)?;
        }
        Err(_) => return Err(taken(())),
    }
    for (name, body) in files {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target.join(name))?;
        file.write_all(body.as_bytes())?;
    }
    Ok(())
}

/// The hosts the person allowed for the adapter `id`.
fn allowed_for<'a>(assembled: &'a Assembled, id: &str) -> &'a [String] {
    assembled.allowed.get(id).map_or(&[], Vec::as_slice)
}

/// The regions of the adapters `ids` by their names, as a person knows them; German with
/// `german`. An id the registry does not hold stays as it is.
/// The names of a region for the JSON listings: `pl` and `cs` appear only when the region has one, so the
/// keys that existed before stay as they were.
pub(crate) fn names_json(names: &freemapper::adapters::Names) -> serde_json::Value {
    let mut json = serde_json::json!({ "en": names.en, "de": names.de });
    if let Some(pl) = &names.pl {
        json["pl"] = serde_json::Value::from(pl.as_str());
    }
    if let Some(cs) = &names.cs {
        json["cs"] = serde_json::Value::from(cs.as_str());
    }
    json
}

fn region_names<'a>(assembled: &'a Assembled, ids: &'a [String], lang: &str) -> Vec<&'a str> {
    ids.iter()
        .map(|id| match assembled.registry.get(id) {
            Some(other) => other.info().names.in_lang(lang),
            None => id.as_str(),
        })
        .collect()
}

/// The adapters on whose ground the adapter `id` lies, if it came from a folder.
fn overlaps_of<'a>(assembled: &'a Assembled, id: &str) -> &'a [String] {
    assembled
        .loaded
        .iter()
        .find(|loaded| loaded.id == id)
        .map_or(&[], |loaded| &loaded.overlaps)
}

/// Whether the adapter `id` is one from a folder that took the place of a built-in one.
fn replaces_built_in(assembled: &Assembled, id: &str) -> bool {
    assembled
        .loaded
        .iter()
        .any(|loaded| loaded.id == id && loaded.replaces_built_in)
}

/// Where the adapter `id` comes from, as the JSON says it.
fn source_of(assembled: &Assembled, id: &str) -> &'static str {
    match assembled.loaded.iter().find(|loaded| loaded.id == id) {
        Some(loaded) if loaded.origin == Origin::Project => "project",
        Some(_) => "user",
        None => "built-in",
    }
}

/// The lines of the list; the names of the regions are German with `german`.
fn list_lines(assembled: &Assembled, user: Option<&Path>, lang: &str) -> Vec<String> {
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
            let name = info.names.in_lang(lang);
            let line = text(
                "cli-adapters-line",
                &[
                    ("adapter", v(&info.id)),
                    ("name", v(name)),
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
            );
            let line = match allowed_for(assembled, info.id.as_str()) {
                [] => line,
                hosts => {
                    let allowed = text("cli-adapters-allowed", &[("hosts", v(hosts.join(", ")))]);
                    format!("{line}  {allowed}")
                }
            };
            match overlaps_of(assembled, info.id.as_str()) {
                [] => line,
                others => {
                    let others = region_names(assembled, others, lang);
                    let others = text("cli-adapters-overlaps", &[("others", v(others.join(", ")))]);
                    format!("{line}  {others}")
                }
            }
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
                "names": names_json(&info.names),
                "provider": info.provider.as_str(),
                "source": source_of(assembled, info.id.as_str()),
                "revision": declared.map(|declared| declared.revision),
                "last_verified": declared
                    .and_then(|declared| declared.last_verified)
                    .map(|day| day.to_string()),
                "hosts": declared.map(|declared| declared.hosts.clone()).unwrap_or_default(),
                "allowed_hosts": allowed_for(assembled, info.id.as_str()),
                "overlaps": overlaps_of(assembled, info.id.as_str()),
                "replaces_built_in": replaces_built_in(assembled, info.id.as_str()),
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

    #[test]
    fn the_polish_name_is_listed_only_where_there_is_one() {
        let mut names = freemapper::adapters::Names {
            en: "Poland".into(),
            de: "Polen".into(),
            pl: None,
            cs: None,
        };
        assert_eq!(
            names_json(&names),
            serde_json::json!({"en": "Poland", "de": "Polen"})
        );
        names.pl = Some("Polska".into());
        assert_eq!(
            names_json(&names),
            serde_json::json!({"en": "Poland", "de": "Polen", "pl": "Polska"})
        );
    }

    #[test]
    fn the_list_names_a_region_in_polish() {
        let assembled = assemble(None, None);
        let lines = list_lines(&assembled, None, "pl");
        assert!(
            lines.iter().any(|l| l.starts_with("polska  Polska  ")),
            "{lines:?}"
        );
    }

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
        let root = if cfg!(windows) { "C:\\" } else { "/" };
        let config = PathBuf::from(root).join("home").join("config");
        let default = config.join("freemapper").join("adapters");
        let used = |dir: PathBuf| UserDir {
            dir: Some(dir),
            ignored: None,
        };
        assert_eq!(
            user_dir_from(None, Some(config.clone())),
            used(default.clone())
        );
        let named = PathBuf::from(root).join("srv").join("adapters");
        assert_eq!(
            user_dir_from(Some(named.clone().into()), Some(config.clone())),
            used(named)
        );
        // An empty variable is no folder.
        assert_eq!(
            user_dir_from(Some(OsString::new()), Some(config.clone())),
            used(default.clone())
        );
        assert_eq!(
            user_dir_from(None, None),
            UserDir {
                dir: None,
                ignored: None
            }
        );
        // A relative folder would be another one in every working folder, the `adapters` of
        // a project that was passed on among them: it is not read, and the person is told.
        let relative = user_dir_from(Some("adapters".into()), Some(config));
        assert_eq!(
            relative,
            UserDir {
                dir: Some(default),
                ignored: Some(PathBuf::from("adapters"))
            }
        );
        assert_eq!(
            relative.warning().unwrap(),
            "warning: FREEMAPPER_ADAPTER_DIR is not an absolute path (adapters); it is ignored"
        );
    }

    /// Every adapter is held to the hosts of its own manifest. An adapter from a folder adds
    /// no host to a state of the program, and a state adds none to it.
    #[test]
    fn every_adapter_is_held_to_its_own_hosts() {
        let user = tempfile::tempdir().unwrap();
        write(user.path(), "useradapter", &manifest("useradapter"));
        let assembled = assemble(Some(user.path()), None);
        assert!(
            assembled
                .registry
                .adapters()
                .all(|adapter| adapter.policy().is_some())
        );
        let policy = |id: &str| assembled.registry.get(id).unwrap().policy().unwrap();
        let theirs = "https://geo.example.org/alkis/wfs";
        let lgb = "https://isk.geobasis-bb.de/x";
        assert!(policy("useradapter").check_url(theirs).is_ok());
        assert!(policy("useradapter").check_url(lgb).is_err());
        assert!(policy("brandenburg").check_url(lgb).is_ok());
        assert!(policy("brandenburg").check_url(theirs).is_err());
        // The hosts of the program's own requests are no adapter's to ask.
        for id in ["useradapter", "brandenburg"] {
            assert!(
                policy(id)
                    .check_url("https://download.geofabrik.de/x")
                    .is_err()
            );
        }
        // Without the folder a state has the same list.
        let alone = assemble(None, None);
        assert_eq!(
            alone.registry.get("brandenburg").unwrap().policy(),
            Some(policy("brandenburg"))
        );
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
        assert_eq!(without.registry.len(), 18);
        assert_eq!(
            ids(&without).last().map(String::as_str),
            Some("useradapter")
        );
        assert!(report_lines(&without).is_empty());
        let with = assemble(Some(user.path()), Some(&adapters));
        assert_eq!(ids(&with)[17..], ["useradapter", "projectadapter"]);
        assert_eq!(
            (
                source_of(&with, "hamburg"),
                source_of(&with, "useradapter"),
                source_of(&with, "projectadapter")
            ),
            ("built-in", "user", "project")
        );
        // The person is told which adapter of the project may ask which hosts, and on whose
        // ground it lies: the square of the samples is in Brandenburg.
        assert_eq!(
            report_lines(&with),
            [
                "warning: the adapter projectadapter from the project folder is in use; it may \
                 ask these hosts: geo.example.org",
                "warning: the adapter projectadapter also covers ground of Brandenburg, \
                 Sample useradapter; its objects go onto maps there"
            ]
        );
        // Neither folder: the fifteen states and Poland.
        assert_eq!(assemble(None, None).registry.len(), 17);
    }

    #[test]
    fn a_folder_with_the_id_of_a_built_in_state_replaces_it_and_a_broken_one_is_left_out() {
        let user = tempfile::tempdir().unwrap();
        write(user.path(), "hamburg", &manifest("hamburg"));
        write(
            user.path(),
            "broken",
            &manifest("broken").replace("https://", "http://"),
        );
        let assembled = assemble(Some(user.path()), None);
        // The state keeps its place and its count: the folder took its place.
        assert_eq!(assembled.registry.len(), 17);
        assert_eq!(
            assembled.registry.get("hamburg").unwrap().info().names.en,
            "Sample hamburg"
        );
        assert_eq!(source_of(&assembled, "hamburg"), "user");
        assert!(replaces_built_in(&assembled, "hamburg"));
        assert!(!replaces_built_in(&assembled, "berlin"));
        let lines = report_lines(&assembled);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines[0].contains("is left out: "), "{}", lines[0]);
        assert!(lines[0].contains("[policy.url]"), "{}", lines[0]);
        assert!(
            lines[1].contains("[adapter.replaces-built-in]"),
            "{}",
            lines[1]
        );
        let json: serde_json::Value =
            serde_json::from_str(&list_json(&assembled, Some(user.path()))).unwrap();
        let hamburg = json["adapters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == "hamburg")
            .unwrap();
        assert_eq!(hamburg["replaces_built_in"], true);
        assert_eq!(hamburg["source"], "user");
    }

    /// A built-in state is replaced once: a folder of the project with the same id does not
    /// replace the one of the person's folder.
    #[test]
    fn a_state_is_replaced_once() {
        let user = tempfile::tempdir().unwrap();
        write(user.path(), "hamburg", &manifest("hamburg"));
        let project = tempfile::tempdir().unwrap();
        let adapters = project_dir(project.path());
        write(&adapters, "hamburg", &manifest("hamburg"));
        let assembled = assemble(Some(user.path()), Some(&adapters));
        assert_eq!(assembled.registry.len(), 17);
        let lines = report_lines(&assembled);
        assert!(
            lines.iter().any(|l| l.contains("[adapter.duplicate]")),
            "{lines:?}"
        );
    }

    /// A host that is an address, a wildcard or this computer keeps the adapter out, in the
    /// folder of a project as anywhere; the others load.
    #[test]
    fn a_project_adapter_with_a_host_that_is_no_public_name_is_left_out() {
        let project = tempfile::tempdir().unwrap();
        let adapters = project_dir(project.path());
        for (id, host) in [
            ("address", "127.0.0.1"),
            ("wildcard", "*"),
            ("thiscomputer", "localhost"),
            ("subdomains", "*.example.org"),
        ] {
            let text = manifest(id).replace(
                "hosts = [\"geo.example.org\"]",
                &format!("hosts = [\"geo.example.org\", \"{host}\"]"),
            );
            write(&adapters, id, &text);
        }
        write(&adapters, "fine", &manifest("fine"));
        let assembled = assemble(None, Some(&adapters));
        assert_eq!(ids(&assembled)[17..], ["fine"]);
        let lines = warning_lines(&assembled);
        assert_eq!(lines.len(), 4, "{lines:?}");
        for line in &lines {
            assert!(line.starts_with("warning: the adapter in "), "{line}");
            assert!(line.contains(" is left out: "), "{line}");
            assert!(line.contains("[policy.host]"), "{line}");
        }
    }

    /// `adapters list --project` with a folder that is not there says so.
    #[test]
    fn a_project_folder_that_is_not_there_is_named() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(missing_project(Some(dir.path())), None);
        assert_eq!(missing_project(None), None);
        let typo = dir.path().join("typo");
        assert_eq!(
            missing_project(Some(&typo)).unwrap(),
            format!(
                "warning: {} is not a folder; no adapters of a project were read",
                typo.display()
            )
        );
    }

    #[test]
    fn the_list_names_source_revision_hosts_and_the_last_check() {
        let user = tempfile::tempdir().unwrap();
        write(user.path(), "useradapter", &manifest("useradapter"));
        write(user.path(), "zweiter", &manifest("zweiter"));
        let assembled = assemble(Some(user.path()), None);
        let lines = list_lines(&assembled, Some(user.path()), "en");
        assert_eq!(lines.len(), 20);
        assert!(
            lines[4]
                .starts_with("hamburg  Hamburg  built in  revision 2  checked 2026-11-15  hosts: "),
            "{}",
            lines[4]
        );
        assert_eq!(
            lines[17],
            "useradapter  Sample useradapter  from your folder  revision 3  checked 2026-10-01  hosts: geo.example.org  shares ground with: Brandenburg"
        );
        // An adapter on the ground of another names it.
        assert_eq!(
            lines[18],
            "zweiter  Sample zweiter  from your folder  revision 3  checked 2026-10-01  hosts: geo.example.org  shares ground with: Brandenburg, Sample useradapter"
        );
        assert_eq!(
            lines[19],
            format!("Your adapters are read from {}", user.path().display())
        );
        // The German list has the German names.
        let german = list_lines(&assembled, Some(user.path()), "de");
        assert!(german[2].starts_with("saxony  Sachsen  "), "{}", german[2]);
        assert!(
            german[17].starts_with("useradapter  Beispiel useradapter  "),
            "{}",
            german[17]
        );
        assert!(
            german[18].ends_with(": Brandenburg, Beispiel useradapter"),
            "{}",
            german[18]
        );
        let json: serde_json::Value =
            serde_json::from_str(&list_json(&assembled, Some(user.path()))).unwrap();
        assert_eq!(json["adapters"].as_array().unwrap().len(), 19);
        assert_eq!(
            json["adapters"][17]["overlaps"],
            serde_json::json!(["brandenburg"])
        );
        assert_eq!(
            json["adapters"][18]["overlaps"],
            serde_json::json!(["brandenburg", "useradapter"])
        );
        assert_eq!(json["adapters"][0]["overlaps"], serde_json::json!([]));
        assert_eq!(json["adapters"][0]["id"], "brandenburg");
        assert_eq!(json["adapters"][0]["source"], "built-in");
        assert_eq!(json["adapters"][0]["provider"], "lgb");
        assert_eq!(json["adapters"][17]["source"], "user");
        assert_eq!(json["adapters"][17]["revision"], 3);
        assert_eq!(json["adapters"][17]["last_verified"], "2026-10-01");
        assert_eq!(json["adapters"][17]["hosts"][0], "geo.example.org");
        assert_eq!(json["adapters"][17]["names"]["de"], "Beispiel useradapter");
        assert_eq!(json["warnings"].as_array().unwrap().len(), 0);
        assert_eq!(json["user_dir"], user.path().display().to_string());
    }
}
