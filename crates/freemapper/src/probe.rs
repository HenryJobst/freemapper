// SPDX-License-Identifier: AGPL-3.0-or-later

//! The reachability probe (`freemapper adapters probe`, issue #286): one small request per
//! service the manifests of the built-in adapters name, and the Geofabrik extracts, so that an
//! outage, a maintenance or a move shows before a person runs into it.
//!
//! The request goes through the fetch layer, so the host rules and the fixed `User-Agent` apply
//! as they do for a run. Files (the OSM extracts, CSV and GeoJSON exports) get a HEAD; a map
//! service gets its capabilities and an OGC API one feature: many
//! servers answer a HEAD with an error that a real request does not get. Reachable does not mean the data is right; the golden runs
//! check that.

use std::collections::BTreeMap;
use std::time::Duration;

use freemapper_adapter_api::policy::{PolicyFetcher, UrlPolicy};

use crate::clock;
use crate::fetch::{FetchError, Fetcher};

/// A service that answers later than this counts as slow.
pub const SLOW: Duration = Duration::from_secs(10);

/// The host that serves the OSM extracts.
const GEOFABRIK: &str = "download.geofabrik.de";

/// One service to ask and the adapters that use it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub url: String,
    pub adapters: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    Slow,
    /// The service sent the request somewhere the adapter does not name.
    Moved,
    Error,
}

