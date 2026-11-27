// SPDX-License-Identifier: AGPL-3.0-or-later

//! `freemapper sources`: the addresses of the data sources as data a person or an agent can read
//! and correct (issue #273, stage 3).
//!
//! The addresses are those the manifests of the adapters declare (ADR 0189). `list`, `get` and
//! `diff` only read. `set` is the one command that writes, and it writes only in the person's
//! own adapter folder (ADR 0188): when the folder of a built-in adapter is not there yet it is
//! made from the built-in manifest as `adapters export` does (ADR 0196), then one value of
//! `manifest.toml` is replaced and the whole manifest is checked before anything is written.
//! A change that leads to a host the manifest does not declare is refused unless the person
//! says `--add-host`, so a correction never frees a host unnoticed.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use freemapper_manifest::edit::{self, Address, EditError};
use freemapper_manifest::loader::Loaded;
use freemapper_manifest::{MANIFEST_FILE, TAKES_ENDPOINTS, parse};
use serde_json::{Map, Value, json};

use crate::adapter_check::{self, LiveBox};
use crate::adapters::{assemble, user_dir, write_export};
use crate::text::{text, v};

/// The manifest of an adapter and where it comes from.
struct Source {
    /// `built-in` or `user`, as `adapters list --json` says it.
    origin: &'static str,
    text: String,
}

/// One address with the adapter it belongs to.
struct Row {
    adapter: String,
    address: Address,
    origin: &'static str,
}

/// An id is a plain name; it becomes a folder name.
fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && (2..=32).contains(&id.len())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// The manifest text of the built-in adapter `id`; `None` for an adapter code declares.
fn built_in_manifest(id: &str) -> Option<String> {
    freemapper::adapters::export_files(id)?
        .into_iter()
        .find(|(name, _)| name == MANIFEST_FILE)
        .map(|(_, body)| body.into_owned())
}

/// The manifest that is in force for `id`: the one in the person's folder when it loaded, else
/// the built-in one.
fn manifest_in_force(id: &str, user: Option<&Path>, loaded: &[Loaded]) -> Option<Source> {
    if let (Some(user), true) = (user, loaded.iter().any(|loaded| loaded.id == id)) {
        let text = std::fs::read_to_string(user.join(id).join(MANIFEST_FILE)).ok()?;
        return Some(Source {
            origin: "user",
            text,
        });
    }
    built_in_manifest(id).map(|text| Source {
        origin: "built-in",
        text,
    })
}

/// The ids of the adapters the program knows, in registry order.
fn rows(user: Option<&Path>, only: Option<&str>) -> anyhow::Result<(Vec<Row>, bool)> {
    let assembled = assemble(user, None);
    let mut rows = Vec::new();
    let mut known = false;
    for adapter in assembled.registry.adapters() {
        let id = adapter.info().id.to_string();
        if only.is_some_and(|only| only != id) {
            continue;
        }
        known = true;
        let Some(source) = manifest_in_force(&id, user, &assembled.loaded) else {
            continue;
        };
        let addresses = edit::addresses(&source.text)
            .map_err(|error| anyhow::anyhow!(edit_message(&id, &error)))?;
        rows.extend(addresses.into_iter().map(|address| Row {
            adapter: id.clone(),
            address,
            origin: source.origin,
        }));
    }
    Ok((rows, known))
}

/// The sentence for a failed edit of the manifest of `adapter`.
fn edit_message(adapter: &str, error: &EditError) -> String {
    match error {
        // The parser's own words, which are English.
        EditError::Syntax(message) => message.clone(),
        EditError::UnknownPath(path) => text(
            "cli-sources-unknown-path-of",
            &[("adapter", v(adapter)), ("path", v(path))],
        ),
        EditError::BadValue | EditError::BadHost => text("cli-sources-bad-value", &[]),
        EditError::HostNotDeclared(host) => {
            text("cli-sources-host-not-declared", &[("host", v(host))])
        }
    }
}

/// `freemapper sources list [<adapter>] [--json]`.
pub fn list(adapter: Option<&str>, json: bool) -> anyhow::Result<()> {
    let user = user_dir();
    let (rows, known) = rows(user.dir.as_deref(), adapter)?;
    if let (Some(id), false) = (adapter, known) {
        anyhow::bail!(text("cli-sources-unknown-adapter", &[("adapter", v(id))]));
    }
    if let (Some(id), true) = (adapter, rows.is_empty()) {
        anyhow::bail!(text("cli-sources-no-manifest", &[("adapter", v(id))]));
    }
    if json {
        let rows: Vec<Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "adapter": row.adapter,
                    "path": row.address.path,
                    "url": row.address.value,
                    "source": row.origin,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        for row in &rows {
            println!(
                "{}  {}  {}  {}",
                row.adapter, row.address.path, row.address.value, row.origin
            );
        }
    }
    Ok(())
}

