//! The validator: from the tables as parsed to a checked [`Manifest`], or to the findings.
//!
//! Every value is checked against a closed list, a range or a pattern, and every URL against
//! the policy and the declared hosts. The validator goes on after a finding, so one run names
//! all of them.

use std::ops::Range;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use freemapper_adapter_api::policy;
use freemapper_georef::crs::Crs;
use freemapper_sources::nas::NAS_LAYERS;
use toml::Spanned;

use crate::error::ManifestError;
use crate::model::{
    Adapter, Alkis, Attribution, Boundary, BoundaryCrs, DateFormat, DateSource, Estimate,
    HeightModel, Licence, Manifest, Names, Nas, Normalise, Orthophoto, OsmExtract, Preview,
    TreeFields, Trees, Version, Wfs,
};
use crate::schema::{
    Params, Raw, RawAdapter, RawAlkis, RawAttribution, RawBoundary, RawDate, RawEstimate,
    RawExtract, RawNas, RawPreview, RawRaster, RawTrees, Text,
};
use crate::{FORMAT, MANIFEST_FILE, MAX_MANIFEST_BYTES, MAX_URLS};

/// The largest answer a manifest may allow one request, in bytes: the cap of the NAS reader.
pub const MAX_ANSWER_BYTES: i64 = 256 * 1024 * 1024;

/// The request parameters a service table may not set: the readers set them.
const RESERVED_PARAMS: [&str; 14] = [
    "service",
    "version",
    "request",
    "layers",
    "styles",
    "crs",
    "srs",
    "bbox",
    "width",
    "height",
    "format",
    "coverageid",
    "subset",
    "query_layers",
];

pub(crate) fn parse(text: &str, folder: &Path) -> Result<Manifest, Vec<ManifestError>> {
    let path = folder.join(MANIFEST_FILE);
    if text.len() > MAX_MANIFEST_BYTES {
        return Err(vec![ManifestError {
            code: "manifest.too-large",
            path,
            line: 0,
            key: String::new(),
            message: format!(
                "the manifest has {} bytes; a manifest has at most {MAX_MANIFEST_BYTES}",
                text.len()
            ),
        }]);
    }
    // Editors on Windows write a byte order mark; it is no part of the text.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut check = Check {
        text,
        path,
        findings: Vec::new(),
        hosts: None,
        urls: 0,
    };
    let table: toml::Table = match toml::from_str(text) {
        Ok(table) => table,
        Err(error) => return Err(vec![check.of_parser(&error)]),
    };
    if let Err(message) = format_of(&table) {
        let line = text
            .lines()
            .position(|line| line.trim_start().starts_with("format"))
            .map_or(0, |index| index + 1);
        return Err(vec![check.finding(
            "manifest.format",
            line,
            "format",
            message,
        )]);
    }
    let raw: Raw = match toml::from_str(text) {
        Ok(raw) => raw,
        Err(error) => return Err(vec![check.of_parser(&error)]),
    };
    let manifest = check.manifest(&raw, folder);
    let mut findings = check.findings;
    // In the order of the file; findings without a place first.
    findings.sort_by_key(|finding| finding.line);
    match manifest {
        Some(manifest) if findings.is_empty() => Ok(manifest),
        _ => Err(findings),
    }
}

/// Whether the file says a format this program reads.
fn format_of(table: &toml::Table) -> Result<(), String> {
    match table.get("format") {
        Some(toml::Value::Integer(FORMAT)) => Ok(()),
        Some(toml::Value::Integer(other)) => Err(format!(
            "this freemapper reads format {FORMAT}, not format {other}"
        )),
        Some(_) => Err(format!("`format` is the number {FORMAT}")),
        None => Err(format!(
            "the manifest does not start with `format = {FORMAT}`"
        )),
    }
}

