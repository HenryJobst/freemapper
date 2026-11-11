//! `freemapper find`: the OpenStreetMap places for a name, the search the MCP tool `find_place`
//! runs, so an agent on the command line can show the matches and pass one to `init --osm`.

use freemapper::fetch::{Fetcher, HttpFetcher};
use freemapper::places::{PlaceCandidate, find_places};
use serde_json::{Value, json};

use crate::text::{Lang, TEXTS, v};

/// One match as JSON, in the shape of the MCP tool `find_place`.
pub fn place_json(place: &PlaceCandidate) -> Value {
    json!({
        "osm_id": place.osm_id,
        "name": place.name,
        "display_name": place.display_name,
        "category": place.category,
        "kind": place.kind,
        "has_area": place.has_area,
    })
}

/// Searches `query` with one Nominatim request and prints the matches in the language of the run.
pub fn run(query: &str, json: bool) -> anyhow::Result<()> {
    crate::nominatim::wait_turn_blocking();
    println!(
        "{}",
        search(fetcher().as_ref(), query, json, &crate::text::lang())?
    );
    Ok(())
}

/// The output of `find` for `query` in `lang`; nothing found is an error.
fn search(fetcher: &dyn Fetcher, query: &str, json: bool, lang: &Lang) -> anyhow::Result<String> {
    let places = find_places(fetcher, query)?;
    if places.is_empty() {
        anyhow::bail!(TEXTS.text(lang, "cli-find-nothing", &[("query", v(query))]));
    }
    if json {
        return Ok(Value::from(places.iter().map(place_json).collect::<Vec<_>>()).to_string());
    }
    let mut lines: Vec<String> = places.iter().map(|place| line(place, lang)).collect();
    lines.push(match places.iter().find(|place| place.has_area) {
        Some(place) => TEXTS.text(lang, "cli-find-hint", &[("id", v(&place.osm_id))]),
        None => TEXTS.text(lang, "cli-find-no-area", &[]),
    });
    Ok(lines.join("\n"))
}

/// One match: id, name, OSM tag and where it is. The name is not repeated from the start of the
/// display name; a place without a name shows the first part of its display name instead.
fn line(place: &PlaceCandidate, lang: &Lang) -> String {
    let display = place.display_name.as_str();
    let name = if place.name.is_empty() {
        display.split(", ").next().unwrap_or_default()
    } else {
        place.name.as_str()
    };
    let rest = display
        .strip_prefix(name)
        .and_then(|rest| rest.strip_prefix(", "))
        .unwrap_or(display);
    let id = if place.has_area {
        "cli-find-place"
    } else {
        "cli-find-place-no-area"
    };
    TEXTS.text(
        lang,
        id,
        &[
            ("id", v(&place.osm_id)),
            ("name", v(name)),
            ("category", v(&place.category)),
            ("kind", v(&place.kind)),
            ("place", v(rest)),
        ],
    )
}

/// Nominatim, or in the CLI tests the answer in the file `FREEMAPPER_FIND_TEST_ANSWER` names.
fn fetcher() -> Box<dyn Fetcher> {
    #[cfg(all(debug_assertions, feature = "test-stub"))]
    if let Some(path) = std::env::var_os("FREEMAPPER_FIND_TEST_ANSWER") {
        return Box::new(stub::AnswerFile(path.into()));
    }
    Box::new(HttpFetcher::new())
}

/// Tests only: a search answer from a file. Read only by a debug build with the `test-stub`
/// feature; any other build has no such switch.
#[cfg(all(debug_assertions, feature = "test-stub"))]
mod stub {
    use std::path::PathBuf;

    use freemapper::fetch::{FetchError, Fetcher};

    pub struct AnswerFile(pub PathBuf);

    impl Fetcher for AnswerFile {
        fn get(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
            std::fs::read(&self.0).map_err(|e| FetchError::Io(self.0.clone(), e))
        }
    }
}

#[cfg(test)]
mod tests {
    use freemapper::fetch::FetchError;
    use freemapper::places::search_url;

    use super::*;

    const SEARCH: &str = include_str!("../../../tests/fixtures/nominatim-search.json");
    const MIXED: &str = include_str!("../../../tests/fixtures/nominatim-search-mixed.json");
    const QUERY: &str = "Clara-Zetkin-Schule Strausberg";