/// `freemapper sources get <adapter> <path>`: prints the address and nothing else.
pub fn get(adapter: &str, path: &str) -> anyhow::Result<()> {
    let user = user_dir();
    let (rows, known) = rows(user.dir.as_deref(), Some(adapter))?;
    if !known {
        anyhow::bail!(text(
            "cli-sources-unknown-adapter",
            &[("adapter", v(adapter))]
        ));
    }
    let row = rows
        .iter()
        .find(|row| row.address.path == path)
        .ok_or_else(|| {
            anyhow::anyhow!(text(
                "cli-sources-unknown-path-of",
                &[("adapter", v(adapter)), ("path", v(path))]
            ))
        })?;
    println!("{}", row.address.value);
    Ok(())
}

/// What `set` did.
#[derive(Debug)]
struct Done {
    folder: PathBuf,
    old: String,
    created: bool,
    host_added: Option<String>,
}

/// Changes the address at `path` of the adapter `id` in the folder of the person `user`.
fn change(user: &Path, id: &str, path: &str, url: &str, add_host: bool) -> anyhow::Result<Done> {
    if !valid_id(id) {
        anyhow::bail!(text("cli-sources-bad-id", &[("adapter", v(id))]));
    }
    let folder = user.join(id);
    let own = folder.join(MANIFEST_FILE);
    let (original, files) = if own.is_file() {
        (std::fs::read_to_string(&own)?, None)
    } else {
        let files = freemapper::adapters::export_files(id).ok_or_else(|| {
            anyhow::anyhow!(text("cli-sources-no-manifest", &[("adapter", v(id))]))
        })?;
        let manifest = files
            .iter()
            .find(|(name, _)| name == MANIFEST_FILE)
            .map(|(_, body)| body.to_string())
            .ok_or_else(|| {
                anyhow::anyhow!(text("cli-sources-no-manifest", &[("adapter", v(id))]))
            })?;
        (manifest, Some(files))
    };
    let old = edit::addresses(&original)
        .map_err(|error| anyhow::anyhow!(edit_message(id, &error)))?
        .into_iter()
        .find(|address| address.path == path)
        .map(|address| address.value)
        .ok_or_else(|| {
            anyhow::anyhow!(text(
                "cli-sources-unknown-path-of",
                &[("adapter", v(id)), ("path", v(path))]
            ))
        })?;
    let edit::Replaced {
        text: changed,
        host_added,
    } = edit::replace(&original, path, url, add_host)
        .map_err(|error| anyhow::anyhow!(edit_message(id, &error)))?;
    // The whole manifest has to pass before a byte is written.
    if let Err(findings) = parse(&changed, &folder) {
        let lines: Vec<String> = findings.iter().map(ToString::to_string).collect();
        anyhow::bail!(text(
            "cli-sources-invalid",
            &[("findings", v(lines.join("\n")))]
        ));
    }
    let created = files.is_some();
    if let Some(files) = &files {
        write_export(&folder, files)?;
    }
    std::fs::write(&own, changed)?;
    Ok(Done {
        folder,
        old,
        created,
        host_added,
    })
}

/// `freemapper sources set <adapter> <path> <url> [--add-host]`.
pub fn set(adapter: &str, path: &str, url: &str, add_host: bool) -> anyhow::Result<()> {
    let user = user_dir();
    let dir = user
        .dir
        .ok_or_else(|| anyhow::anyhow!(text("cli-sources-no-user-dir", &[])))?;
    let done = change(&dir, adapter, path, url, add_host)?;
    if done.created {
        println!(
            "{}",
            text(
                "cli-sources-set-created",
                &[
                    ("adapter", v(adapter)),
                    ("folder", v(done.folder.display()))
                ]
            )
        );
    }
    println!("{}: {path}: {} -> {url}", done.folder.display(), done.old);
    if let Some(host) = &done.host_added {
        println!(
            "{}",
            text("cli-sources-set-host-added", &[("host", v(host))])
        );
    }
    println!(
        "{}",
        text(
            "cli-sources-set-next",
            &[("folder", v(done.folder.display()))]
        )
    );
    Ok(())
}

