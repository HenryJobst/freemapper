//! The hosts a person allowed for built-in adapters besides those the adapters name (ADR 0110).
//!
//! A built-in state asks the hosts of its manifest and no other. When an agency moves a service
//! to a new server, maps of that state would fail until a release names the server. A person can
//! allow the state that one host in the meantime: `freemapper adapters allow-host <adapter>
//! <host>`. The hosts live in one file of the person's configuration folder and nowhere else:
//! no project folder, no variable of freemapper, no switch of a run and no request of the MCP
//! server or the area page adds one. A host is checked like a host of a manifest, and the check
//! of the address a connection goes to holds for it as for every host.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use freemapper::adapters::{ExtraHosts, Registry};
use freemapper::fetch::dns_name;
use serde::{Deserialize, Serialize};

use crate::text::{text, v};

/// The name of the file in `<config>/freemapper/`.
pub const FILE_NAME: &str = "allowed-hosts.toml";

/// The largest file read, in bytes.
const MAX_BYTES: u64 = 64 * 1024;

const HEADER: &str = "# Hosts freemapper may ask for a built-in state besides those its adapter names.\n\
                      # Change this file with `freemapper adapters allow-host` and `disallow-host`.\n\n";

/// The file with the allowed hosts: `allowed-hosts.toml` in `freemapper` of the configuration
/// folder of the system. `None`: the system has no such folder.
pub fn file() -> Option<PathBuf> {
    dirs::config_dir().map(|config| config.join("freemapper").join(FILE_NAME))
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stored {
    /// The hosts by the id of the adapter.
    #[serde(default)]
    allow: BTreeMap<String, Vec<String>>,
}

/// What the file says, checked against the adapters of the program.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Allowed {
    /// The hosts that hold: a built-in adapter, a host that is a public DNS name and that the
    /// adapter does not name itself.
    pub hosts: ExtraHosts,
    /// Entries for a host the adapter names itself by now: `(adapter, host)`.
    pub redundant: Vec<(String, String)>,
    /// What was left out, each a line for the person.
    pub warnings: Vec<String>,
}

/// The hosts the adapter `id` of `built_in` names itself.
fn declared<'a>(built_in: &'a Registry, id: &str) -> Option<&'a [String]> {
    let adapter = built_in.get(id)?;
    Some(adapter.declared().map_or(&[], |declared| &declared.hosts))
}

/// What the file `path` holds, as it is written: nothing when the file is not there, and why
/// when it cannot be read.
fn read(path: &Path) -> Result<Stored, String> {
    if !path.exists() {
        return Ok(Stored::default());
    }
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MAX_BYTES {
        return Err(format!("{size} bytes; at most {MAX_BYTES}"));
    }
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    toml::from_str(&text).map_err(|e| e.message().to_string())
}

/// Reads the file `path` against the adapters `built_in` of the program. A file that is not
/// there allows nothing; one that cannot be read allows nothing and is a warning.
pub fn load(path: Option<&Path>, built_in: &Registry) -> Allowed {
    let mut allowed = Allowed::default();
    let Some(path) = path.filter(|path| path.exists()) else {
        return allowed;
    };
    let stored = match read(path) {
        Ok(stored) => stored,
        Err(detail) => {
            allowed.warnings.push(text(
                "cli-allowed-hosts-unreadable",
                &[("file", v(path.display())), ("detail", v(detail))],
            ));
            return allowed;
        }
    };
    let lang = crate::text::lang();
    for (adapter, hosts) in stored.allow {
        let mut ignore = |host: &str, reason: String| {
            allowed.warnings.push(text(
                "cli-allowed-hosts-ignored",
                &[
                    ("file", v(path.display())),
                    ("adapter", v(&adapter)),
                    ("host", v(host)),
                    ("reason", v(reason)),
                ],
            ));
        };
        let Some(named) = declared(built_in, &adapter) else {
            for host in &hosts {
                ignore(host, text("cli-allowed-hosts-no-adapter", &[]));
            }
            continue;
        };
        for host in &hosts {
            match dns_name(host) {
                Err(refused) => ignore(
                    host,
                    crate::errors::not_allowed_reason(&lang, refused.reason),
                ),
                Ok(host) if named.contains(&host) => {
                    allowed.redundant.push((adapter.clone(), host));
                }
                Ok(host) => {
                    let list = allowed.hosts.entry(adapter.clone()).or_default();
                    if !list.contains(&host) {
                        list.push(host);
                    }
                }
            }
        }
    }
    allowed
}

