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
    let output = search(fetcher().as_ref(), query, json, &crate::text::lang())?;
    print(&mut std::io::stdout().lock(), &output)
}

/// The output of `find` for `query` in `lang`. Nothing found is an error in text and an empty
/// array in JSON, as the MCP tool `find_place` answers.
fn search(fetcher: &dyn Fetcher, query: &str, json: bool, lang: &Lang) -> anyhow::Result<String> {
    let places = find_places(fetcher, query)?;
    if json {
        return Ok(Value::from(places.iter().map(place_json).collect::<Vec<_>>()).to_string());
    }
    if places.is_empty() {
        anyhow::bail!(TEXTS.text(lang, "cli-find-nothing", &[("query", v(query))]));
    }
    let places: Vec<PlaceCandidate> = places.iter().map(cleaned).collect();
    let mut lines: Vec<String> = places.iter().map(|place| line(place, lang)).collect();
    let usable = |place: &&PlaceCandidate| place.has_area && plain_osm_id(&place.osm_id);
    lines.push(match places.iter().find(usable) {
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

/// `place` with every text field [`clean`]ed, for the terminal.
fn cleaned(place: &PlaceCandidate) -> PlaceCandidate {
    PlaceCandidate {
        osm_id: clean(&place.osm_id),
        name: clean(&place.name),
        display_name: clean(&place.display_name),
        category: clean(&place.category),
        kind: clean(&place.kind),
        has_area: place.has_area,
    }
}

/// `s` without what could steer a terminal: control characters (C0, DEL, C1) become spaces, and
/// the bidi embeddings, overrides and isolates (U+202A to U+202E, U+2066 to U+2069) go. The texts
/// come from OpenStreetMap, where anyone can write them.
fn clean(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// Whether `id` is `node/<digits>`, `way/<digits>` or `relation/<digits>`, so the hint line can
/// be copied into a shell as it is.
fn plain_osm_id(id: &str) -> bool {
    id.split_once('/').is_some_and(|(kind, number)| {
        matches!(kind, "node" | "way" | "relation")
            && !number.is_empty()
            && number.bytes().all(|b| b.is_ascii_digit())
    })
}

/// Writes `text` and a line end to `out`. A reader that is gone (`find x | head -1`) ends the
/// output quietly.
fn print(out: &mut dyn std::io::Write, text: &str) -> anyhow::Result<()> {
    match writeln!(out, "{text}").and_then(|()| out.flush()) {
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        result => Ok(result?),
    }
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
    fn nothing_found_is_an_error_in_text_and_an_empty_array_in_json() {
        let err = search(&Answer("[]"), QUERY, false, &english()).unwrap_err();
        assert_eq!(
            err.to_string(),
            "no place found for \"Clara-Zetkin-Schule Strausberg\"; try another name, or \
             draw the area with: freemapper select <dir>"
        );
        // As the MCP tool find_place: an empty list.
        assert_eq!(
            search(&Answer("[]"), QUERY, true, &english()).unwrap(),
            "[]"
        );
    }

    /// Names and the other fields come from OpenStreetMap; terminal escapes, carriage returns and
    /// bidi overrides in them must not reach the terminal. The JSON keeps them as they are.
    #[test]
    fn text_output_has_no_control_or_bidi_characters() {
        const EVIL: &str = r#"[{"osm_type":"way","osm_id":5,"name":"Evil\u001b[2J\rname","display_name":"Evil\u001b[2J\rname, \u202eStrausberg\u2066x\u2069","category":"amenity\u0085","type":"school\u007f","geojson":{"type":"Polygon","coordinates":[[[13.8,52.5],[13.81,52.5],[13.81,52.51],[13.8,52.5]]]}}]"#;
        let out = search(&Answer(EVIL), QUERY, false, &english()).unwrap();
        let bidi = |c: char| matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}');
        assert!(
            out.chars()
                .all(|c| (c == '\n' || !c.is_control()) && !bidi(c)),
            "{out:?}"
        );
        assert!(
            out.starts_with("way/5  Evil [2J name (amenity =school ): "),
            "{out:?}"
        );
        let json = search(&Answer(EVIL), QUERY, true, &english()).unwrap();
        let places: Vec<Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(places[0]["name"], "Evil\u{1b}[2J\rname");
    }

    #[test]
    fn the_hint_names_only_a_plain_osm_id() {
        const ODD: &str = r#"[
            {"osm_type":"way;rm -rf ~","osm_id":1,"name":"A","display_name":"A, B","category":"amenity","type":"school","geojson":{"type":"Polygon","coordinates":[[[13.8,52.5],[13.81,52.5],[13.81,52.51],[13.8,52.5]]]}},
            {"osm_type":"relation","osm_id":2,"name":"C","display_name":"C, D","category":"amenity","type":"school","geojson":{"type":"Polygon","coordinates":[[[13.8,52.5],[13.81,52.5],[13.81,52.51],[13.8,52.5]]]}}
        ]"#;
        let out = search(&Answer(ODD), QUERY, false, &english()).unwrap();
        assert_eq!(
            out.lines().last(),
            Some("Create the project with: freemapper init <dir> --osm relation/2")
        );
        for id in ["node/1", "way/22", "relation/333"] {
            assert!(plain_osm_id(id), "{id}");
        }
        for id in [
            "way/", "way/1a", "ways/1", "way/1 x", "/1", "way/-1", "way;rm/1",
        ] {
            assert!(!plain_osm_id(id), "{id}");
        }
    }

    /// `freemapper find x | head -1`: a closed pipe ends the output quietly.
    #[test]
    fn a_closed_pipe_is_no_error() {
        struct Closed(std::io::ErrorKind);
        impl std::io::Write for Closed {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(self.0.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        assert!(print(&mut Closed(std::io::ErrorKind::BrokenPipe), "x").is_ok());
        assert!(print(&mut Closed(std::io::ErrorKind::PermissionDenied), "x").is_err());
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