/// The lines of `diff`: what the folder of the person changes against the built-in manifest.
fn diff_lines(user: &Path, id: &str) -> anyhow::Result<Vec<String>> {
    if !valid_id(id) {
        anyhow::bail!(text("cli-sources-bad-id", &[("adapter", v(id))]));
    }
    let own = std::fs::read_to_string(user.join(id).join(MANIFEST_FILE)).map_err(|_| {
        anyhow::anyhow!(text(
            "cli-sources-diff-no-folder",
            &[("adapter", v(id)), ("folder", v(user.join(id).display()))]
        ))
    })?;
    let built_in = built_in_manifest(id)
        .ok_or_else(|| anyhow::anyhow!(text("cli-sources-no-manifest", &[("adapter", v(id))])))?;
    let by_path = |text: &str| -> anyhow::Result<BTreeMap<String, String>> {
        Ok(edit::addresses(text)
            .map_err(|error| anyhow::anyhow!(edit_message(id, &error)))?
            .into_iter()
            .map(|address| (address.path, address.value))
            .collect())
    };
    let (was, now) = (by_path(&built_in)?, by_path(&own)?);
    let mut lines = Vec::new();
    for (path, old) in &was {
        match now.get(path) {
            Some(new) if new == old => {}
            Some(new) => lines.push(format!("~ {path}: {old} -> {new}")),
            None => lines.push(format!("- {path}: {old}")),
        }
    }
    for (path, new) in &now {
        if !was.contains_key(path) {
            lines.push(format!("+ {path}: {new}"));
        }
    }
    Ok(lines)
}

/// `freemapper sources diff <adapter>`.
pub fn diff(adapter: &str) -> anyhow::Result<()> {
    let user = user_dir();
    let dir = user
        .dir
        .ok_or_else(|| anyhow::anyhow!(text("cli-sources-no-user-dir", &[])))?;
    let lines = diff_lines(&dir, adapter)?;
    if lines.is_empty() {
        println!("{}", text("cli-sources-diff-none", &[]));
    }
    for line in lines {
        println!("{line}");
    }
    Ok(())
}

/// `freemapper sources check <adapter>`: the check of `adapters check` on the folder of the
/// person's adapter of that id.
pub fn check(
    adapter: &str,
    live_box: Option<LiveBox>,
    json: bool,
    cache: impl FnOnce() -> anyhow::Result<freemapper::fetch::Cache>,
) -> anyhow::Result<ExitCode> {
    if !valid_id(adapter) {
        anyhow::bail!(text("cli-sources-bad-id", &[("adapter", v(adapter))]));
    }
    let user = user_dir();
    let dir = user
        .dir
        .ok_or_else(|| anyhow::anyhow!(text("cli-sources-no-user-dir", &[])))?;
    adapter_check::run(&dir.join(adapter), live_box, json, cache)
}

/// A node of the tree of address slots.
#[derive(Default)]
struct Node {
    /// The key holds an array of this node.
    array: bool,
    /// The key itself is an address.
    leaf: bool,
    children: BTreeMap<String, Node>,
}

impl Node {
    fn insert(&mut self, segments: &[&str]) {
        let Some((first, rest)) = segments.split_first() else {
            self.leaf = true;
            return;
        };
        let (name, array) = first
            .strip_suffix("[]")
            .map_or((*first, false), |name| (name, true));
        let child = self.children.entry(name.to_string()).or_default();
        child.array |= array;
        child.insert(rest);
    }

    fn schema(&self, host: bool) -> Value {
        let item = if self.children.is_empty() {
            json!({ "$ref": if host { "#/$defs/host" } else { "#/$defs/url" } })
        } else {
            let properties: Map<String, Value> = self
                .children
                .iter()
                .map(|(name, child)| (name.clone(), child.schema(name == "hosts")))
                .collect();
            json!({ "type": "object", "properties": properties })
        };
        if self.array {
            json!({ "type": "array", "items": item })
        } else {
            item
        }
    }
}

/// The address slots of the built-in manifests: the paths with the indices left out
/// (`permits.wfs[].url`).
fn slots() -> Vec<String> {
    let mut slots = std::collections::BTreeSet::new();
    for adapter in freemapper::adapters::builtin().adapters() {
        let Some(manifest) = built_in_manifest(adapter.info().id.as_str()) else {
            continue;
        };
        for address in edit::addresses(&manifest).unwrap_or_default() {
            slots.insert(slot_of(&address.path));
        }
    }
    slots.into_iter().collect()
}