    /// Answers the search URL of [`QUERY`] with `answer`; any other URL fails.
    struct Answer(&'static str);

    impl Fetcher for Answer {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            if url == search_url(QUERY) {
                Ok(self.0.as_bytes().to_vec())
            } else {
                Err(FetchError::NotFound(url.to_owned()))
            }
        }
    }

    fn english() -> Lang {
        Lang::english()
    }

    fn german() -> Lang {
        TEXTS.parse_lang_arg("de").unwrap()
    }

    #[test]
    fn areas_come_first_and_a_point_is_marked() {
        let out = search(&Answer(MIXED), QUERY, false, &english()).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines,
            [
                "way/992423119  Clara-Zetkin-Schule (amenity=school): 1, Am Sportpark, \
                 Strausberg Vorstadt, Strausberg, Märkisch-Oderland, Brandenburg, 15344, Deutschland",
                "node/2147483647  Clara-Zetkin-Schule (highway=bus_stop; no area, cannot be a map \
                 area): Am Sportpark, Strausberg Vorstadt, Strausberg, Märkisch-Oderland, \
                 Brandenburg, 15344, Deutschland",
                "Create the project with: freemapper init <dir> --osm way/992423119",
            ]
        );
    }

    #[test]
    fn json_has_the_fields_of_find_place() {
        let out = search(&Answer(MIXED), QUERY, true, &english()).unwrap();
        let places: Vec<Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(places.len(), 2);
        for place in &places {
            let mut fields: Vec<&str> = place
                .as_object()
                .unwrap()
                .keys()
                .map(|k| k.as_str())
                .collect();
            fields.sort_unstable();
            assert_eq!(
                fields,
                [
                    "category",
                    "display_name",
                    "has_area",
                    "kind",
                    "name",
                    "osm_id"
                ]
            );
        }
        assert_eq!(places[0]["osm_id"], "way/992423119");
        assert_eq!(places[0]["has_area"], true);
        assert_eq!(places[1]["has_area"], false);
        // No hint line: the output is the JSON array alone.
        assert!(out.starts_with('[') && out.ends_with(']'), "{out}");
    }

    #[test]
    fn the_hint_names_the_first_area() {
        let out = search(&Answer(SEARCH), QUERY, false, &english()).unwrap();
        assert_eq!(
            out.lines().last(),
            Some("Create the project with: freemapper init <dir> --osm way/992423119")
        );
    }

    #[test]
    fn without_an_area_the_hint_says_so() {
        const POINTS: &str = r#"[{"osm_type":"node","osm_id":7,"name":"","display_name":"Hydrant, Strausberg","category":"emergency","type":"fire_hydrant","geojson":{"type":"Point","coordinates":[13.8,52.5]}}]"#;
        let out = search(&Answer(POINTS), QUERY, false, &english()).unwrap();
        assert_eq!(
            out,
            "node/7  Hydrant (emergency=fire_hydrant; no area, cannot be a map area): Strausberg\n\
             None of these places has an area that can be a map area. Draw the area with: \
             freemapper select <dir>"
        );
    }

    #[test]
    fn nothing_found_is_an_error() {
        for json in [false, true] {
            let err = search(&Answer("[]"), QUERY, json, &english()).unwrap_err();
            assert_eq!(
                err.to_string(),
                "no place found for \"Clara-Zetkin-Schule Strausberg\"; try another name, or \
                 draw the area with: freemapper select <dir>"
            );
        }
    }

    #[test]
    fn a_failed_request_is_the_library_error() {
        let err = search(&Answer(SEARCH), "elsewhere", false, &english()).unwrap_err();
        let message = crate::errors::message(err.as_ref(), &english());
        assert!(message.contains("not found on the server"), "{message}");
    }

    #[test]
    fn german_texts() {
        let out = search(&Answer(MIXED), QUERY, false, &german()).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert!(
            lines[1].starts_with(
                "node/2147483647  Clara-Zetkin-Schule (highway=bus_stop; keine Fläche, kann kein \
                 Kartengebiet sein): Am Sportpark"
            ),
            "{out}"
        );
        assert_eq!(
            lines[2],
            "Projekt anlegen mit: freemapper init <ordner> --osm way/992423119"
        );
        let err = search(&Answer("[]"), QUERY, false, &german()).unwrap_err();
        assert!(
            err.to_string()
                .starts_with("Zu „Clara-Zetkin-Schule Strausberg“ wurde kein Ort gefunden."),
            "{err}"
        );
    }
}