impl Status {
    pub fn code(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Slow => "slow",
            Self::Moved => "moved",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub url: String,
    pub host: String,
    pub adapters: Vec<String>,
    pub status: Status,
    /// What went wrong, or a remark on an `ok`.
    pub detail: Option<String>,
    pub millis: u64,
}

/// The services of the manifests `texts`, each URL once, with the adapters that name it, and
/// every host the manifests declare plus the Geofabrik host. A text that does not parse is
/// skipped: the loader reports it.
pub fn targets(texts: &[&str]) -> (Vec<Target>, Vec<String>) {
    let mut by_url: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut hosts = vec![GEOFABRIK.to_string()];
    for text in texts {
        let Ok(manifest) = text.parse::<toml::Table>() else {
            continue;
        };
        let id = manifest
            .get("adapter")
            .and_then(|a| a.get("id"))
            .and_then(toml::Value::as_str)
            .unwrap_or("?")
            .to_string();
        let mut urls = Vec::new();
        collect_urls(&toml::Value::Table(manifest.clone()), &mut urls);
        if let Some(extracts) = manifest
            .get("osm")
            .and_then(|o| o.get("extract"))
            .and_then(toml::Value::as_array)
        {
            for path in extracts
                .iter()
                .filter_map(|e| e.get("geofabrik").and_then(toml::Value::as_str))
            {
                // The extract sits in a folder of its own name, as the fetch of a run asks for it.
                let name = path.rsplit('/').next().unwrap_or(path);
                urls.push(format!("https://{GEOFABRIK}/{path}/{name}-latest.osm.pbf"));
            }
        }
        for url in urls {
            let adapters = by_url.entry(url).or_default();
            if !adapters.contains(&id) {
                adapters.push(id.clone());
            }
        }
        if let Some(listed) = manifest
            .get("net")
            .and_then(|n| n.get("hosts"))
            .and_then(toml::Value::as_array)
        {
            hosts.extend(listed.iter().filter_map(|h| h.as_str().map(str::to_string)));
        }
    }
    hosts.sort();
    hosts.dedup();
    let targets = by_url
        .into_iter()
        .map(|(url, adapters)| Target { url, adapters })
        .collect();
    (targets, hosts)
}

/// The services of the built-in adapters. The Polish adapter declares its addresses in code and
/// joins when they move into manifests (issue #273).
pub fn builtin_targets() -> (Vec<Target>, Vec<String>) {
    targets(freemapper_adapters_de::states::manifest_texts())
}

/// Every `https://` string in `value`; a template with `{…}` is no address to ask.
fn collect_urls(value: &toml::Value, urls: &mut Vec<String>) {
    match value {
        toml::Value::String(s) if s.starts_with("https://") && !s.contains(['{', '}']) => {
            urls.push(s.clone());
        }
        toml::Value::Array(items) => items.iter().for_each(|v| collect_urls(v, urls)),
        toml::Value::Table(table) => table.values().for_each(|v| collect_urls(v, urls)),
        _ => {}
    }
}

/// Asks every target once, within the hosts `hosts` and in the order given.
pub fn probe(fetcher: &dyn Fetcher, targets: &[Target], hosts: &[String]) -> Vec<Outcome> {
    let policy = UrlPolicy::new(hosts.iter().map(String::as_str));
    let bound = policy.as_ref().ok().map(|p| PolicyFetcher::new(fetcher, p));
    let fetcher: &dyn Fetcher = match &bound {
        Some(bound) => bound,
        None => fetcher,
    };
    targets
        .iter()
        .map(|target| {
            let started = clock::now();
            let answer = match request_for(&target.url) {
                Request::Head(url) => fetcher.last_modified(&url).map(|_| ()),
                Request::Get(url) => fetcher.get(&url).map(|_| ()),
            };
            outcome(
                target,
                answer,
                clock::now().duration_since(started).unwrap_or_default(),
            )
        })
        .collect()
}

/// What to ask for a service address of a manifest.
#[derive(Debug, PartialEq, Eq)]
enum Request {
    Head(String),
    Get(String),
}

fn request_for(url: &str) -> Request {
    let lower = url.to_ascii_lowercase();
    // A fixed query that selects the service (`?mapfile=glb`) stays; the question follows it.
    if lower.contains('?') && lower.contains("service=") {
        return Request::Get(url.to_string());
    }
    if lower.ends_with("/items") {
        return Request::Get(format!("{url}?limit=1"));
    }
    if lower.ends_with(".pbf")
        || lower.ends_with(".csv")
        || lower.ends_with(".zip")
        || lower.contains("/exports/")
    {
        return Request::Head(url.to_string());
    }
    let service = if lower.contains("wms") {
        "WMS"
    } else if lower.contains("wcs") {
        "WCS"
    } else {
        "WFS"
    };
    let joint = if lower.contains('?') { '&' } else { '?' };
    Request::Get(format!(
        "{url}{joint}service={service}&request=GetCapabilities"
    ))
}

fn outcome(target: &Target, answer: Result<(), FetchError>, took: Duration) -> Outcome {
    let (status, detail) = match answer {
        Ok(()) if took > SLOW => (Status::Slow, Some(format!("{} s", took.as_secs()))),
        Ok(()) => (Status::Ok, None),
        // The server is there and answers; it only refuses a HEAD or a request without a box.
        Err(FetchError::Refused { .. }) => (
            Status::Ok,
            Some("answers, but refuses the probe request".to_string()),
        ),
        Err(error @ (FetchError::HostNotNamed(_) | FetchError::NotAllowed { .. })) => {
            (Status::Moved, Some(error.to_string()))
        }
        Err(FetchError::NotFound(_)) => (Status::Error, Some("404 not found".to_string())),
        // The bare message: the probe asks once, so the text about repeated asking does not fit.
        Err(FetchError::Http { message, .. }) => (Status::Error, Some(message)),
        Err(error) => (Status::Error, Some(error.to_string())),
    };
    Outcome {
        url: target.url.clone(),
        host: host_of(&target.url),
        adapters: target.adapters.clone(),
        status,
        detail,
        millis: took.as_millis() as u64,
    }
}

fn host_of(url: &str) -> String {
    url.trim_start_matches("https://")
        .split(['/', '?'])
        .next()
        .unwrap_or_default()
        .to_string()
}

/// The result as JSON: `{"results": [{url, host, adapters, status, detail, millis}]}`.
pub fn to_json(outcomes: &[Outcome]) -> serde_json::Value {
    serde_json::json!({
        "results": outcomes.iter().map(|o| serde_json::json!({
            "url": o.url,
            "host": o.host,
            "adapters": o.adapters,
            "status": o.status.code(),
            "detail": o.detail,
            "millis": o.millis,
        })).collect::<Vec<_>>(),
    })
}

/// One line per service: status, time, adapters, URL and the detail.
pub fn lines(outcomes: &[Outcome]) -> Vec<String> {
    outcomes
        .iter()
        .map(|o| {
            let detail = o
                .detail
                .as_deref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default();
            format!(
                "{:<5} {:>6} ms  {}  {}{detail}",
                o.status.code(),
                o.millis,
                o.adapters.join(","),
                o.url
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Answers by URL with the error it is given, or with success.
    struct Scripted(RefCell<BTreeMap<String, FetchError>>);

    impl Fetcher for Scripted {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            let asked = url.split('?').next().unwrap_or(url);
            match self.0.borrow_mut().remove(asked) {
                Some(error) => Err(error),
                None => Ok(Vec::new()),
            }
        }

        fn last_modified(&self, url: &str) -> Result<Option<chrono::NaiveDate>, FetchError> {
            match self.0.borrow_mut().remove(url) {
                Some(error) => Err(error),
                None => Ok(None),
            }
        }
    }

    const MANIFEST: &str = r#"
[adapter]
id = "demo"
[net]
hosts = ["a.example.org", "b.example.org"]
[[osm.extract]]
geofabrik = "europe/germany/demo"
[alkis]
buildings = "https://a.example.org/items"
template = "https://a.example.org/{z}/{x}"
[[extra.trees]]
endpoints = ["https://b.example.org/t", "https://a.example.org/items"]
"#;

    #[test]
    fn targets_list_every_service_once() {
        let (targets, hosts) = targets(&[MANIFEST, "not toml ["]);
        let urls: Vec<_> = targets.iter().map(|t| t.url.as_str()).collect();
        assert_eq!(
            urls,
            [
                "https://a.example.org/items",
                "https://b.example.org/t",
                "https://download.geofabrik.de/europe/germany/demo/demo-latest.osm.pbf",
            ]
        );
        assert!(targets.iter().all(|t| t.adapters == ["demo"]));
        assert_eq!(
            hosts,
            ["a.example.org", "b.example.org", "download.geofabrik.de"]
        );
    }

    #[test]
    fn an_adapter_is_named_once_per_shared_service() {
        let other = MANIFEST.replace("demo\"\n[net]", "other\"\n[net]");
        let (targets, _) = targets(&[MANIFEST, &other]);
        let shared = targets
            .iter()
            .find(|t| t.url == "https://a.example.org/items")
            .unwrap();
        assert_eq!(shared.adapters, ["demo", "other"]);
    }

    #[test]
    fn every_answer_has_its_status() {
        let (targets, hosts) = targets(&[MANIFEST]);
        let errors = BTreeMap::from([
            (
                "https://a.example.org/items".to_string(),
                FetchError::NotFound("x".into()),
            ),
            (
                "https://b.example.org/t".to_string(),
                FetchError::Http {
                    url: "https://b.example.org/t".into(),
                    message: "http status: 503".into(),
                },
            ),
        ]);
        let outcomes = probe(&Scripted(RefCell::new(errors)), &targets, &hosts);
        let status: Vec<_> = outcomes.iter().map(|o| o.status).collect();
        assert_eq!(status, [Status::Error, Status::Error, Status::Ok]);
        assert_eq!(outcomes[0].detail.as_deref(), Some("404 not found"));
        assert!(outcomes[1].detail.as_deref().unwrap().contains("503"));
        assert_eq!(outcomes[2].host, "download.geofabrik.de");
    }

    #[test]
    fn a_slow_answer_and_a_refused_head_are_told_apart() {
        let target = Target {
            url: "https://a.example.org/x".into(),
            adapters: vec!["demo".into()],
        };
        let slow = outcome(&target, Ok(()), SLOW + Duration::from_secs(1));
        assert_eq!(slow.status, Status::Slow);
        // Exactly as long as the limit is still in time.
        assert_eq!(outcome(&target, Ok(()), SLOW).status, Status::Ok);
        let refused = outcome(
            &target,
            Err(FetchError::Refused {
                url: target.url.clone(),
                status: 405,
                detail: None,
            }),
            Duration::ZERO,
        );
        assert_eq!(refused.status, Status::Ok);
        assert!(refused.detail.is_some());
    }

    #[test]
    fn the_json_and_the_lines_carry_the_same_facts() {
        let (targets, hosts) = targets(&[MANIFEST]);
        let outcomes = probe(&Scripted(RefCell::default()), &targets, &hosts);
        let json = to_json(&outcomes);
        assert_eq!(json["results"].as_array().unwrap().len(), 3);
        assert_eq!(json["results"][0]["status"], "ok");
        assert!(lines(&outcomes)[0].starts_with("ok"));
    }

    #[test]
    fn each_kind_of_address_is_asked_the_way_it_answers() {
        let get = |u: &str| Request::Get(u.to_string());
        assert_eq!(
            request_for("https://h.de/dienste/alkis_wfs_nas"),
            get("https://h.de/dienste/alkis_wfs_nas?service=WFS&request=GetCapabilities")
        );
        assert_eq!(
            request_for("https://h.de/wms_dop"),
            get("https://h.de/wms_dop?service=WMS&request=GetCapabilities")
        );
        assert_eq!(
            request_for("https://h.de/ows_WCS_ST_DOM1"),
            get("https://h.de/ows_WCS_ST_DOM1?service=WCS&request=GetCapabilities")
        );
        assert_eq!(
            request_for("https://h.de/v1/collections/a/items"),
            get("https://h.de/v1/collections/a/items?limit=1")
        );
        assert_eq!(
            request_for("https://h.de/x?a=1"),
            get("https://h.de/x?a=1&service=WFS&request=GetCapabilities")
        );
        assert_eq!(
            request_for("https://h.de/x?service=WFS&request=GetCapabilities"),
            get("https://h.de/x?service=WFS&request=GetCapabilities")
        );
        let head = |u: &str| Request::Head(u.to_string());
        assert_eq!(
            request_for("https://h.de/b/c-latest.osm.pbf"),
            head("https://h.de/b/c-latest.osm.pbf")
        );
        assert_eq!(
            request_for("https://h.de/d/exports/geojson"),
            head("https://h.de/d/exports/geojson")
        );
        assert_eq!(
            request_for("https://h.de/baum.csv"),
            head("https://h.de/baum.csv")
        );
    }

    #[test]
    fn every_builtin_state_has_services_and_an_extract_to_probe() {
        let (targets, hosts) = builtin_targets();
        for state in freemapper_adapters_de::states::State::ALL {
            let id = state.id();
            assert!(
                targets.iter().any(|t| t.adapters.iter().any(|a| a == id)
                    && t.url.starts_with("https://download.geofabrik.de/")),
                "{id} has no extract to probe"
            );
        }
        assert!(targets.iter().all(|t| hosts.contains(&host_of(&t.url))));
    }
}