/// `permits.wfs[3].url` is the slot `permits.wfs[].url`.
fn slot_of(path: &str) -> String {
    let mut slot = String::new();
    let mut in_index = false;
    for c in path.chars() {
        match c {
            '[' => {
                in_index = true;
                slot.push('[');
            }
            ']' => {
                in_index = false;
                slot.push(']');
            }
            _ if in_index => {}
            _ => slot.push(c),
        }
    }
    slot
}

/// The JSON Schema of the addresses a manifest holds, for the slots in `slots`.
fn schema_of(slots: &[String]) -> Value {
    let mut root = Node::default();
    for slot in slots {
        let segments: Vec<&str> = slot.split('.').collect();
        root.insert(&segments);
    }
    let Value::Object(mut schema) = root.schema(false) else {
        unreachable!("the root has children");
    };
    schema.insert(
        "$schema".into(),
        json!("https://json-schema.org/draft/2020-12/schema"),
    );
    schema.insert("title".into(), json!("Addresses of a freemapper manifest"));
    schema.insert(
        "description".into(),
        json!(
            "The keys of manifest.toml (format 1) that hold an address of a data source. \
             Other keys are described in docs/adapters/manifest-v1.en.md."
        ),
    );
    schema.insert(
        "$defs".into(),
        json!({
            "url": {
                "type": "string",
                "pattern": r"^https://[A-Za-z0-9.-]+(/[^?#\s]*)?$",
                "description": "https, port 443, a DNS name, no query; the host must be in net.hosts"
            },
            "host": {
                "type": "string",
                "pattern": r"^[A-Za-z0-9.-]+$",
                "description": "a DNS name, no address, no port"
            }
        }),
    );
    schema.insert(
        "x-freemapper-endpoint-names".into(),
        TAKES_ENDPOINTS
            .iter()
            .map(|(backend, names)| ((*backend).to_string(), json!(names)))
            .collect::<Map<String, Value>>()
            .into(),
    );
    Value::Object(schema)
}

