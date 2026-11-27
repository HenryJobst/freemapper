// SPDX-License-Identifier: AGPL-3.0-or-later

//! The addresses of a manifest text, and changing one of them in place (issue #273, stage 3).
//!
//! An address is a string value that starts with `https://` or an entry of `[net] hosts`. It is
//! named by its path in the file: `alkis.url`, `permits.wfs[0].url`, `orthophoto.endpoints.hm`,
//! `net.hosts[2]`. [`addresses`] reads them with their place in the text; [`replace`] puts a new
//! value in the place of one and leaves every other byte of the file, comments included, as it
//! was. Neither checks the manifest: the caller runs [`crate::parse`] on the result before it
//! writes anything.

use std::ops::Range;

use toml::Spanned;
use toml::de::{DeTable, DeValue};

/// One address of a manifest text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    /// Where it stands: `alkis.url`, `permits.wfs[0].url`, `net.hosts[1]`.
    pub path: String,
    /// The address as the file states it.
    pub value: String,
    /// The bytes of the value in the text, with its quotes.
    pub span: Range<usize>,
}

/// Why an address could not be changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    /// The text is not TOML; the message is the parser's.
    Syntax(String),
    /// The manifest has no address under this path.
    UnknownPath(String),
    /// The new value is empty or holds a quote, a backslash or a control character.
    BadValue,
    /// `add_host` was asked for a URL whose host is not a plain DNS name.
    BadHost,
    /// The new address leads to a host the manifest does not declare, and `add_host` was not
    /// asked; the host is named.
    HostNotDeclared(String),
}

/// The addresses of the manifest `text` in the order of the file.
pub fn addresses(text: &str) -> Result<Vec<Address>, EditError> {
    let table = DeTable::parse(text).map_err(|error| EditError::Syntax(error.to_string()))?;
    let mut found = Vec::new();
    walk("", table.get_ref(), &mut found);
    found.sort_by_key(|address| address.span.start);
    Ok(found)
}

fn walk(prefix: &str, table: &DeTable<'_>, found: &mut Vec<Address>) {
    for (key, value) in table {
        let key = key.get_ref();
        let path = if prefix.is_empty() {
            key.to_string()
        } else {
            format!("{prefix}.{key}")
        };
        visit(&path, value, found);
    }
}

fn visit(path: &str, value: &Spanned<DeValue<'_>>, found: &mut Vec<Address>) {
    match value.get_ref() {
        DeValue::String(text) => {
            if text.starts_with("https://") || path.starts_with("net.hosts[") {
                found.push(Address {
                    path: path.to_string(),
                    value: text.to_string(),
                    span: value.span(),
                });
            }
        }
        DeValue::Table(table) => walk(path, table, found),
        DeValue::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                visit(&format!("{path}[{index}]"), item, found);
            }
        }
        _ => {}
    }
}

/// The host of an `https` URL, in lower case; `None` for a URL with a user, a port or no host.
pub fn host_of(url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://")?;
    let host = rest.split(['/', '?', '#']).next()?;
    if host.is_empty() || host.contains(['@', ':']) {
        return None;
    }
    Some(host.to_ascii_lowercase())
}

/// The manifest text after a change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replaced {
    pub text: String,
    /// The host that joined `[net] hosts`, when it did.
    pub host_added: Option<String>,
}

/// The manifest with the address at `path` set to `value`. With `add_host`, the host
/// of `value` joins `[net] hosts` when it is not there; without it, a value whose host the
/// manifest does not declare is refused, so that a change never frees a host unnoticed.
pub fn replace(text: &str, path: &str, value: &str, add_host: bool) -> Result<Replaced, EditError> {
    if value.is_empty()
        || value
            .chars()
            .any(|c| c == '"' || c == '\\' || c.is_control())
    {
        return Err(EditError::BadValue);
    }
    let all = addresses(text)?;
    let target = all
        .iter()
        .find(|address| address.path == path)
        .ok_or_else(|| EditError::UnknownPath(path.to_string()))?;
    // Edits as (range, replacement), applied from the end of the text so that earlier places hold.
    let mut edits = vec![(target.span.clone(), format!("\"{value}\""))];
    let mut host_added = None;
    if !path.starts_with("net.hosts[") {
        let host = host_of(value).ok_or(EditError::BadHost)?;
        let declared = all
            .iter()
            .filter(|address| address.path.starts_with("net.hosts["))
            .any(|address| address.value.eq_ignore_ascii_case(&host));
        if !declared {
            if !add_host {
                return Err(EditError::HostNotDeclared(host));
            }
            let last = all
                .iter()
                .rfind(|address| address.path.starts_with("net.hosts["))
                .ok_or(EditError::BadHost)?;
            edits.push((last.span.end..last.span.end, format!(", \"{host}\"")));
            host_added = Some(host);
        }
    }
    edits.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut result = text.to_string();
    for (range, replacement) in edits {
        result.replace_range(range, &replacement);
    }
    Ok(Replaced {
        text: result,
        host_added,
    })
}
