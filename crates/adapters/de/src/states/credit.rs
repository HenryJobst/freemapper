//! The credit lines of the states: the text each licence prescribes, the year it carries and
//! when it is due.

use chrono::Datelike as _;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};

use crate::berlin::{BERLIN_REGISTERS, TREE_REGISTERS};
use crate::sources::lgb::{ProductTiles as _, TileProvider, attribution, attribution_with_berlin};
use crate::states::{Agency, State};

/// GeoSN's source note for dl-de/by-2-0 data, which asks for changes to be marked.
pub const GEOSN_ATTRIBUTION: &str = "Quelle: GeoSN, dl-de/by-2-0, Daten geändert";
/// Geoportal Berlin data needs no attribution (dl-de/zero-2-0); it is named anyway.
pub const BERLIN_ATTRIBUTION: &str =
    "© Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0, Daten geändert";
/// The Berlin K5 map, once the template was written.
pub const BERLIN_K5_ATTRIBUTION: &str =
    "Geoportal Berlin / Karte 1:5000 von Berlin (K5), dl-de/zero-2-0";
/// The Berlin tree cadastre, once tree candidates were written.
pub const BERLIN_TREES_ATTRIBUTION: &str = "Geoportal Berlin / Baumbestand Berlin, dl-de/zero-2-0";
/// The Straßenbefahrung layers, once one of them has features.
pub const BERLIN_SURVEY_ATTRIBUTION: &str =
    "Geoportal Berlin / Straßenbefahrung 2014, dl-de/zero-2-0, Daten geändert";
/// The building heights (footprint dates), once their layer has features.
pub const BERLIN_HEIGHTS_ATTRIBUTION: &str =
    "Geoportal Berlin / Gebäudehöhen (Umweltatlas), dl-de/zero-2-0";
/// The licence text the dl-de/by-2-0 credits of Rhineland-Palatinate, Saarland and
/// Baden-Württemberg refer to
/// (added to the project notes only).
pub const DL_DE_BY_LICENCE_NOTE: &str = "Lizenz dl-de/by-2-0 (Datenlizenz Deutschland – Namensnennung – Version 2.0): https://www.govdata.de/dl-de/by-2-0";

/// The licence text the CC BY credits (Schleswig-Holstein, Mecklenburg-Vorpommern,
/// Niedersachsen, Bremen, Bavaria) refer to (added to the project notes only).
pub const CC_BY_LICENCE_NOTE: &str = "Lizenz CC BY 4.0 (Namensnennung 4.0 International): https://creativecommons.org/licenses/by/4.0/";
/// The LVermGeo SH credit the licence prescribes for changed data.
pub const SH_ATTRIBUTION: &str = "© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)";

/// The credit of the Bayerische Vermessungsverwaltung for changed data, as its FAQ writes it.
pub const BY_ATTRIBUTION: &str = "Geobasisdaten: Bayerische Vermessungsverwaltung – www.geodaten.bayern.de (Daten verändert), Lizenz: CC BY 4.0";

/// The LAiV M-V credit the licence prescribes for changed data.
pub const MV_ATTRIBUTION: &str = "©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)";