/// Writes `stored` as the file `path`, with everything in it, also what this version leaves
/// out when it reads; without an entry the file is removed. A file that is a link is written
/// where the link points.
fn save(path: &Path, mut stored: Stored) -> Result<(), String> {
    stored.allow.retain(|_, hosts| !hosts.is_empty());
    // The file itself, not the link to it: a link stays a link.
    let target = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if stored.allow.is_empty() {
        return match std::fs::remove_file(&target) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
            _ => Ok(()),
        };
    }
    let body = toml::to_string(&stored).map_err(|e| e.to_string())?;
    let folder = target.parent().ok_or("no folder")?;
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    // Whole or not at all: written beside the file under a name of this process, then put
    // in its place.
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.subsec_nanos());
    let part = folder.join(format!(".{FILE_NAME}.{}-{stamp}.part", std::process::id()));
    std::fs::write(&part, format!("{HEADER}{body}"))
        .and_then(|()| std::fs::rename(&part, &target))
        .map_err(|e| {
            let _ = std::fs::remove_file(&part);
            e.to_string()
        })
}

/// Whether the entry `written` of the file is the host `host`: the same text, or the same
/// name in another case.
fn same_host(written: &str, host: &str) -> bool {
    written == host || dns_name(written).is_ok_and(|name| name == host)
}

/// Takes the redundant entries out of the file `path`: the hosts an adapter names itself by
/// now. Everything else in the file stays. The lines that say so, one per entry; none when
/// the file could not be read or written.
pub fn drop_redundant(path: &Path, allowed: &Allowed, built_in: &Registry) -> Vec<String> {
    if allowed.redundant.is_empty() {
        return Vec::new();
    }
    let Ok(mut stored) = read(path) else {
        return Vec::new();
    };
    for (adapter, host) in &allowed.redundant {
        if let Some(hosts) = stored.allow.get_mut(adapter) {
            hosts.retain(|written| !same_host(written, host));
        }
    }
    if save(path, stored).is_err() {
        return Vec::new();
    }
    allowed
        .redundant
        .iter()
        .map(|(adapter, host)| {
            text(
                "cli-allowed-host-redundant",
                &[
                    ("host", v(host)),
                    ("name", v(name_of(built_in, adapter))),
                    ("file", v(path.display())),
                ],
            )
        })
        .collect()
}

/// The file `path` as it is written, for a command that changes it. A file that cannot be
/// read is an error: the command must not write over what it could not read.
fn read_to_change(path: &Path) -> Result<Stored, String> {
    read(path).map_err(|detail| {
        text(
            "cli-allow-host-unreadable",
            &[("file", v(path.display())), ("detail", v(detail))],
        )
    })
}

fn write_changed(path: &Path, stored: Stored) -> Result<(), String> {
    save(path, stored).map_err(|detail| {
        text(
            "cli-allow-host-write",
            &[("file", v(path.display())), ("detail", v(detail))],
        )
    })
}

/// The name of the region of the adapter `id` in the language of the run.
fn name_of(built_in: &Registry, id: &str) -> String {
    let german = crate::text::lang().code() == "de";
    built_in.get(id).map_or_else(
        || id.to_string(),
        |adapter| {
            let names = &adapter.info().names;
            if german { &names.de } else { &names.en }.clone()
        },
    )
}