/// `freemapper sources schema [--json]`.
pub fn schema(json: bool) -> anyhow::Result<()> {
    let slots = slots();
    if json {
        println!("{}", serde_json::to_string_pretty(&schema_of(&slots))?);
    } else {
        for slot in &slots {
            println!("{slot}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn ids_are_plain_names() {
        for good in ["hamburg", "saxonyanhalt", "ab", "a1"] {
            assert!(valid_id(good), "{good}");
        }
        for bad in [
            "",
            "a",
            "1a",
            "Ab",
            "a-b",
            "a/b",
            "..",
            "a.b",
            &"a".repeat(33),
        ] {
            assert!(!valid_id(bad), "{bad}");
        }
        assert!(valid_id(&"a".repeat(32)));
    }

    #[test]
    fn a_slot_leaves_the_indices_out() {
        assert_eq!(slot_of("permits.wfs[3].url"), "permits.wfs[].url");
        assert_eq!(slot_of("net.hosts[12]"), "net.hosts[]");
        assert_eq!(slot_of("alkis.url"), "alkis.url");
    }

    #[test]
    fn the_schema_nests_the_slots_and_marks_arrays() {
        let schema = schema_of(&[
            "alkis.url".to_string(),
            "net.hosts[]".to_string(),
            "permits.wfs[].url".to_string(),
        ]);
        assert_eq!(
            schema["properties"]["alkis"]["properties"]["url"]["$ref"],
            "#/$defs/url"
        );
        assert_eq!(
            schema["properties"]["net"]["properties"]["hosts"]["type"],
            "array"
        );
        assert_eq!(
            schema["properties"]["net"]["properties"]["hosts"]["items"]["$ref"],
            "#/$defs/host"
        );
        assert_eq!(
            schema["properties"]["permits"]["properties"]["wfs"]["items"]["properties"]["url"]["$ref"],
            "#/$defs/url"
        );
        assert_eq!(schema["$defs"]["url"]["type"], "string");
        assert!(schema["x-freemapper-endpoint-names"]["lgv"].is_array());
    }

    #[test]
    fn the_slots_of_the_built_in_manifests_cover_the_known_places() {
        let slots = slots();
        for known in [
            "orthophoto.url",
            "net.hosts[]",
            "orthophoto.url",
            "permits.wfs[].url",
        ] {
            assert!(slots.contains(&known.to_string()), "{known}: {slots:?}");
        }
        assert!(
            slots.windows(2).all(|pair| pair[0] < pair[1]),
            "sorted and distinct"
        );
    }

    #[test]
    fn set_makes_the_folder_from_the_built_in_manifest_and_changes_one_address() {
        let dir = temp();
        let (rows_before, _) = rows(None, Some("hamburg")).unwrap();
        let first = rows_before
            .iter()
            .find(|row| row.address.path == "orthophoto.url")
            .unwrap();
        let host = edit::host_of(&first.address.value).unwrap();
        let url = format!("https://{host}/other/path");
        let done = change(dir.path(), "hamburg", "orthophoto.url", &url, false).unwrap();
        assert!(done.created);
        assert_eq!(done.old, first.address.value);
        assert_eq!(done.host_added, None);
        assert_eq!(done.folder, dir.path().join("hamburg"));
        let manifest = std::fs::read_to_string(dir.path().join("hamburg/manifest.toml")).unwrap();
        assert!(manifest.contains(&format!("\"{url}\"")), "{manifest}");
        // Only that value differs from the built-in text.
        let built_in = built_in_manifest("hamburg").unwrap();
        assert_eq!(
            manifest,
            built_in.replacen(
                &format!("\"{}\"", first.address.value),
                &format!("\"{url}\""),
                1
            )
        );
        // The second change finds the folder and does not make it again.
        let again = change(
            dir.path(),
            "hamburg",
            "orthophoto.url",
            &first.address.value,
            false,
        )
        .unwrap();
        assert!(!again.created);
        assert_eq!(again.old, url);
        // The diff against the built-in text now says nothing.
        assert!(diff_lines(dir.path(), "hamburg").unwrap().is_empty());
    }

    #[test]
    fn set_refuses_what_the_manifest_would_not_pass_and_writes_nothing() {
        let dir = temp();
        // A host the manifest does not declare, without --add-host.
        let error = change(
            dir.path(),
            "hamburg",
            "orthophoto.url",
            "https://elsewhere.example.org/x",
            false,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("elsewhere.example.org"), "{error}");
        assert!(!dir.path().join("hamburg").exists());
        // Not https.
        let error = change(
            dir.path(),
            "hamburg",
            "orthophoto.url",
            "http://x.example.org/x",
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("host"), "{error}");
        assert!(!dir.path().join("hamburg").exists());
        // A query is no address the manifest takes.
        let (rows, _) = rows(None, Some("hamburg")).unwrap();
        let host = edit::host_of(
            &rows
                .iter()
                .find(|r| r.address.path == "orthophoto.url")
                .unwrap()
                .address
                .value,
        )
        .unwrap();
        let error = change(
            dir.path(),
            "hamburg",
            "orthophoto.url",
            &format!("https://{host}/x?a=b"),
            false,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("policy.url"), "{error}");
        assert!(!dir.path().join("hamburg").exists());
        // Unknown path, unknown adapter, bad id.
        let error = change(
            dir.path(),
            "hamburg",
            "alkis.nothing",
            "https://a.example.org/x",
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("alkis.nothing"), "{error}");
        let error = change(
            dir.path(),
            "polska",
            "orthophoto.url",
            "https://a.example.org/x",
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("polska"), "{error}");
        let error = change(
            dir.path(),
            "../x",
            "orthophoto.url",
            "https://a.example.org/x",
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("../x"), "{error}");
    }

    #[test]
    fn add_host_adds_the_host_and_says_so() {
        let dir = temp();
        let done = change(
            dir.path(),
            "hamburg",
            "orthophoto.url",
            "https://moved.example.org/wfs",
            true,
        )
        .unwrap();
        assert_eq!(done.host_added.as_deref(), Some("moved.example.org"));
        let manifest = std::fs::read_to_string(dir.path().join("hamburg/manifest.toml")).unwrap();
        assert!(manifest.contains("\"moved.example.org\""), "{manifest}");
        let lines = diff_lines(dir.path(), "hamburg").unwrap();
        assert!(
            lines
                .iter()
                .any(|l| l.contains("orthophoto.url") && l.contains("moved.example.org")),
            "{lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|l| l.contains("net.hosts[") && l.contains("moved.example.org")),
            "{lines:?}"
        );
    }

    #[test]
    fn diff_needs_a_folder_and_a_manifest() {
        let dir = temp();
        let error = diff_lines(dir.path(), "hamburg").unwrap_err().to_string();
        assert!(error.contains("hamburg"), "{error}");
        assert!(diff_lines(dir.path(), "Bad").is_err());
    }
}