/// The credit lines of `region` for what the project holds of it; empty before anything of it
/// was fetched.
pub fn lines(state: State, used: &UsedSources<'_>) -> Vec<CreditLine> {
    // The year of the newest fetch of the state's per-area rasters and ALKIS objects.
    let year = || used.newest_fetch_year(state.agency().id(), state.id());
    let line: Option<CreditLine> = match state {
        State::Brandenburg => lgb(used).map(CreditLine::new),
        State::Berlin => {
            // The lines of the city tree cadastres keep their place behind those of Berlin
            // until the cadastres belong to their adapters (plan C13).
            return berlin(used)
                .into_iter()
                .chain(tree_attribution(used.drawn_registers))
                .map(CreditLine::new)
                .collect();
        }
        State::Saxony => geosn(used).then(|| CreditLine::new(GEOSN_ATTRIBUTION)),
        State::SaxonyAnhalt => year().map(|year| {
            CreditLine::new(format!(
                "© GeoBasis-DE / LVermGeo ST {year}, dl-de/by-2-0, Daten geändert"
            ))
        }),
        State::Hamburg => year().map(|year| {
            CreditLine::new(format!(
                "Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) {year}, dl-de/by-2-0, Daten geändert"
            ))
        }),
        State::Hessen => year().map(|year| {
            CreditLine::new(format!(
                "Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation (HVBG) {year}, dl-de/zero-2-0, verändert"
            ))
        }),
        State::SchleswigHolstein => {
            year().map(|_| CreditLine::with_licence(SH_ATTRIBUTION, CC_BY_LICENCE_NOTE))
        }
        State::MecklenburgVorpommern => {
            year().map(|_| CreditLine::with_licence(MV_ATTRIBUTION, CC_BY_LICENCE_NOTE))
        }
        // The licence (dl-de/zero-2-0) asks for no note of changes.
        State::NordrheinWestfalen => year().map(|year| {
            CreditLine::new(format!(
                "Geobasisdaten © Geobasis NRW (Bezirksregierung Köln) {year}, dl-de/zero-2-0"
            ))
        }),
        // The credit the AGNB prescribes (with the year and the note of changes) and the licence.
        State::Niedersachsen => year().map(|year| {
            CreditLine::with_licence(
                format!("© GeoBasis-DE/LGLN {year}, Daten geändert, CC BY 4.0"),
                CC_BY_LICENCE_NOTE,
            )
        }),
        // The form the open-data page prescribes (no blank before the year), with the note of
        // changes, since freemapper derives map objects.
        State::RheinlandPfalz => year().map(|year| {
            CreditLine::with_licence(
                format!(
                    "©GeoBasis-DE / LVermGeoRP{year}, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]"
                ),
                DL_DE_BY_LICENCE_NOTE,
            )
        }),
        // The source note of the LVGL's records, `© GeoBasis DE/LVGL-SL (Jahr)`, with the year
        // in its parentheses, the note of changes and the licence.
        State::Saarland => year().map(|year| {
            CreditLine::with_licence(
                format!("© GeoBasis DE/LVGL-SL ({year}), dl-de/by-2-0, Daten geändert"),
                DL_DE_BY_LICENCE_NOTE,
            )
        }),
        // The line the open-data page of the LGL prescribes, „Datenquelle: LGL, www.lgl-bw.de,
        // dl-de/by-2-0“, kept whole; after it the note of changes and the year of the download,
        // which the capabilities of the services ask for.
        State::BadenWuerttemberg => year().map(|year| {
            CreditLine::with_licence(
                format!("Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0 (Daten geändert, {year})"),
                DL_DE_BY_LICENCE_NOTE,
            )
        }),
        // The source note the open-data page of the agency prescribes, `© GeoBasis-DE /
        // Landesamt GeoInformation Bremen (Jahr)`, with the licence and the note of changes.
        State::Bremen => year().map(|year| {
            CreditLine::with_licence(
                format!(
                    "© GeoBasis-DE / Landesamt GeoInformation Bremen ({year}), CC BY 4.0, Daten geändert"
                ),
                CC_BY_LICENCE_NOTE,
            )
        }),
        // The source note the FAQ of the agency gives for changed data, kept whole; it names
        // no year.
        State::Bayern => year().map(|_| CreditLine::with_licence(BY_ATTRIBUTION, CC_BY_LICENCE_NOTE)),
    };
    line.into_iter().collect()
}

/// The LGB line, once its tiles or Brandenburg ALKIS objects were fetched. LGB tiles in Berlin
/// hold Berlin data, and LGB's licence note then asks for the Geoportal Berlin as well.
fn lgb(used: &UsedSources<'_>) -> Option<String> {
    let tile_year = used
        .tiles
        .iter()
        .filter(|t| t.product.provider() == TileProvider::Lgb)
        .map(|t| t.published.year())
        .max();
    let alkis_year = used
        .alkis_fetches(State::Brandenburg.id())
        .map(|fetched| fetched.year())
        .max();
    let year = tile_year.or(alkis_year)?;
    let in_berlin = used
        .overlapped
        .iter()
        .any(|id| id.as_str() == State::Berlin.id());
    Some(if in_berlin {
        attribution_with_berlin(year)
    } else {
        attribution(year)
    })
}

/// Whether the GeoSN line is due: GeoSN tiles, its orthophoto or Saxon ALKIS objects were
/// fetched.
fn geosn(used: &UsedSources<'_>) -> bool {
    used.tiles
        .iter()
        .any(|t| t.product.provider() == TileProvider::GeoSn)
        || (used.orthophoto.is_some() && used.rasters_from(Agency::GeoSn.id()))
        || used.has_alkis_of(State::Saxony.id())
}

/// The Geoportal Berlin lines: ALKIS once Berlin ALKIS objects were fetched, the K5 map and the
/// tree cadastre once their data was written, the street survey and the building heights once
/// their layers have features, and one line for the registers that drew objects, each named.
fn berlin(used: &UsedSources<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    if used.has_alkis_of(State::Berlin.id()) {
        lines.push(BERLIN_ATTRIBUTION.to_string());
    }
    if used.extra_templates.contains(&crate::berlin::K5_KEY) {
        lines.push(BERLIN_K5_ATTRIBUTION.to_string());
    }
    // A stored layer without features contributed nothing (a tree layer without trees draws no
    // candidate).
    let has = |keep: &dyn Fn(&str) -> bool| {
        used.extra_layers
            .iter()
            .any(|&(file, count)| count > 0 && keep(file))
    };
    if has(&|file| TREE_REGISTERS.contains(&file)) {
        lines.push(BERLIN_TREES_ATTRIBUTION.to_string());
    }
    if has(&|file| file.starts_with("sb-")) {
        lines.push(BERLIN_SURVEY_ATTRIBUTION.to_string());
    }
    // A register that is fetched but drew nothing (another map kind, outside the area) is not
    // named.
    let names: Vec<&str> = BERLIN_REGISTERS
        .iter()
        .filter(|(file, _)| used.drawn_registers.contains(file))
        .map(|&(_, name)| name)
        .collect();
    if !names.is_empty() {
        lines.push(format!(
            "Geoportal Berlin / {}, dl-de/zero-2-0",
            names.join(", ")
        ));
    }
    if has(&|file| file == "gebaeudehoehen") {
        lines.push(BERLIN_HEIGHTS_ATTRIBUTION.to_string());
    }
    lines
}