/// The built-in adapter `adapter` and the host `host` as a host of a manifest is written, or
/// why the two cannot be an entry. `with_folders` holds the adapters of the person's folder as
/// well, to tell an adapter of theirs from a name nobody has.
fn entry(
    adapter: &str,
    host: &str,
    built_in: &Registry,
    with_folders: &Registry,
) -> Result<String, String> {
    if built_in.get(adapter).is_none() {
        return Err(if with_folders.get(adapter).is_some() {
            text("cli-allow-host-folder-adapter", &[("adapter", v(adapter))])
        } else {
            let known: Vec<&str> = built_in
                .adapters()
                .map(|adapter| adapter.info().id.as_str())
                .collect();
            text(
                "cli-allow-host-unknown-adapter",
                &[("adapter", v(adapter)), ("adapters", v(known.join(", ")))],
            )
        });
    }
    dns_name(host).map_err(|refused| {
        text(
            "cli-allow-host-refused",
            &[
                ("host", v(host)),
                (
                    "reason",
                    v(crate::errors::not_allowed_reason(
                        &crate::text::lang(),
                        refused.reason,
                    )),
                ),
            ],
        )
    })
}

/// `freemapper adapters allow-host`: allows the built-in adapter `adapter` the host `host` in
/// the file `path`. The line that says what happened, or why nothing was allowed.
pub fn allow(
    path: Option<&Path>,
    adapter: &str,
    host: &str,
    built_in: &Registry,
    with_folders: &Registry,
) -> Result<String, String> {
    let host = entry(adapter, host, built_in, with_folders)?;
    let name = name_of(built_in, adapter);
    let args = |file: &Path| {
        [
            ("host", v(&host)),
            ("name", v(&name)),
            ("adapter", v(adapter)),
            ("file", v(file.display())),
        ]
    };
    if declared(built_in, adapter).is_some_and(|named| named.contains(&host)) {
        return Ok(text("cli-allow-host-declared", &args(Path::new(""))));
    }
    let path = path.ok_or_else(|| text("cli-allow-host-no-config", &[]))?;
    let mut stored = read_to_change(path)?;
    let hosts = stored.allow.entry(adapter.to_string()).or_default();
    if hosts.iter().any(|written| same_host(written, &host)) {
        return Ok(text("cli-allow-host-already", &args(path)));
    }
    hosts.push(host.clone());
    write_changed(path, stored)?;
    Ok(text("cli-allow-host-added", &args(path)))
}

/// `freemapper adapters disallow-host`: takes the host `host` back that was allowed for the
/// adapter `adapter` in the file `path`. An entry this version leaves out when it reads (an
/// adapter it does not know, a host that fails the check) is taken back by its text.
pub fn disallow(
    path: Option<&Path>,
    adapter: &str,
    host: &str,
    built_in: &Registry,
    with_folders: &Registry,
) -> Result<String, String> {
    let checked = entry(adapter, host, built_in, with_folders);
    let name = name_of(built_in, adapter);
    let host = checked.as_deref().unwrap_or(host);
    let args = [("host", v(host)), ("name", v(&name))];
    let mut stored = match path {
        Some(path) => read_to_change(path)?,
        None => Stored::default(),
    };
    let removed = stored.allow.get_mut(adapter).is_some_and(|hosts| {
        let before = hosts.len();
        hosts.retain(|written| !same_host(written, host));
        hosts.len() < before
    });
    match (removed, path) {
        (true, Some(path)) => {
            write_changed(path, stored)?;
            Ok(text("cli-disallow-host-removed", &args))
        }
        // Nothing of that name in the file: say why the two cannot be an entry, if so.
        _ => checked.map(|_| text("cli-disallow-host-absent", &args)),
    }
}

#[cfg(test)]
mod tests {
    use freemapper::adapters::{builtin, builtin_with_hosts};

    use super::*;

    fn file_in(dir: &Path) -> PathBuf {
        dir.join("freemapper").join(FILE_NAME)
    }

