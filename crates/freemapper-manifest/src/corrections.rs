// SPDX-License-Identifier: AGPL-3.0-or-later

//! The list of corrections of addresses (issue #273, stage 4): the one file that brings a changed
//! address to a program without a new release.
//!
//! The list is untrusted until its signature has been checked; this module only reads and applies
//! it, the caller checks the signature first (`freemapper-cli`, as the self-update does). A
//! correction names one address of a built-in manifest by its path, the value it must still have
//! (`from`) and the value it becomes (`to`). A manifest that no longer holds `from` is left as it
//! is: the program already has the newer address, and a list that is read twice or comes late
//! does no harm. Every changed manifest is parsed and checked again as a whole, with the host
//! rules, before it counts; a correction that makes it fail is refused alone.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::edit::{self, EditError};

/// The name of the list as a release asset; its signature has `.minisig` added.
pub const FILE: &str = "freemapper-sources.json";
/// The format this program reads.
pub const FORMAT: i64 = 1;
/// The largest list read, in bytes; the same limit as of a signature.
pub const MAX_BYTES: usize = 64 * 1024;
/// The most corrections one list may hold.
pub const MAX_CORRECTIONS: usize = 200;

/// The list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct List {
    pub format: i64,
    /// The day the list was made, `YYYY-MM-DD`. A list older than the one a person has is refused.
    pub issued: String,
    pub corrections: Vec<Correction>,
}

/// One change of one address.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correction {
    /// The built-in adapter, `hamburg`.
    pub adapter: String,
    /// The place in its manifest, as `freemapper sources list` names it: `alkis.url`.
    pub path: String,
    /// The address the manifest holds now; only then the correction applies.
    pub from: String,
    pub to: String,
    /// The host of `to` joins the hosts of the manifest when it is not there yet.
    #[serde(default)]
    pub add_host: bool,
    /// Why, for the person who reads it.
    #[serde(default)]
    pub note: String,
}

/// Why a list is not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListError {
    TooLarge,
    /// Not JSON of the schema; the message is the parser's.
    Syntax(String),
    /// Another format than [`FORMAT`].
    Format(i64),
    /// `issued` is no date.
    Issued,
    TooMany,
    /// A field is empty, too long or of the wrong shape: the index of the correction and the field.
    Field(usize, &'static str),
}

impl std::fmt::Display for ListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge => write!(f, "larger than {MAX_BYTES} bytes"),
            Self::Syntax(message) => write!(f, "{message}"),
            Self::Format(format) => write!(f, "format {format}, this program reads {FORMAT}"),
            Self::Issued => write!(f, "`issued` is no date (YYYY-MM-DD)"),
            Self::TooMany => write!(f, "more than {MAX_CORRECTIONS} corrections"),
            Self::Field(index, field) => write!(f, "correction {}: `{field}`", index + 1),
        }
    }
}

/// Reads the list `bytes`: the JSON, the format, the date and the shape of every field.
pub fn parse_list(bytes: &[u8]) -> Result<List, ListError> {
    if bytes.len() > MAX_BYTES {
        return Err(ListError::TooLarge);
    }
    let list: List =
        serde_json::from_slice(bytes).map_err(|error| ListError::Syntax(error.to_string()))?;
    if list.format != FORMAT {
        return Err(ListError::Format(list.format));
    }
    if chrono::NaiveDate::parse_from_str(&list.issued, "%Y-%m-%d").is_err() {
        return Err(ListError::Issued);
    }
    if list.corrections.len() > MAX_CORRECTIONS {
        return Err(ListError::TooMany);
    }
    for (index, correction) in list.corrections.iter().enumerate() {
        let bad = |field| Err(ListError::Field(index, field));
        if !valid_id(&correction.adapter) {
            return bad("adapter");
        }
        if correction.path.is_empty()
            || correction.path.len() > 128
            || has_control(&correction.path)
        {
            return bad("path");
        }
        for (field, value) in [("from", &correction.from), ("to", &correction.to)] {
            if !value.starts_with("https://") || value.len() > 2000 || has_control(value) {
                return bad(field);
            }
        }
        if correction.note.len() > 300 || has_control(&correction.note) {
            return bad("note");
        }
    }
    Ok(list)
}

fn has_control(text: &str) -> bool {
    text.chars().any(char::is_control)
}

/// An adapter id: a short lower-case name.
fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && (2..=32).contains(&id.len())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// What became of one correction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The manifest was changed; the host that joined its hosts, if one did.
    Applied { host_added: Option<String> },
    /// The manifest already holds `to`: a release brought it.
    AlreadyCurrent,
    /// The manifest holds neither `from` nor `to`: another change came in between.
    Stale,
    /// No built-in manifest has this adapter id.
    UnknownAdapter,
    /// The path is not an address of the manifest.
    UnknownPath,
    /// The change would leave the manifest invalid, or lead to a host it does not declare; the
    /// reason is the first finding.
    Refused(String),
}

/// A correction and what became of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub correction: Correction,
    pub outcome: Outcome,
}

/// The result of applying a list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Applied {
    /// The corrected text of each manifest the list changed, by adapter id.
    pub texts: BTreeMap<String, String>,
    pub rows: Vec<Row>,
}

/// Applies `list` to the manifests `manifest_of` gives by adapter id, in the order of the list.
/// Only a manifest that was changed is in [`Applied::texts`].
pub fn apply(list: &List, manifest_of: impl Fn(&str) -> Option<String>) -> Applied {
    let mut applied = Applied::default();
    for correction in &list.corrections {
        let outcome = apply_one(correction, &manifest_of, &mut applied.texts);
        applied.rows.push(Row {
            correction: correction.clone(),
            outcome,
        });
    }
    applied
}

fn apply_one(
    correction: &Correction,
    manifest_of: &impl Fn(&str) -> Option<String>,
    texts: &mut BTreeMap<String, String>,
) -> Outcome {
    let current = match texts.get(&correction.adapter) {
        Some(text) => text.clone(),
        None => match manifest_of(&correction.adapter) {
            Some(text) => text,
            None => return Outcome::UnknownAdapter,
        },
    };
    let addresses = match edit::addresses(&current) {
        Ok(addresses) => addresses,
        Err(error) => return Outcome::Refused(describe(&error)),
    };
    let Some(address) = addresses.iter().find(|a| a.path == correction.path) else {
        return Outcome::UnknownPath;
    };
    if address.value == correction.to {
        return Outcome::AlreadyCurrent;
    }
    if address.value != correction.from {
        return Outcome::Stale;
    }
    let replaced = match edit::replace(
        &current,
        &correction.path,
        &correction.to,
        correction.add_host,
    ) {
        Ok(replaced) => replaced,
        Err(error) => return Outcome::Refused(describe(&error)),
    };
    // The whole manifest again, with the host rules: a change that breaks it does not count.
    if let Err(findings) = crate::parse(&replaced.text, Path::new(&correction.adapter)) {
        let first = findings
            .first()
            .map_or_else(String::new, ToString::to_string);
        return Outcome::Refused(first);
    }
    texts.insert(correction.adapter.clone(), replaced.text);
    Outcome::Applied {
        host_added: replaced.host_added,
    }
}

fn describe(error: &EditError) -> String {
    match error {
        EditError::Syntax(message) => message.clone(),
        EditError::UnknownPath(path) => format!("no address at {path}"),
        EditError::BadValue => "the new value is not a plain address".to_string(),
        EditError::BadHost => "the host is not a plain DNS name".to_string(),
        EditError::HostNotDeclared(host) => {
            format!("the manifest does not declare the host {host}")
        }
    }
}