fn is_id(text: &str) -> bool {
    let bytes = text.as_bytes();
    (2..=32).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

/// A lower-case name with hyphens: of a backend, a cache file, a tree cadastre, a label.
fn is_slug(text: &str, max: usize) -> bool {
    let bytes = text.as_bytes();
    (1..=max).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

/// The name of a layer or a coverage of a service.
fn is_layer(text: &str) -> bool {
    (1..=100).contains(&text.len())
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
}

/// The name of a field or of one part of a feature type.
fn is_field(text: &str) -> bool {
    let bytes = text.as_bytes();
    (1..=60).contains(&bytes.len())
        && (bytes[0].is_ascii_alphabetic() || bytes[0] == b'_')
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(b))
}

/// A feature type, with or without a prefix (`ave:GebaeudeBauwerk`).
fn is_type_name(text: &str) -> bool {
    let mut parts = text.split(':');
    let (first, second) = (parts.next(), parts.next());
    parts.next().is_none() && first.is_some_and(is_field) && second.is_none_or(is_field)
}

/// A path of an extract on the Geofabrik server (`europe/germany/hessen`).
fn is_geofabrik_path(text: &str) -> bool {
    let segments: Vec<&str> = text.split('/').collect();
    text.len() <= 120
        && (1..=6).contains(&segments.len())
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && !segment.starts_with('-')
                && segment
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

/// A plain file name in the adapter folder with the ending `.geojson`.
fn is_geojson_name(text: &str) -> bool {
    freemapper_fs::is_plain_name(text)
        && text.len() <= 64
        && text.ends_with(".geojson")
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

/// What goes into the `NAMESPACES` parameter of a NAS request.
fn is_namespaces(text: &str) -> bool {
    (1..=300).contains(&text.len())
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.:/(),=-".contains(&b))
}

struct Check<'a> {
    /// The manifest, without a byte order mark.
    text: &'a str,
    path: PathBuf,
    findings: Vec<ManifestError>,
    /// `[net] hosts` in lower case; `None` while the list itself has a finding.
    hosts: Option<Vec<String>>,
    /// The URLs seen so far.
    urls: usize,
}

impl Check<'_> {
    fn finding(
        &self,
        code: &'static str,
        line: usize,
        key: &str,
        message: String,
    ) -> ManifestError {
        ManifestError {
            code,
            path: self.path.clone(),
            line,
            key: key.to_string(),
            message,
        }
    }

    /// The line of the byte `at`, from 1.
    fn line(&self, at: usize) -> usize {
        let before = &self.text.as_bytes()[..at.min(self.text.len())];
        before.iter().filter(|b| **b == b'\n').count() + 1
    }

    fn fail(
        &mut self,
        code: &'static str,
        at: &Range<usize>,
        key: &str,
        message: impl Into<String>,
    ) {
        let finding = self.finding(code, self.line(at.start), key, message.into());
        self.findings.push(finding);
    }

    /// A finding about the manifest as a whole.
    fn fail_whole(&mut self, code: &'static str, key: &str, message: impl Into<String>) {
        let finding = self.finding(code, 0, key, message.into());
        self.findings.push(finding);
    }

    /// The finding for an error of the TOML parser or of the schema.
    fn of_parser(&self, error: &toml::de::Error) -> ManifestError {
        let message = error.message();
        let named = || message.split('`').nth(1).unwrap_or_default().to_string();
        let line = error.span().map_or(0, |span| self.line(span.start));
        // The key at the start of the line, where the parser names none.
        let key_of_line = || {
            let text = self.text.lines().nth(line.saturating_sub(1)).unwrap_or("");
            match text.split_once('=') {
                Some((key, _)) if line > 0 => key.trim().to_string(),
                _ => String::new(),
            }
        };
        let (code, key) = if message.starts_with("unknown field") {
            ("manifest.unknown-key", named())
        } else if message.starts_with("missing field") {
            ("manifest.missing-key", named())
        } else if message.starts_with("invalid type") || message.starts_with("invalid value") {
            ("manifest.type", key_of_line())
        } else {
            ("manifest.syntax", String::new())
        };
        self.finding(code, line, &key, message.to_string())
    }

    /// A text without control characters, of 1 to `max` characters.
    fn text(&mut self, value: &Text, key: &str, code: &'static str, max: usize) -> Option<String> {
        let text = value.get_ref();
        let problem = if text.trim().is_empty() {
            "is empty".to_string()
        } else if text.chars().count() > max {
            format!("has more than {max} characters")
        } else if text.chars().any(char::is_control) {
            "has a line break or another control character".to_string()
        } else {
            return Some(text.clone());
        };
        self.fail(code, &value.span(), key, format!("the text {problem}"));
        None
    }

    fn optional_text(
        &mut self,
        value: Option<&Text>,
        key: &str,
        code: &'static str,
        max: usize,
    ) -> Option<Option<String>> {
        match value {
            Some(value) => self.text(value, key, code, max).map(Some),
            None => Some(None),
        }
    }

    /// A value that `rule` accepts; `expected` says in words what that is.
    fn matching(
        &mut self,
        value: &Text,
        key: &str,
        code: &'static str,
        rule: impl Fn(&str) -> bool,
        expected: &str,
    ) -> Option<String> {
        let text = value.get_ref();
        if rule(text) {
            return Some(text.clone());
        }
        let shown: String = text.chars().filter(|c| !c.is_control()).take(60).collect();
        self.fail(
            code,
            &value.span(),
            key,
            format!("`{shown}` is not {expected}"),
        );
        None
    }

    /// A URL the policy accepts, without a query, on a declared host. It counts.
    fn url(&mut self, value: &Text, key: &str) -> Option<String> {
        self.urls += 1;
        if self.urls == MAX_URLS + 1 {
            self.fail(
                "manifest.too-many-urls",
                &value.span(),
                key,
                format!("a manifest holds at most {MAX_URLS} URLs"),
            );
        }
        let url = value.get_ref();
        let host = match policy::host_of(url) {
            Ok(host) => host,
            Err(refused) => {
                self.fail(refused.code, &value.span(), key, refused.to_string());
                return None;
            }
        };
        if url.contains(['?', '#']) {
            let shown = url.split(['?', '#']).next().unwrap_or_default();
            self.fail(
                "policy.url",
                &value.span(),
                key,
                format!("{shown}: a query or a fragment; request parameters belong in `params`"),
            );
            return None;
        }
        if self
            .hosts
            .as_ref()
            .is_some_and(|hosts| !hosts.contains(&host))
        {
            self.fail(
                "policy.host-not-declared",
                &value.span(),
                key,
                format!("{url}: the host {host} is not in `[net] hosts`"),
            );
            return None;
        }
        Some(url.clone())
    }

    fn crs(&mut self, value: &Spanned<i64>, key: &str) -> Option<Crs> {
        let crs = u32::try_from(*value.get_ref())
            .ok()
            .and_then(Crs::from_epsg);
        if crs.is_none() {
            self.fail(
                "manifest.crs",
                &value.span(),
                key,
                format!(
                    "EPSG:{} is not a CRS of a map; a map is in EPSG:25832 or EPSG:25833",
                    value.get_ref()
                ),
            );
        }
        crs
    }

    /// The CRS of a service: the one it names, else the one of the adapter.
    fn service_crs(
        &mut self,
        value: Option<&Spanned<i64>>,
        key: &str,
        adapter: Option<Crs>,
    ) -> Option<Crs> {
        match value {
            Some(value) => self.crs(value, key),
            None => adapter,
        }
    }

    fn number(&mut self, value: &Spanned<f64>, key: &str, range: Range<f64>) -> Option<f64> {
        let number = *value.get_ref();
        // The range is open at its start and closed at its end: a size is more than nothing.
        if number.is_finite() && number > range.start && number <= range.end {
            return Some(number);
        }
        self.fail(
            "manifest.value",
            &value.span(),
            key,
            format!(
                "{number} is not a number above {} and up to {}",
                range.start, range.end
            ),
        );
        None
    }

    fn integer(&mut self, value: &Spanned<i64>, key: &str, min: i64, max: i64) -> Option<i64> {
        let number = *value.get_ref();
        if (min..=max).contains(&number) {
            return Some(number);
        }
        self.fail(
            "manifest.value",
            &value.span(),
            key,
            format!("{number} is not a whole number from {min} to {max}"),
        );
        None
    }

    /// `[min_e, min_n, max_e, max_n]`.
    fn envelope(&mut self, value: &Spanned<Vec<f64>>, key: &str) -> Option<[f64; 4]> {
        match value.get_ref()[..] {
            [min_e, min_n, max_e, max_n]
                if [min_e, min_n, max_e, max_n].iter().all(|v| v.is_finite())
                    && min_e < max_e
                    && min_n < max_n =>
            {
                Some([min_e, min_n, max_e, max_n])
            }
            _ => {
                self.fail(
                    "manifest.value",
                    &value.span(),
                    key,
                    "an extent is `[min_e, min_n, max_e, max_n]` with each minimum below its maximum",
                );
                None
            }
        }
    }

    /// More parameters of a request, by name. The readers encode the values.
    fn params(
        &mut self,
        value: Option<&Spanned<Params>>,
        key: &str,
    ) -> Option<Vec<(String, String)>> {
        let Some(table) = value else {
            return Some(Vec::new());
        };
        let mut params = Vec::new();
        let mut fine = table.get_ref().len() <= 16;
        if !fine {
            self.fail(
                "manifest.params",
                &table.span(),
                key,
                "more than 16 parameters",
            );
        }
        for (name, value) in table.get_ref() {
            let key = format!("{key}.{name}");
            let name_fits = (1..=40).contains(&name.len())
                && name.as_bytes()[0].is_ascii_alphabetic()
                && name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b));
            if !name_fits {
                self.fail(
                    "manifest.params",
                    &value.span(),
                    &key,
                    "the name of a parameter has letters, digits and `_.:-`",
                );
                fine = false;
            } else if RESERVED_PARAMS.contains(&name.to_ascii_lowercase().as_str()) {
                self.fail(
                    "manifest.params",
                    &value.span(),
                    &key,
                    format!("freemapper sets `{name}` itself"),
                );
                fine = false;
            } else if let Some(text) = self.text(value, &key, "manifest.params", 200) {
                params.push((name.clone(), text));
            } else {
                fine = false;
            }
        }
        fine.then_some(params)
    }

    /// A key that does not belong to the `kind` of its table.
    fn absent<T>(&mut self, value: Option<&Spanned<T>>, key: &str, kind: &str) -> Option<()> {
        match value {
            Some(value) => {
                self.fail(
                    "manifest.unknown-key",
                    &value.span(),
                    key,
                    format!("not a key of {kind}"),
                );
                None
            }
            None => Some(()),
        }
    }

    /// A key that the `kind` of its table needs; a missing one is reported at `anchor`.
    fn required<'v, T>(
        &mut self,
        value: Option<&'v Spanned<T>>,
        anchor: &Range<usize>,
        key: &str,
        kind: &str,
    ) -> Option<&'v Spanned<T>> {
        if value.is_none() {
            self.fail(
                "manifest.missing-key",
                anchor,
                key,
                format!("{kind} needs this key"),
            );
        }
        value
    }

    fn manifest(&mut self, raw: &Raw, folder: &Path) -> Option<Manifest> {
        let hosts = self.hosts(&raw.net.hosts);
        self.hosts = hosts.clone();
        let adapter = self.adapter(&raw.adapter, folder);
        let crs = adapter.as_ref().map(|adapter| adapter.crs);
        let english = adapter.as_ref().map(|adapter| adapter.names.en.clone());
        let boundary = self.boundary(&raw.boundary);
        let osm = self.extracts(&raw.osm.extract);
        let attribution = self.attribution(&raw.attribution);
        let alkis = match &raw.alkis {
            Some(alkis) => self.alkis(alkis, crs, english.as_deref()).map(Some),
            None => Some(None),
        };
        let orthophoto = match &raw.orthophoto {
            Some(table) => self.orthophoto(table, crs, english.as_deref()).map(Some),
            None => Some(None),
        };
        let terrain = match &raw.terrain {
            Some(table) => self
                .height_model(table, "terrain", english.as_deref())
                .map(Some),
            None => Some(None),
        };
        let surface = match &raw.surface {
            Some(table) => self
                .height_model(table, "surface", english.as_deref())
                .map(Some),
            None => Some(None),
        };
        let nas = self.nas_services(&raw.extra.nas, crs);
        let trees = self.tree_cadastres(&raw.extra.trees, crs);
        let preview = match &raw.preview {
            Some(preview) => self.preview(preview).map(Some),
            None => Some(None),
        };
        let estimate = self.estimate(&raw.estimate);
        if let (Some(orthophoto), Some(terrain), Some(surface)) = (&orthophoto, &terrain, &surface)
        {
            self.rasters_fit(orthophoto.as_ref(), terrain.as_ref(), surface.as_ref());
        }
        if raw.alkis.is_none() && raw.orthophoto.is_none() {
            self.fail_whole(
                "manifest.empty",
                "",
                "an adapter has `[alkis]`, `[orthophoto]` or both",
            );
        }
        Some(Manifest {
            adapter: adapter?,
            hosts: hosts?,
            boundary: boundary?,
            osm: osm?,
            attribution: attribution?,
            alkis: alkis?,
            orthophoto: orthophoto?,
            terrain: terrain?,
            surface: surface?,
            nas: nas?,
            trees: trees?,
            preview: preview?,
            estimate: estimate?,
        })
    }

    /// The raster tables as a whole: an orthophoto and a terrain model, with or without a
    /// surface model, all of them services or all of them one backend. A backend fetches the
    /// three together, so a table cannot go its own way.
    fn rasters_fit(
        &mut self,
        orthophoto: Option<&Orthophoto>,
        terrain: Option<&HeightModel>,
        surface: Option<&HeightModel>,
    ) {
        if orthophoto.is_none() && terrain.is_none() && surface.is_none() {
            return;
        }
        let (Some(orthophoto), Some(terrain)) = (orthophoto, terrain) else {
            self.fail_whole(
                "manifest.rasters",
                if orthophoto.is_none() { "orthophoto" } else { "terrain" },
                "an adapter with rasters has `[orthophoto]` and `[terrain]`; `[surface]` may follow",
            );
            return;
        };
        // The backend each table names; `None` for a service.
        let of_model = |model: &HeightModel| match model {
            HeightModel::Backend { name } => Some(name.clone()),
            HeightModel::Wcs { .. } => None,
        };
        let first = match orthophoto {
            Orthophoto::Backend { name } => Some(name.clone()),
            Orthophoto::Wms { .. } | Orthophoto::Wcs { .. } => None,
        };
        let others = [Some(terrain), surface].into_iter().flatten().map(of_model);
        if others.into_iter().any(|backend| backend != first) {
            self.fail_whole(
                "manifest.rasters",
                "terrain",
                "the raster tables are all services, or they all name the same backend",
            );
        }
    }

    /// `[net] hosts`: DNS names, in lower case, each once.
    fn hosts(&mut self, entries: &[Text]) -> Option<Vec<String>> {
        let mut hosts = Vec::new();
        let mut fine = true;
        if entries.len() > 32 {
            self.fail(
                "manifest.value",
                &entries[32].span(),
                "net.hosts",
                "more than 32 hosts",
            );
            fine = false;
        }
        for entry in entries {
            match policy::dns_name(entry.get_ref()) {
                Ok(host) if hosts.contains(&host) => {}
                Ok(host) => hosts.push(host),
                Err(refused) => {
                    self.fail(
                        refused.code,
                        &entry.span(),
                        "net.hosts",
                        refused.to_string(),
                    );
                    fine = false;
                }
            }
        }
        fine.then_some(hosts)
    }

    fn adapter(&mut self, raw: &RawAdapter, folder: &Path) -> Option<Adapter> {
        let pattern = "an id: a lower-case letter, then 1 to 31 lower-case letters or digits";
        let id = self.matching(&raw.id, "adapter.id", "manifest.id", is_id, pattern);
        if let (Some(id), Some(name)) = (&id, folder.file_name())
            && name.to_string_lossy() != *id
        {
            self.fail(
                "manifest.id",
                &raw.id.span(),
                "adapter.id",
                format!(
                    "the folder is called `{}`; the folder of an adapter has the name of its id",
                    name.to_string_lossy()
                ),
            );
        }
        let provider = match &raw.provider {
            Some(provider) => {
                self.matching(provider, "adapter.provider", "manifest.id", is_id, pattern)
            }
            None => id.clone(),
        };
        let revision = self.integer(&raw.revision, "adapter.revision", 1, i64::from(u32::MAX));
        let min_freemapper = Version::parse(raw.min_freemapper.get_ref());
        if min_freemapper.is_none() {
            self.fail(
                "manifest.min-freemapper",
                &raw.min_freemapper.span(),
                "adapter.min_freemapper",
                "a version is three numbers, as in `0.13.0`",
            );
        }
        let crs = self.crs(&raw.crs, "adapter.crs");
        let en = self.text(&raw.names.en, "adapter.names.en", "manifest.names", 80);
        let de = match &raw.names.de {
            Some(de) => self.text(de, "adapter.names.de", "manifest.names", 80),
            None => en.clone(),
        };
        let object_id_prefix = match &raw.object_id_prefix {
            Some(prefix) => self
                .matching(
                    prefix,
                    "adapter.object_id_prefix",
                    "manifest.value",
                    |text| {
                        (2..=12).contains(&text.len())
                            && text.as_bytes()[0].is_ascii_uppercase()
                            && text
                                .bytes()
                                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
                    },
                    "the start of an object id: 2 to 12 capital letters or digits",
                )
                .map(Some),
            None => Some(None),
        };
        let mut maintainers = Some(Vec::new());
        if raw.maintainers.len() > 8 {
            self.fail(
                "manifest.value",
                &raw.maintainers[8].span(),
                "adapter.maintainers",
                "more than 8 maintainers",
            );
            maintainers = None;
        }
        for entry in &raw.maintainers {
            let text = self.text(entry, "adapter.maintainers", "manifest.value", 100);
            maintainers = maintainers.zip(text).map(|(mut all, text)| {
                all.push(text);
                all
            });
        }
        let last_verified = match &raw.last_verified {
            Some(date) => {
                let parsed = NaiveDate::parse_from_str(date.get_ref(), "%Y-%m-%d").ok();
                if parsed.is_none() {
                    self.fail(
                        "manifest.value",
                        &date.span(),
                        "adapter.last_verified",
                        "a date is written as in `2026-11-12`",
                    );
                }
                parsed.map(Some)
            }
            None => Some(None),
        };
        let agency = match &raw.agency {
            Some(agency) => self.text(agency, "adapter.agency", "manifest.value", 40),
            None => en.clone(),
        };
        let licence_of_manifest = self.optional_text(
            raw.licence_of_manifest.as_ref(),
            "adapter.licence_of_manifest",
            "manifest.value",
            100,
        );
        Some(Adapter {
            id: id?,
            provider: provider?,
            revision: u32::try_from(revision?).ok()?,
            min_freemapper: min_freemapper?,
            names: Names { en: en?, de: de? },
            agency: agency?,
            crs: crs?,
            object_id_prefix: object_id_prefix?,
            maintainers: maintainers?,
            last_verified: last_verified?,
            licence_of_manifest: licence_of_manifest?,
        })
    }

    fn file(&mut self, value: &Text, key: &str) -> Option<String> {
        self.matching(
            value,
            key,
            "manifest.file-name",
            is_geojson_name,
            "a plain file name in the folder of the adapter that ends in `.geojson`",
        )
    }

    fn boundary(&mut self, raw: &RawBoundary) -> Option<Boundary> {
        let file = self.file(&raw.file, "boundary.file");
        let crs = match raw.crs.get_ref() {
            4326 => Some(BoundaryCrs::Wgs84),
            25833 => Some(BoundaryCrs::Utm33),
            other => {
                self.fail(
                    "manifest.crs",
                    &raw.crs.span(),
                    "boundary.crs",
                    format!("a boundary is in EPSG:4326 or EPSG:25833, not in EPSG:{other}"),
                );
                None
            }
        };
        let simplify_m = match &raw.simplify_m {
            // Zero says the boundary is not simplified.
            Some(value) if *value.get_ref() == 0.0 => Some(0.0),
            Some(value) => self.number(value, "boundary.simplify_m", 0.0..1000.0),
            None => Some(25.0),
        };
        Some(Boundary {
            file: file?,
            crs: crs?,
            simplify_m: simplify_m?,
        })
    }

    fn extracts(&mut self, raw: &[RawExtract]) -> Option<Vec<OsmExtract>> {
        if raw.is_empty() {
            self.fail_whole(
                "manifest.osm",
                "osm.extract",
                "an adapter names at least one `[[osm.extract]]`",
            );
            return None;
        }
        let mut fine = true;
        if raw.len() > 16 {
            self.fail(
                "manifest.osm",
                &raw[16].geofabrik.span(),
                "osm.extract",
                "more than 16 extracts",
            );
            fine = false;
        }
        // With several extracts each says where it lies; the first without a boundary is named.
        if raw.len() > 1
            && let Some(extract) = raw.iter().find(|extract| extract.boundary.is_none())
        {
            self.fail(
                "manifest.osm",
                &extract.geofabrik.span(),
                "osm.extract.boundary",
                "an adapter with several extracts gives each its boundary file",
            );
            fine = false;
        }
        let mut extracts: Vec<OsmExtract> = Vec::new();
        for extract in raw {
            let geofabrik = self.matching(
                &extract.geofabrik,
                "osm.extract.geofabrik",
                "manifest.geofabrik",
                is_geofabrik_path,
                "a path of an extract on download.geofabrik.de, as in `europe/germany/hessen`",
            );
            let megabytes =
                self.number(&extract.megabytes, "osm.extract.megabytes", 0.0..100_000.0);
            let label = self.text(&extract.label, "osm.extract.label", "manifest.value", 80);
            let boundary = match &extract.boundary {
                Some(file) => self.file(file, "osm.extract.boundary").map(Some),
                None => Some(None),
            };
            let cache_prefix = match &extract.cache_prefix {
                Some(prefix) => self.matching(
                    prefix,
                    "osm.extract.cache_prefix",
                    "manifest.value",
                    |text| is_slug(text, 40),
                    "the start of a file name: lower-case letters, digits and hyphens",
                ),
                // The last part of the path: `hessen` of `europe/germany/hessen`.
                None => geofabrik
                    .as_deref()
                    .and_then(|path| path.rsplit('/').next())
                    .map(str::to_string),
            };
            let built = (|| {
                Some(OsmExtract {
                    geofabrik: geofabrik?,
                    megabytes: megabytes?,
                    label: label?,
                    boundary: boundary?,
                    cache_prefix: cache_prefix?,
                })
            })();
            match built {
                Some(built) => {
                    if extracts
                        .iter()
                        .any(|other| other.cache_prefix == built.cache_prefix)
                    {
                        self.fail(
                            "manifest.osm",
                            &extract.geofabrik.span(),
                            "osm.extract.cache_prefix",
                            format!(
                                "two extracts would share the cache file `{}_…`",
                                built.cache_prefix
                            ),
                        );
                        fine = false;
                    }
                    extracts.push(built);
                }
                None => fine = false,
            }
        }
        fine.then_some(extracts)
    }

    /// A credit: one line, with `{year}` at most once and no other placeholder.
    fn credit(&mut self, value: &Text, key: &str, max: usize) -> Option<String> {
        let text = self.text(value, key, "manifest.attribution", max)?;
        let rest = text.replacen("{year}", "", 1);
        if rest.contains(['{', '}']) {
            self.fail(
                "manifest.attribution",
                &value.span(),
                key,
                "a credit has `{year}` at most once and no other placeholder",
            );
            return None;
        }
        Some(text)
    }

    fn attribution(&mut self, raw: &RawAttribution) -> Option<Attribution> {
        let licence = Licence::from_id(raw.licence.get_ref());
        if licence.is_none() {
            let known: Vec<&str> = Licence::ALL.iter().map(|licence| licence.id()).collect();
            self.fail(
                "manifest.licence",
                &raw.licence.span(),
                "attribution.licence",
                format!("a licence is one of {}", known.join(", ")),
            );
        }
        let licence_text = match (&raw.licence_text, licence) {
            (Some(text), Some(Licence::Other)) => self
                .text(text, "attribution.licence_text", "manifest.licence", 2000)
                .map(Some),
            (None, Some(Licence::Other)) => {
                self.fail(
                    "manifest.licence",
                    &raw.licence.span(),
                    "attribution.licence_text",
                    "the licence `other` needs `licence_text`",
                );
                None
            }
            (Some(text), Some(_)) => {
                self.fail(
                    "manifest.licence",
                    &text.span(),
                    "attribution.licence_text",
                    "`licence_text` belongs to the licence `other` only",
                );
                None
            }
            _ => Some(None),
        };
        let text = self.credit(&raw.text, "attribution.text", 400);
        if let Some(year) = raw
            .year
            .as_ref()
            .filter(|year| *year.get_ref() != "newest-fetch")
        {
            self.fail(
                "manifest.value",
                &year.span(),
                "attribution.year",
                "`year` is `newest-fetch`: the year of the newest fetch",
            );
            return None;
        }
        let short = self.optional_text(
            raw.short.as_ref(),
            "attribution.short",
            "manifest.attribution",
            60,
        );
        let note = self.optional_text(
            raw.note.as_ref(),
            "attribution.note",
            "manifest.attribution",
            400,
        );
        Some(Attribution {
            licence: licence?,
            licence_text: licence_text?,
            text: text?,
            short: short?,
            note: note?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn wfs(
        &mut self,
        table: &str,
        name: Option<&Text>,
        default_name: Option<String>,
        url: &Text,
        buildings_type: &Text,
        land_use_type: &Text,
        square_bez: Option<&Text>,
        sorted: bool,
        crs: Option<Crs>,
    ) -> Option<Wfs> {
        let name = match name {
            Some(name) => self.text(name, &format!("{table}.name"), "manifest.value", 80),
            None => default_name,
        };
        let url = self.url(url, &format!("{table}.url"));
        let type_name = "the name of a feature type, as in `ave:GebaeudeBauwerk`";
        let buildings_type = self.matching(
            buildings_type,
            &format!("{table}.buildings_type"),
            "manifest.value",
            is_type_name,
            type_name,
        );
        let land_use_type = self.matching(
            land_use_type,
            &format!("{table}.land_use_type"),
            "manifest.value",
            is_type_name,
            type_name,
        );
        let square_bez = self.optional_text(
            square_bez,
            &format!("{table}.square_bez"),
            "manifest.value",
            100,
        );
        Some(Wfs {
            name: name?,
            url: url?,
            buildings_type: buildings_type?,
            land_use_type: land_use_type?,
            crs: crs?,
            square_bez: square_bez?,
            sorted,
        })
    }

    fn alkis(
        &mut self,
        raw: &RawAlkis,
        adapter: Option<Crs>,
        english: Option<&str>,
    ) -> Option<Alkis> {
        let at = raw.kind.span();
        let crs = self.service_crs(raw.crs.as_ref(), "alkis.crs", adapter);
        match raw.kind.get_ref().as_str() {
            "wfs" => {
                let kind = "kind `wfs`";
                let fine = [
                    self.absent(raw.buildings.as_ref(), "alkis.buildings", kind),
                    self.absent(raw.land_use.as_ref(), "alkis.land_use", kind),
                    self.absent(raw.normalise.as_ref(), "alkis.normalise", kind),
                    self.absent(
                        raw.fallback.as_ref().map(|fallback| &fallback.url),
                        "alkis.fallback",
                        kind,
                    ),
                ]
                .iter()
                .all(Option::is_some);
                let url = self.required(raw.url.as_ref(), &at, "alkis.url", kind);
                let buildings = self.required(
                    raw.buildings_type.as_ref(),
                    &at,
                    "alkis.buildings_type",
                    kind,
                );
                let land_use =
                    self.required(raw.land_use_type.as_ref(), &at, "alkis.land_use_type", kind);
                let wfs = self.wfs(
                    "alkis",
                    raw.name.as_ref(),
                    english.map(|name| format!("{name} ALKIS")),
                    url?,
                    buildings?,
                    land_use?,
                    raw.square_bez.as_ref(),
                    raw.sorted.as_ref().is_none_or(|sorted| *sorted.get_ref()),
                    crs,
                );
                fine.then_some(Alkis::Wfs(wfs?))
            }
            "ogc-api-features" => {
                let kind = "kind `ogc-api-features`";
                let fine = [
                    self.absent(raw.name.as_ref(), "alkis.name", kind),
                    self.absent(raw.url.as_ref(), "alkis.url", kind),
                    self.absent(raw.buildings_type.as_ref(), "alkis.buildings_type", kind),
                    self.absent(raw.land_use_type.as_ref(), "alkis.land_use_type", kind),
                    self.absent(raw.square_bez.as_ref(), "alkis.square_bez", kind),
                    self.absent(raw.sorted.as_ref(), "alkis.sorted", kind),
                ]
                .iter()
                .all(Option::is_some);
                let buildings = self
                    .required(raw.buildings.as_ref(), &at, "alkis.buildings", kind)
                    .and_then(|url| self.url(url, "alkis.buildings"));
                let land_use = self
                    .required(raw.land_use.as_ref(), &at, "alkis.land_use", kind)
                    .and_then(|url| self.url(url, "alkis.land_use"));
                let normalise = match &raw.normalise {
                    Some(preset) => {
                        let normalise = Normalise::from_id(preset.get_ref());
                        if normalise.is_none() {
                            self.fail(
                                "manifest.normalise",
                                &preset.span(),
                                "alkis.normalise",
                                "a preset is `lgb` or `hamburg-ascii`",
                            );
                        }
                        normalise
                    }
                    None => Some(Normalise::Lgb),
                };
                let fallback = match &raw.fallback {
                    Some(fallback) => {
                        if let Some(kind) = fallback
                            .kind
                            .as_ref()
                            .filter(|kind| *kind.get_ref() != "wfs")
                        {
                            self.fail(
                                "manifest.kind",
                                &kind.span(),
                                "alkis.fallback.kind",
                                "a fallback is a `wfs`",
                            );
                            return None;
                        }
                        self.wfs(
                            "alkis.fallback",
                            fallback.name.as_ref(),
                            english.map(|name| format!("{name} ALKIS (WFS)")),
                            &fallback.url,
                            &fallback.buildings_type,
                            &fallback.land_use_type,
                            fallback.square_bez.as_ref(),
                            true,
                            crs,
                        )
                        .map(Some)
                    }
                    None => Some(None),
                };
                fine.then_some(Alkis::OgcApi {
                    buildings: buildings?,
                    land_use: land_use?,
                    crs: crs?,
                    normalise: normalise?,
                    fallback: fallback?,
                })
            }
            _ => {
                self.fail(
                    "manifest.kind",
                    &at,
                    "alkis.kind",
                    "ALKIS comes from a `wfs` or from `ogc-api-features`",
                );
                None
            }
        }
    }

    /// What a table with `kind` and `backend` is: a kind of `kinds`, or the backend `Err(name)`.
    fn kind_of<'v>(
        &mut self,
        kind: Option<&'v Text>,
        backend: Option<&Text>,
        table: &str,
        code: &'static str,
        kinds: &[&str],
    ) -> Option<Result<&'v Text, String>> {
        let name = "the name of a backend: lower-case letters, digits and hyphens";
        match (kind, backend) {
            (Some(kind), Some(_)) if kind.get_ref() != "backend" => {
                self.fail(
                    code,
                    &kind.span(),
                    &format!("{table}.kind"),
                    format!(
                        "a `backend` does the whole of this table; `kind = \"{}\"` contradicts it",
                        kind.get_ref()
                    ),
                );
                None
            }
            (_, Some(backend)) => self
                .matching(
                    backend,
                    &format!("{table}.backend"),
                    "manifest.value",
                    |text| text.len() >= 2 && is_slug(text, 32),
                    name,
                )
                .map(Err),
            (Some(kind), None) if kind.get_ref() == "backend" => {
                self.fail(
                    "manifest.missing-key",
                    &kind.span(),
                    &format!("{table}.backend"),
                    "kind `backend` needs the name of the backend",
                );
                None
            }
            (Some(kind), None) if kinds.contains(&kind.get_ref().as_str()) => Some(Ok(kind)),
            (Some(kind), None) => {
                self.fail(
                    code,
                    &kind.span(),
                    &format!("{table}.kind"),
                    format!("the kind is one of {}, backend", kinds.join(", ")),
                );
                None
            }
            (None, None) => {
                self.fail_whole(
                    "manifest.missing-key",
                    &format!("{table}.kind"),
                    "the table says its `kind` or names a `backend`",
                );
                None
            }
        }
    }

    /// What a raster of the table `table` is called: its `label`, else `default`.
    fn label(&mut self, raw: &RawRaster, table: &str, default: Option<String>) -> Option<String> {
        match &raw.label {
            Some(label) => self.text(label, &format!("{table}.label"), "manifest.value", 80),
            None => default,
        }
    }

    /// The keys of a raster table that `kind` does not have.
    fn absent_of_raster(
        &mut self,
        raw: &RawRaster,
        table: &str,
        kind: &str,
        keys: &[&str],
    ) -> bool {
        let mut fine = true;
        for key in keys {
            let full = format!("{table}.{key}");
            let absent = match *key {
                "label" => self.absent(raw.label.as_ref(), &full, kind),
                "url" => self.absent(raw.url.as_ref(), &full, kind),
                "layer" => self.absent(raw.layer.as_ref(), &full, kind),
                "coverage" => self.absent(raw.coverage.as_ref(), &full, kind),
                "crs" => self.absent(raw.crs.as_ref(), &full, kind),
                "params" => self.absent(raw.params.as_ref(), &full, kind),
                "max_px" => self.absent(raw.max_px.as_ref(), &full, kind),
                "multipart" => self.absent(raw.multipart.as_ref(), &full, kind),
                "envelope" => self.absent(raw.envelope.as_ref(), &full, kind),
                "max_m" => self.absent(raw.max_m.as_ref(), &full, kind),
                "date" => self.absent(raw.date.as_ref(), &full, kind),
                "source_label" => self.absent(raw.source_label.as_ref(), &full, kind),
                other => unreachable!("{other} is no key of a raster table"),
            };
            fine &= absent.is_some();
        }
        fine
    }

    fn orthophoto(
        &mut self,
        raw: &RawRaster,
        adapter: Option<Crs>,
        english: Option<&str>,
    ) -> Option<Orthophoto> {
        let table = "orthophoto";
        let kind = match self.kind_of(
            raw.kind.as_ref(),
            raw.backend.as_ref(),
            table,
            "manifest.kind",
            &["wms", "wcs"],
        )? {
            Ok(kind) => kind,
            Err(name) => {
                let fine = self.absent_of_raster(
                    raw,
                    table,
                    "a backend",
                    &[
                        "label",
                        "url",
                        "layer",
                        "coverage",
                        "crs",
                        "params",
                        "max_px",
                        "multipart",
                        "envelope",
                        "max_m",
                        "date",
                        "source_label",
                    ],
                );
                return fine.then_some(Orthophoto::Backend { name });
            }
        };
        let at = kind.span();
        let wms = kind.get_ref() == "wms";
        let kind = if wms { "kind `wms`" } else { "kind `wcs`" };
        let mut fine = self.absent_of_raster(
            raw,
            table,
            kind,
            &["multipart", "envelope", "max_m", "source_label"],
        );
        let other = if wms { "coverage" } else { "layer" };
        fine &= self.absent_of_raster(raw, table, kind, &[other]);
        if !wms {
            fine &= self.absent_of_raster(raw, table, kind, &["params"]);
        }
        let url = self
            .required(raw.url.as_ref(), &at, "orthophoto.url", kind)
            .and_then(|url| self.url(url, "orthophoto.url"));
        let named = if wms { &raw.layer } else { &raw.coverage };
        let key = format!("{table}.{}", if wms { "layer" } else { "coverage" });
        let named = self
            .required(named.as_ref(), &at, &key, kind)
            .and_then(|name| {
                self.matching(
                    name,
                    &key,
                    "manifest.value",
                    is_layer,
                    "the name of a layer: letters, digits and `_.:-`",
                )
            });
        let crs = self.service_crs(raw.crs.as_ref(), "orthophoto.crs", adapter);
        let params = self.params(raw.params.as_ref(), "orthophoto.params");
        let service = if wms { "WMS" } else { "WCS" };
        let label = self.label(
            raw,
            table,
            english.map(|name| format!("orthophoto ({name} {service})")),
        );
        let max_px = self
            .required(raw.max_px.as_ref(), &at, "orthophoto.max_px", kind)
            .and_then(|value| self.integer(value, "orthophoto.max_px", 256, 10_000))
            .and_then(|value| u32::try_from(value).ok());
        let date = self.date(raw.date.as_ref(), table, wms);
        if !fine {
            return None;
        }
        Some(if wms {
            Orthophoto::Wms {
                label: label?,
                url: url?,
                layer: named?,
                crs: crs?,
                params: params?,
                max_px: max_px?,
                date: date?,
            }
        } else {
            Orthophoto::Wcs {
                label: label?,
                url: url?,
                coverage: named?,
                crs: crs?,
                max_px: max_px?,
                date: date?,
            }
        })
    }

    /// `[terrain]` or `[surface]`.
    fn height_model(
        &mut self,
        raw: &RawRaster,
        table: &str,
        english: Option<&str>,
    ) -> Option<HeightModel> {
        let kind = match self.kind_of(
            raw.kind.as_ref(),
            raw.backend.as_ref(),
            table,
            "manifest.kind",
            &["wcs"],
        )? {
            Ok(kind) => kind,
            Err(name) => {
                let fine = self.absent_of_raster(
                    raw,
                    table,
                    "a backend",
                    &[
                        "label",
                        "url",
                        "layer",
                        "coverage",
                        "crs",
                        "params",
                        "max_px",
                        "multipart",
                        "envelope",
                        "max_m",
                        "date",
                        "source_label",
                    ],
                );
                return fine.then_some(HeightModel::Backend { name });
            }
        };
        let at = kind.span();
        let kind = "kind `wcs` of a height model";
        let mut fine = self.absent_of_raster(raw, table, kind, &["layer", "crs", "max_px"]);
        if table != "terrain" {
            fine &= self.absent_of_raster(raw, table, "this table", &["source_label"]);
        }
        let key = |name: &str| format!("{table}.{name}");
        let url = self
            .required(raw.url.as_ref(), &at, &key("url"), kind)
            .and_then(|url| self.url(url, &key("url")));
        let coverage = self
            .required(raw.coverage.as_ref(), &at, &key("coverage"), kind)
            .and_then(|name| {
                self.matching(
                    name,
                    &key("coverage"),
                    "manifest.value",
                    is_layer,
                    "the name of a coverage: letters, digits and `_.:-`",
                )
            });
        let params = self.params(raw.params.as_ref(), &key("params"));
        let label = self.label(
            raw,
            table,
            english.map(|name| format!("{table} model ({name} WCS)")),
        );
        let envelope = self
            .required(raw.envelope.as_ref(), &at, &key("envelope"), kind)
            .and_then(|value| self.envelope(value, &key("envelope")));
        let max_m = self
            .required(raw.max_m.as_ref(), &at, &key("max_m"), kind)
            .and_then(|value| self.number(value, &key("max_m"), 99.0..20_000.0));
        let date = self.date(raw.date.as_ref(), table, false);
        let source_label = match raw.source_label.as_ref().filter(|_| table == "terrain") {
            Some(label) => self
                .matching(
                    label,
                    &key("source_label"),
                    "manifest.value",
                    |text| text.len() >= 2 && is_slug(text, 40),
                    "a label: lower-case letters, digits and hyphens",
                )
                .map(Some),
            None => Some(None),
        };
        fine.then_some(HeightModel::Wcs {
            label: label?,
            url: url?,
            coverage: coverage?,
            params: params?,
            multipart: raw.multipart.as_ref().is_some_and(|value| *value.get_ref()),
            envelope: envelope?,
            max_m: max_m?,
            date: date?,
            source_label: source_label?,
        })
    }

    /// `date` of the raster table `table`; `wms` says whether the table is a WMS, which alone
    /// can be asked for feature info.
    fn date(
        &mut self,
        raw: Option<&Spanned<RawDate>>,
        table: &str,
        wms: bool,
    ) -> Option<DateSource> {
        let Some(raw) = raw else {
            return Some(DateSource::None);
        };
        let table = format!("{table}.date");
        let at = raw.span();
        let raw = raw.get_ref();
        let key = |name: &str| format!("{table}.{name}");
        let kinds = ["none", "wfs-geojson", "stac-newest", "wms-featureinfo"];
        let Some(kind) = raw.kind.as_ref() else {
            // A table without `kind` has a place, unlike a missing table.
            self.fail(
                "manifest.missing-key",
                &at,
                &key("kind"),
                "a date lookup says its `kind`",
            );
            return None;
        };
        if !kinds.contains(&kind.get_ref().as_str()) {
            self.fail(
                "manifest.date-kind",
                &kind.span(),
                &key("kind"),
                format!("the kind of a date lookup is one of {}", kinds.join(", ")),
            );
            return None;
        }
        let at = kind.span();
        let name = kind.get_ref().as_str();
        let of_kind = format!("kind `{name}`");
        let of_kind = of_kind.as_str();
        // Which of url, layer, field and format the kind has.
        let (has_url, has_layer, has_field, has_format) = match name {
            "wfs-geojson" => (true, true, true, true),
            "stac-newest" => (true, false, false, false),
            "wms-featureinfo" => (false, true, true, false),
            _ => (false, false, false, false),
        };
        if name == "wms-featureinfo" && !wms {
            self.fail(
                "manifest.date-kind",
                &at,
                &key("kind"),
                "`wms-featureinfo` asks the WMS of an orthophoto of kind `wms`",
            );
            return None;
        }
        let mut fine = true;
        let mut pick = |check: &mut Self, value: Option<&Text>, has: bool, name: &str| {
            if has {
                check.required(value, &at, &key(name), of_kind).cloned()
            } else {
                fine &= check.absent(value, &key(name), of_kind).is_some();
                None
            }
        };
        let url = pick(self, raw.url.as_ref(), has_url, "url");
        let layer = pick(self, raw.layer.as_ref(), has_layer, "layer");
        let field = pick(self, raw.field.as_ref(), has_field, "field");
        let format = pick(self, raw.format.as_ref(), has_format, "format");
        let url = url.and_then(|url| self.url(&url, &key("url")));
        let layer = layer.and_then(|layer| {
            let (rule, expected): (fn(&str) -> bool, _) = if name == "wfs-geojson" {
                (is_type_name, "the name of a feature type")
            } else {
                (is_layer, "the name of a layer: letters, digits and `_.:-`")
            };
            self.matching(&layer, &key("layer"), "manifest.value", rule, expected)
        });
        let field = field.and_then(|field| {
            self.matching(
                &field,
                &key("field"),
                "manifest.value",
                is_field,
                "the name of a field",
            )
        });
        let format = format.and_then(|format| match format.get_ref().as_str() {
            "DD.MM.YYYY" => Some(DateFormat::DayMonthYear),
            "YYYY-MM-DD" => Some(DateFormat::Iso),
            _ => {
                self.fail(
                    "manifest.value",
                    &format.span(),
                    &key("format"),
                    "a date format is `DD.MM.YYYY` or `YYYY-MM-DD`",
                );
                None
            }
        });
        if !fine {
            return None;
        }
        Some(match name {
            "wfs-geojson" => DateSource::WfsGeoJson {
                url: url?,
                layer: layer?,
                field: field?,
                format: format?,
            },
            "stac-newest" => DateSource::StacNewest { url: url? },
            "wms-featureinfo" => DateSource::WmsFeatureInfo {
                layer: layer?,
                field: field?,
            },
            _ => DateSource::None,
        })
    }

    fn nas_services(&mut self, raw: &[RawNas], adapter: Option<Crs>) -> Option<Vec<Nas>> {
        let mut fine = true;
        if raw.len() > 1 {
            self.fail(
                "manifest.value",
                &raw[1].name.span(),
                "extra.nas",
                "a region has one NAS service; this is a second one",
            );
            fine = false;
        }
        let mut services = Vec::new();
        for service in raw {
            let name = self.text(&service.name, "extra.nas.name", "manifest.value", 80);
            let url = self.url(&service.url, "extra.nas.url");
            let crs = self.service_crs(service.crs.as_ref(), "extra.nas.crs", adapter);
            let max_count = self.integer(&service.max_count, "extra.nas.max_count", 1, 1_000_000);
            let max_bytes = self.integer(
                &service.max_bytes,
                "extra.nas.max_bytes",
                1,
                MAX_ANSWER_BYTES,
            );
            let mut layers = Some(Vec::new());
            for layer in service.layers.iter().flatten() {
                let layer = self.matching(
                    layer,
                    "extra.nas.layers",
                    "manifest.value",
                    |text| NAS_LAYERS.iter().any(|known| known.file == text),
                    &format!(
                        "a NAS layer; the layers are {}",
                        NAS_LAYERS
                            .iter()
                            .map(|known| known.file)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
                layers = layers.zip(layer).map(|(mut all, layer)| {
                    all.push(layer);
                    all
                });
            }
            let namespaces = match &service.namespaces {
                Some(namespaces) => self
                    .matching(
                        namespaces,
                        "extra.nas.namespaces",
                        "manifest.value",
                        is_namespaces,
                        "a list of namespaces: letters, digits and `_.:/(),=-`",
                    )
                    .map(Some),
                None => Some(None),
            };
            let built = (|| {
                Some(Nas {
                    name: name?,
                    url: url?,
                    crs: crs?,
                    max_count: u32::try_from(max_count?).ok()?,
                    max_bytes: u64::try_from(max_bytes?).ok()?,
                    layers: match &service.layers {
                        Some(_) => Some(layers.clone()?),
                        None => None,
                    },
                    namespaces: namespaces?,
                })
            })();
            match built {
                Some(built) => services.push(built),
                None => fine = false,
            }
        }
        fine.then_some(services)
    }

    fn tree_cadastres(&mut self, raw: &[RawTrees], adapter: Option<Crs>) -> Option<Vec<Trees>> {
        let mut fine = true;
        if raw.len() > 32 {
            self.fail(
                "manifest.value",
                &raw[32].id.span(),
                "extra.trees",
                "more than 32 tree cadastres",
            );
            fine = false;
        }
        let mut cadastres: Vec<Trees> = Vec::new();
        for trees in raw {
            let id = self.matching(
                &trees.id,
                "extra.trees.id",
                "manifest.value",
                |text| text.len() >= 2 && is_slug(text, 32),
                "the id of a tree cadastre: lower-case letters, digits and hyphens",
            );
            let name = self.text(&trees.name, "extra.trees.name", "manifest.value", 80);
            let crs = self.service_crs(trees.crs.as_ref(), "extra.trees.crs", adapter);
            let envelope = self.envelope(&trees.envelope, "extra.trees.envelope");
            if trees.format.get_ref() != "ods-geojson-export" {
                self.fail(
                    "manifest.kind",
                    &trees.format.span(),
                    "extra.trees.format",
                    "a tree cadastre is read as `ods-geojson-export`",
                );
                fine = false;
            }
            let field = "the name of a field";
            let geo_field = self.matching(
                &trees.geo_field,
                "extra.trees.geo_field",
                "manifest.value",
                is_field,
                field,
            );
            let mut endpoints = Some(Vec::new());
            if !(1..=8).contains(&trees.endpoints.len()) {
                self.fail(
                    "manifest.value",
                    &trees.id.span(),
                    "extra.trees.endpoints",
                    "a tree cadastre has 1 to 8 endpoints",
                );
                endpoints = None;
            }
            for endpoint in &trees.endpoints {
                let url = self.url(endpoint, "extra.trees.endpoints");
                endpoints = endpoints.zip(url).map(|(mut all, url)| {
                    all.push(url);
                    all
                });
            }
            let name_of = |check: &mut Self, value: &Text, key: &str| {
                check.matching(
                    value,
                    &format!("extra.trees.fields.{key}"),
                    "manifest.value",
                    is_field,
                    field,
                )
            };
            let kind = name_of(self, &trees.fields.kind, "kind");
            let tree_id = name_of(self, &trees.fields.id, "id");
            let date = name_of(self, &trees.fields.date, "date");
            let species = match &trees.fields.species {
                Some(species) => name_of(self, species, "species").map(Some),
                None => Some(None),
            };
            let crown = match &trees.fields.crown {
                Some(crown) => name_of(self, crown, "crown").map(Some),
                None => Some(None),
            };
            let mut skipped_kinds = Some(Vec::new());
            if trees.skipped_kinds.len() > 16 {
                self.fail(
                    "manifest.value",
                    &trees.skipped_kinds[16].span(),
                    "extra.trees.skipped_kinds",
                    "more than 16 kinds",
                );
                skipped_kinds = None;
            }
            for kind in &trees.skipped_kinds {
                let kind = self.text(kind, "extra.trees.skipped_kinds", "manifest.value", 80);
                skipped_kinds = skipped_kinds.zip(kind).map(|(mut all, kind)| {
                    all.push(kind);
                    all
                });
            }
            let attribution = self.credit(&trees.attribution, "extra.trees.attribution", 400);
            let max_bytes = self.integer(
                &trees.max_bytes,
                "extra.trees.max_bytes",
                1,
                MAX_ANSWER_BYTES,
            );
            let built = (|| {
                Some(Trees {
                    id: id?,
                    name: name?,
                    crs: crs?,
                    envelope: envelope?,
                    geo_field: geo_field?,
                    endpoints: endpoints?,
                    fields: TreeFields {
                        kind: kind?,
                        id: tree_id?,
                        date: date?,
                        species: species?,
                        crown: crown?,
                    },
                    skipped_kinds: skipped_kinds?,
                    attribution: attribution?,
                    max_bytes: u64::try_from(max_bytes?).ok()?,
                })
            })();
            match built {
                Some(built) => {
                    if cadastres.iter().any(|other| other.id == built.id) {
                        self.fail(
                            "manifest.value",
                            &trees.id.span(),
                            "extra.trees.id",
                            format!("two tree cadastres have the id `{}`", built.id),
                        );
                        fine = false;
                    }
                    cadastres.push(built);
                }
                None => fine = false,
            }
        }
        fine.then_some(cadastres)
    }

    fn preview(&mut self, raw: &RawPreview) -> Option<Preview> {
        let wms = self.url(&raw.wms, "preview.wms");
        let layer = self.matching(
            &raw.layer,
            "preview.layer",
            "manifest.value",
            is_layer,
            "the name of a layer: letters, digits and `_.:-`",
        );
        let format = self.matching(
            &raw.format,
            "preview.format",
            "manifest.value",
            |text| ["image/png", "image/jpeg"].contains(&text),
            "`image/png` or `image/jpeg`",
        );
        let params = self.params(raw.params.as_ref(), "preview.params");
        let attribution = self.credit(&raw.attribution, "preview.attribution", 200);
        let agency = self.text(&raw.agency, "preview.agency", "manifest.value", 40);
        Some(Preview {
            wms: wms?,
            layer: layer?,
            format: format?,
            params: params?,
            attribution: attribution?,
            agency: agency?,
        })
    }

    fn estimate(&mut self, raw: &RawEstimate) -> Option<Estimate> {
        let size = |check: &mut Self, value: Option<&Spanned<f64>>, key: &str| match value {
            Some(value) => check
                .number(value, &format!("estimate.{key}"), 0.0..10_000.0)
                .map(Some),
            None => Some(None),
        };
        let orthophoto = size(
            self,
            raw.orthophoto_mb_per_km2.as_ref(),
            "orthophoto_mb_per_km2",
        );
        let terrain = size(self, raw.terrain_mb_per_km2.as_ref(), "terrain_mb_per_km2");
        let surface = size(self, raw.surface_mb_per_km2.as_ref(), "surface_mb_per_km2");
        Some(Estimate {
            orthophoto_mb_per_km2: orthophoto?,
            terrain_mb_per_km2: terrain?,
            surface_mb_per_km2: surface?,
        })
    }
}