    /// A host that was allowed goes through for its state and for no other, and taking it
    /// back closes it again.
    #[test]
    fn a_host_that_was_allowed_goes_through_for_its_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = file_in(dir.path());
        let built_in = builtin();
        let line = allow(
            Some(&path),
            "hessen",
            "CDN.Example.net",
            &built_in,
            &built_in,
        )
        .unwrap();
        assert_eq!(
            line,
            format!(
                "freemapper may now also ask cdn.example.net for Hesse. The entry is in {}; \
                 `freemapper adapters disallow-host hessen cdn.example.net` takes it back. An area \
                 page or an AI assistant that is running reads the entry when it is started again",
                path.display()
            )
        );
        let allowed = load(Some(&path), &built_in);
        assert_eq!(allowed.hosts["hessen"], ["cdn.example.net"]);
        assert!(allowed.warnings.is_empty() && allowed.redundant.is_empty());
        let registry = builtin_with_hosts(&allowed.hosts);
        let policy = |id: &str| registry.get(id).unwrap().policy().unwrap();
        assert!(
            policy("hessen")
                .check_url("https://cdn.example.net/x")
                .is_ok()
        );
        assert!(
            policy("hamburg")
                .check_url("https://cdn.example.net/x")
                .is_err()
        );
        // The file says what it is.
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("# Hosts freemapper may ask"), "{text}");
        assert!(
            text.contains("[allow]\nhessen = [\"cdn.example.net\"]"),
            "{text}"
        );
        // A second time nothing changes; taking it back removes the file with its last host.
        let again = allow(
            Some(&path),
            "hessen",
            "cdn.example.net",
            &built_in,
            &built_in,
        );
        assert_eq!(
            again.unwrap(),
            "cdn.example.net is already allowed for Hesse"
        );
        let taken = disallow(
            Some(&path),
            "hessen",
            "cdn.example.net",
            &built_in,
            &built_in,
        );
        assert_eq!(
            taken.unwrap(),
            "freemapper no longer asks cdn.example.net for Hesse"
        );
        assert!(!path.exists());
        let nothing = disallow(
            Some(&path),
            "hessen",
            "cdn.example.net",
            &built_in,
            &built_in,
        );
        assert_eq!(
            nothing.unwrap(),
            "cdn.example.net was not allowed for Hesse; nothing to take back"
        );
    }

    /// What is no public DNS name cannot be allowed: an address, this computer, a name of the
    /// local network, a wildcard, user info, a port, a URL. Nothing is written.
    #[test]
    fn a_host_that_is_no_public_name_is_refused_when_it_is_entered() {
        let dir = tempfile::tempdir().unwrap();
        let path = file_in(dir.path());
        let built_in = builtin();
        for (host, reason) in [
            ("192.168.0.1", "an IP address, not a DNS name"),
            ("127.0.0.1", "an IP address, not a DNS name"),
            ("localhost", "a name of this computer or without a domain"),
            ("fritz.box", "a name of the local network"),
            ("printer.local", "a name of the local network"),
            ("*", "not a DNS name"),
            ("*.example.net", "not a DNS name"),
        ] {
            let refused = allow(Some(&path), "hessen", host, &built_in, &built_in).unwrap_err();
            assert_eq!(refused, format!("{host} cannot be allowed: {reason}"));
        }
        for host in [
            "user@cdn.example.net",
            "cdn.example.net:8443",
            "https://cdn.example.net/x",
            "",
        ] {
            assert!(
                allow(Some(&path), "hessen", host, &built_in, &built_in).is_err(),
                "{host}"
            );
        }
        assert!(!path.exists());
        // A host the adapter names itself needs no entry.
        let named = allow(Some(&path), "hessen", "gds.hessen.de", &built_in, &built_in);
        assert_eq!(
            named.unwrap(),
            "the adapter of Hesse names gds.hessen.de itself; nothing to allow"
        );
        assert!(!path.exists());
        // Only an adapter of the program takes a host this way.
        let unknown = allow(
            Some(&path),
            "atlantis",
            "cdn.example.net",
            &built_in,
            &built_in,
        );
        assert!(
            unknown
                .unwrap_err()
                .starts_with("there is no built-in adapter atlantis; the built-in adapters are: brandenburg, berlin, "),
        );
    }

    /// A file written by hand is read as strictly: an entry for an adapter the program does
    /// not have, or a host that is no public name, is left out with a warning, and the rest
    /// holds. A file that is not TOML allows nothing.
    #[test]
    fn a_file_written_by_hand_is_checked_like_an_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = file_in(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "[allow]\nhessen = [\"cdn.example.net\", \"10.0.0.1\", \"cdn.example.net\"]\n\
             atlantis = [\"cdn.example.net\"]\nhamburg = [\"localhost\"]\n",
        )
        .unwrap();
        let built_in = builtin();
        let allowed = load(Some(&path), &built_in);
        assert_eq!(allowed.hosts.len(), 1);
        assert_eq!(allowed.hosts["hessen"], ["cdn.example.net"]);
        assert_eq!(allowed.warnings.len(), 3, "{:?}", allowed.warnings);
        assert_eq!(
            allowed.warnings[0],
            format!(
                "warning: {}: the entry cdn.example.net for atlantis is ignored: freemapper \
                 has no built-in adapter of that name",
                path.display()
            )
        );
        assert!(
            allowed.warnings[2].ends_with(
                "the entry 10.0.0.1 for hessen is ignored: an IP address, not a DNS name"
            ),
            "{}",
            allowed.warnings[2]
        );
        std::fs::write(&path, "allow = nonsense").unwrap();
        let broken = load(Some(&path), &built_in);
        assert!(broken.hosts.is_empty());
        assert!(
            broken.warnings[0].contains("could not be read"),
            "{}",
            broken.warnings[0]
        );
        // No file, no folder: nothing is allowed and nothing is said.
        assert_eq!(
            load(Some(&dir.path().join("none")), &built_in),
            Allowed::default()
        );
        assert_eq!(load(None, &built_in), Allowed::default());
    }

    /// Once a release names the host itself, the entry is said to be redundant, taken out of
    /// the file and not mentioned again.
    #[test]
    fn an_entry_a_release_made_redundant_is_removed_and_said_once() {
        let dir = tempfile::tempdir().unwrap();
        let path = file_in(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "[allow]\nhessen = [\"gds.hessen.de\", \"cdn.example.net\"]\n",
        )
        .unwrap();
        let built_in = builtin();
        let allowed = load(Some(&path), &built_in);
        assert_eq!(allowed.hosts["hessen"], ["cdn.example.net"]);
        assert_eq!(
            allowed.redundant,
            [("hessen".to_string(), "gds.hessen.de".to_string())]
        );
        assert_eq!(
            drop_redundant(&path, &allowed, &built_in),
            [format!(
                "warning: gds.hessen.de is now one of the hosts the adapter of Hesse names; \
                 your entry in {} was removed",
                path.display()
            )]
        );
        let after = load(Some(&path), &built_in);
        assert!(after.redundant.is_empty());
        assert_eq!(after.hosts["hessen"], ["cdn.example.net"]);
        assert!(drop_redundant(&path, &after, &built_in).is_empty());
    }

    /// Writing the file keeps what this version does not accept: an entry for an adapter it
    /// does not know (a newer freemapper may) and a host that fails the check. Reading leaves
    /// them out; only the entry a command names is added or removed.
    #[test]
    fn rewriting_the_file_keeps_what_this_version_does_not_accept() {
        let dir = tempfile::tempdir().unwrap();
        let path = file_in(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "[allow]\natlantis = [\"cdn.example.net\"]\n\
             hessen = [\"10.0.0.1\", \"gds.hessen.de\", \"old.example.net\"]\n",
        )
        .unwrap();
        let built_in = builtin();
        let stored = || read(&path).unwrap().allow;
        allow(
            Some(&path),
            "hessen",
            "cdn.example.net",
            &built_in,
            &built_in,
        )
        .unwrap();
        assert_eq!(stored()["atlantis"], ["cdn.example.net"]);
        assert_eq!(
            stored()["hessen"],
            [
                "10.0.0.1",
                "gds.hessen.de",
                "old.example.net",
                "cdn.example.net"
            ]
        );
        // The entry a release made redundant goes, and nothing else.
        let allowed = load(Some(&path), &built_in);
        assert_eq!(drop_redundant(&path, &allowed, &built_in).len(), 1);
        assert_eq!(stored()["atlantis"], ["cdn.example.net"]);
        assert_eq!(
            stored()["hessen"],
            ["10.0.0.1", "old.example.net", "cdn.example.net"]
        );
        let taken = disallow(
            Some(&path),
            "hessen",
            "old.example.net",
            &built_in,
            &built_in,
        );
        assert!(taken.is_ok());
        assert_eq!(stored()["hessen"], ["10.0.0.1", "cdn.example.net"]);
        // What this version ignores can still be taken back by its name.
        for (adapter, host) in [("hessen", "10.0.0.1"), ("atlantis", "cdn.example.net")] {
            let taken = disallow(Some(&path), adapter, host, &built_in, &built_in).unwrap();
            assert!(taken.starts_with("freemapper no longer asks "), "{taken}");
        }
        assert_eq!(stored().len(), 1);
        assert_eq!(stored()["hessen"], ["cdn.example.net"]);
    }

    /// A file that cannot be read is left as it is: no command writes over it.
    #[test]
    fn a_file_that_does_not_parse_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = file_in(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let broken = "[allow]\nhessen = [\"old.example.net\"\n# a bracket is missing\n";
        std::fs::write(&path, broken).unwrap();
        let built_in = builtin();
        for change in [allow, disallow] {
            let refused = change(
                Some(&path),
                "hessen",
                "cdn.example.net",
                &built_in,
                &built_in,
            )
            .unwrap_err();
            assert!(
                refused.starts_with(&format!("{} cannot be read (", path.display())),
                "{refused}"
            );
            assert!(
                refused.ends_with("correct or remove the file first; nothing was changed"),
                "{refused}"
            );
        }
        let allowed = Allowed {
            redundant: vec![("hessen".to_string(), "old.example.net".to_string())],
            ..Allowed::default()
        };
        assert!(drop_redundant(&path, &allowed, &built_in).is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), broken);
    }

    /// A file that is a link stays a link: the entry is written where the link points.
    #[cfg(unix)]
    #[test]
    fn a_linked_file_is_written_where_it_points() {
        let dir = tempfile::tempdir().unwrap();
        let path = file_in(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let real = dir.path().join("dotfiles").join("hosts.toml");
        std::fs::create_dir_all(real.parent().unwrap()).unwrap();
        std::fs::write(&real, "[allow]\n").unwrap();
        std::os::unix::fs::symlink(&real, &path).unwrap();
        let built_in = builtin();
        allow(
            Some(&path),
            "hessen",
            "cdn.example.net",
            &built_in,
            &built_in,
        )
        .unwrap();
        assert!(std::fs::symlink_metadata(&path).unwrap().is_symlink());
        assert!(
            std::fs::read_to_string(&real)
                .unwrap()
                .contains("cdn.example.net")
        );
        // Nothing is left beside either file.
        for folder in [path.parent().unwrap(), real.parent().unwrap()] {
            assert_eq!(std::fs::read_dir(folder).unwrap().count(), 1);
        }
    }

    /// The file lies in the configuration folder of the system, under a fixed name; the
    /// variable that names the adapter folder does not move it.
    #[test]
    fn the_file_is_in_the_configuration_folder_only() {
        let Some(path) = file() else { return };
        assert!(path.ends_with(Path::new("freemapper").join(FILE_NAME)));
        assert_eq!(
            path.parent().and_then(Path::parent),
            dirs::config_dir().as_deref()
        );
    }
}