/// The attribution lines of the city tree cadastres in `registers` ([`drawn_registers`]): one
/// for each dataset that drew trees.
pub fn tree_attribution(registers: &[&str]) -> Vec<String> {
    crate::states::all_tree_datasets()
        .into_iter()
        .filter(|d| registers.contains(&d.id.as_ref()))
        .map(|d| d.attribution.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use freemapper_adapter_api::adapter::RegionAdapter as _;
    use freemapper_adapter_api::credit::{AlkisUse, lines_with_licences};
    use freemapper_adapter_api::records::{AdapterId, WcsRecord, WmsRecord};
    use freemapper_adapter_api::registry::Registry;

    use crate::states::StateAdapter;

    use super::*;

    fn day(year: i32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, 6, 15).unwrap()
    }

    /// The source note of the LVGL's records, `© GeoBasis DE/LVGL-SL (Jahr)`, with the year of
    /// the newest fetch, the licence and the note of changes; the notes of the project add the
    /// address of the licence text.
    #[test]
    fn saarland_names_the_lvgl_the_year_and_the_licence() {
        let line =
            |year: i32| format!("© GeoBasis DE/LVGL-SL ({year}), dl-de/by-2-0, Daten geändert");
        let nothing = UsedSources::default();
        assert!(lines(State::Saarland, &nothing).is_empty());

        let alkis = UsedSources {
            alkis: vec![AlkisUse {
                regions: vec![AdapterId::built_in("saarland")],
                fetched: day(2025),
            }],
            ..UsedSources::default()
        };
        let texts = |used: &UsedSources<'_>| -> Vec<String> {
            lines(State::Saarland, used)
                .into_iter()
                .map(|line| line.text)
                .collect()
        };
        assert_eq!(texts(&alkis), [line(2025)]);

        let terrain = WcsRecord {
            fetched: day(2026),
            bbox: [0.0; 4],
            date: None,
        };
        let orthophoto = WmsRecord {
            flight: None,
            flight_label: None,
            fetched: day(2026),
            bbox: [0.0; 4],
        };
        let rasters = UsedSources {
            raster_provider: Some(AdapterId::built_in("lvgl")),
            terrain: Some(&terrain),
            orthophoto: Some(&orthophoto),
            ..UsedSources::default()
        };
        assert_eq!(texts(&rasters), [line(2026)]);
        // Both: one line, with the newer year.
        let both = UsedSources {
            alkis: alkis.alkis.clone(),
            ..rasters.clone()
        };
        assert_eq!(texts(&both), [line(2026)]);
        // Rasters of another provider are not the LVGL's.
        let other = UsedSources {
            raster_provider: Some(AdapterId::built_in("lvermgeorp")),
            ..rasters.clone()
        };
        assert!(lines(State::Saarland, &other).is_empty());

        assert_eq!(
            lines_with_licences(&lines(State::Saarland, &both)),
            [line(2026), DL_DE_BY_LICENCE_NOTE.to_string()]
        );
    }

    /// The credit of a state is said twice: here, and in its manifest, which is what a project
    /// gets for every state but Brandenburg, Berlin and Saxony. The two say the same, text and
    /// licence note.
    #[test]
    fn the_credit_of_each_manifest_is_the_one_written_here() {
        let mut backends = Registry::new();
        crate::states::register_backends(&mut backends).unwrap();
        for state in State::ALL {
            let alkis = UsedSources {
                alkis: vec![AlkisUse {
                    regions: vec![AdapterId::built_in(state.id())],
                    fetched: day(2026),
                }],
                ..UsedSources::default()
            };
            let adapter = StateAdapter::new(state, &backends);
            let from_manifest = adapter.manifest.attribution(&alkis);
            assert_eq!(from_manifest.len(), 1, "{}", state.id());
            if !matches!(state, State::Brandenburg | State::Berlin | State::Saxony) {
                assert_eq!(from_manifest, lines(state, &alkis), "{}", state.id());
                assert_eq!(adapter.attribution(&alkis), from_manifest, "{}", state.id());
            }
        }
    }
}
