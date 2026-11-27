// SPDX-License-Identifier: AGPL-3.0-or-later

//! The status of the services over the nights (`freemapper adapters probe --status`, ADR 0186,
//! issue #286).
//!
//! A service counts as down only after [`NIGHTS`] nights in a row in which it failed or moved:
//! one bad night can be a maintenance window or a datacentre the agency blocks. The nightly
//! workflow keeps the result on the branch `status`; the website reads it from there.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::probe::{Outcome, Status as Answer};

/// Bad nights in a row after which a service counts as down.
pub const NIGHTS: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Status {
    /// The day of the probe, `YYYY-MM-DD`.
    pub updated: String,
    pub services: BTreeMap<String, Service>,
    /// The addresses that are down, by adapter, for the website.
    #[serde(default)]
    pub down_by_adapter: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    pub host: String,
    pub adapters: Vec<String>,
    /// What the last probe said: `ok`, `slow`, `moved` or `error`.
    pub status: String,
    pub detail: Option<String>,
    #[serde(default)]
    pub bad_nights: u32,
    #[serde(default)]
    pub down: bool,
    #[serde(default)]
    pub down_since: Option<String>,
    /// Whether `down` differs from the night before.
    #[serde(default)]
    pub changed: bool,
}

/// The status after the probe `outcomes` of the day `date`, given the status of the night before.
pub fn update(outcomes: &[Outcome], previous: &Status, date: &str) -> Status {
    let services: BTreeMap<String, Service> = outcomes
        .iter()
        .map(|outcome| {
            let before = previous.services.get(&outcome.url);
            let bad = matches!(outcome.status, Answer::Error | Answer::Moved);
            let bad_nights = if bad {
                before.map_or(0, |b| b.bad_nights) + 1
            } else {
                0
            };
            let down = bad_nights >= NIGHTS;
            let down_since = down.then(|| {
                before
                    .and_then(|b| b.down_since.clone())
                    .unwrap_or_else(|| date.to_string())
            });
            let service = Service {
                host: outcome.host.clone(),
                adapters: outcome.adapters.clone(),
                status: outcome.status.code().to_string(),
                detail: outcome.detail.clone(),
                bad_nights,
                down,
                down_since,
                changed: down != before.is_some_and(|b| b.down),
            };
            (outcome.url.clone(), service)
        })
        .collect();
    let mut down_by_adapter: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (url, service) in services.iter().filter(|(_, s)| s.down) {
        for adapter in &service.adapters {
            down_by_adapter
                .entry(adapter.clone())
                .or_default()
                .push(url.clone());
        }
    }
    Status {
        updated: date.to_string(),
        services,
        down_by_adapter,
    }
}

/// The changes of the night as lines, `down <url>` or `up <url>`.
pub fn transitions(status: &Status) -> Vec<String> {
    status
        .services
        .iter()
        .filter(|(_, service)| service.changed)
        .map(|(url, service)| format!("{} {url}", if service.down { "down" } else { "up" }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://a.example.org/wfs";

    fn night(previous: &Status, answer: Answer, date: &str) -> Status {
        let outcome = Outcome {
            url: URL.to_string(),
            host: "a.example.org".to_string(),
            adapters: vec!["demo".to_string()],
            status: answer,
            detail: None,
            millis: 5,
        };
        update(&[outcome], previous, date)
    }

    #[test]
    fn one_bad_night_is_no_outage() {
        let first = night(&Status::default(), Answer::Error, "2026-11-23");
        assert!(!first.services[URL].down);
        assert_eq!(first.services[URL].bad_nights, 1);
        assert!(transitions(&first).is_empty());
    }

    #[test]
    fn two_bad_nights_in_a_row_are_an_outage() {
        let first = night(&Status::default(), Answer::Error, "2026-11-23");
        let second = night(&first, Answer::Moved, "2026-11-24");
        assert!(second.services[URL].down);
        assert_eq!(
            second.services[URL].down_since.as_deref(),
            Some("2026-11-24")
        );
        assert_eq!(transitions(&second), [format!("down {URL}")]);
        assert_eq!(second.down_by_adapter["demo"], [URL]);
    }

    #[test]
    fn an_outage_is_reported_once_and_keeps_its_start() {
        let mut status = Status::default();
        for date in ["2026-11-23", "2026-11-24", "2026-11-25"] {
            status = night(&status, Answer::Error, date);
        }
        assert!(transitions(&status).is_empty());
        assert_eq!(
            status.services[URL].down_since.as_deref(),
            Some("2026-11-24")
        );
        assert_eq!(status.services[URL].bad_nights, 3);
    }

    #[test]
    fn a_good_night_in_between_resets_the_count() {
        let first = night(&Status::default(), Answer::Error, "d1");
        let second = night(&first, Answer::Ok, "d2");
        let third = night(&second, Answer::Error, "d3");
        assert!(!third.services[URL].down);
        assert_eq!(third.services[URL].bad_nights, 1);
    }

    #[test]
    fn an_answer_ends_the_outage() {
        let down = night(
            &night(&Status::default(), Answer::Error, "d1"),
            Answer::Error,
            "d2",
        );
        let up = night(&down, Answer::Ok, "d3");
        assert_eq!(transitions(&up), [format!("up {URL}")]);
        assert!(up.down_by_adapter.is_empty());
        assert_eq!(up.services[URL].down_since, None);
    }

    #[test]
    fn a_slow_service_is_not_down() {
        let first = night(&Status::default(), Answer::Slow, "d1");
        let second = night(&first, Answer::Slow, "d2");
        assert!(!second.services[URL].down);
        assert_eq!(second.services[URL].bad_nights, 0);
    }

    #[test]
    fn the_status_survives_a_round_trip_through_json() {
        let status = night(
            &night(&Status::default(), Answer::Error, "d1"),
            Answer::Error,
            "d2",
        );
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(serde_json::from_str::<Status>(&json).unwrap(), status);
        // A status written before a field existed still reads.
        let old = r#"{"updated":"d0","services":{}}"#;
        assert_eq!(
            serde_json::from_str::<Status>(old).unwrap().services.len(),
            0
        );
    }
}
