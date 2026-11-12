//! The credit lines of the eleven states, as the attribution functions of `project.rs` gave
//! them (plan C4): the texts, the year and the conditions are unchanged, the lines are asked
//! through [`RegionAdapter::attribution`](freemapper_adapter_api::adapter::RegionAdapter).

use chrono::Datelike as _;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};

use crate::project::BERLIN_REGISTERS;
use crate::sources::lgb::{ProductTiles as _, TileProvider, attribution, attribution_with_berlin};
use crate::sources::region::{Provider, Region};
use crate::sources::trees;

/// GeoSN's source note for dl-de/by-2-0 data, which asks for changes to be marked.
pub(crate) const GEOSN_ATTRIBUTION: &str = "Quelle: GeoSN, dl-de/by-2-0, Daten geändert";
/// Geoportal Berlin data needs no attribution (dl-de/zero-2-0); it is named anyway.
pub(crate) const BERLIN_ATTRIBUTION: &str =
    "© Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0, Daten geändert";
/// The Berlin K5 map, once the template was written.
pub(crate) const BERLIN_K5_ATTRIBUTION: &str =
    "Geoportal Berlin / Karte 1:5000 von Berlin (K5), dl-de/zero-2-0";
/// The Berlin tree cadastre, once tree candidates were written.
pub(crate) const BERLIN_TREES_ATTRIBUTION: &str =
    "Geoportal Berlin / Baumbestand Berlin, dl-de/zero-2-0";
/// The Straßenbefahrung layers, once one of them has features.
pub(crate) const BERLIN_SURVEY_ATTRIBUTION: &str =
    "Geoportal Berlin / Straßenbefahrung 2014, dl-de/zero-2-0, Daten geändert";
/// The building heights (footprint dates), once their layer has features.
pub(crate) const BERLIN_HEIGHTS_ATTRIBUTION: &str =
    "Geoportal Berlin / Gebäudehöhen (Umweltatlas), dl-de/zero-2-0";
/// The licence the LVermGeo RP line refers to (added to the project notes only).
pub(crate) const RP_LICENCE_NOTE: &str = "Lizenz dl-de/by-2-0 (Datenlizenz Deutschland – Namensnennung – Version 2.0): https://www.govdata.de/dl-de/by-2-0";

/// The licence text the CC BY credits (Schleswig-Holstein, Mecklenburg-Vorpommern,
/// Niedersachsen) refer to (added to the project notes only).
pub(crate) const CC_BY_LICENCE_NOTE: &str = "Lizenz CC BY 4.0 (Namensnennung 4.0 International): https://creativecommons.org/licenses/by/4.0/";
/// The LVermGeo SH credit the licence prescribes for changed data.
pub(crate) const SH_ATTRIBUTION: &str = "© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)";

/// The LAiV M-V credit the licence prescribes for changed data.
pub(crate) const MV_ATTRIBUTION: &str = "©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)";

/// The credit lines of `region` for what the project holds of it; empty before anything of it
/// was fetched.
pub(crate) fn lines(region: Region, used: &UsedSources<'_>) -> Vec<CreditLine> {
    // The year of the newest fetch of the state's per-area rasters and ALKIS objects.
    let year = || used.newest_fetch_year(region.provider().id(), region.id());
    let line: Option<CreditLine> = match region {
        Region::Brandenburg => lgb(used).map(CreditLine::new),
        Region::Berlin => {
            // The lines of the city tree cadastres keep their place behind those of Berlin
            // until the cadastres belong to their adapters (plan C13).
            return berlin(used)
                .into_iter()
                .chain(tree_attribution(used.drawn_registers))
                .map(CreditLine::new)
                .collect();
        }
        Region::Saxony => geosn(used).then(|| CreditLine::new(GEOSN_ATTRIBUTION)),
        Region::SaxonyAnhalt => year().map(|year| {
            CreditLine::new(format!(
                "© GeoBasis-DE / LVermGeo ST {year}, dl-de/by-2-0, Daten geändert"
            ))
        }),
        Region::Hamburg => year().map(|year| {
            CreditLine::new(format!(
                "Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) {year}, dl-de/by-2-0, Daten geändert"
            ))
        }),
        Region::Hessen => year().map(|year| {
            CreditLine::new(format!(
                "Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation (HVBG) {year}, dl-de/zero-2-0, verändert"
            ))
        }),
        Region::SchleswigHolstein => {
            year().map(|_| CreditLine::with_licence(SH_ATTRIBUTION, CC_BY_LICENCE_NOTE))
        }
        Region::MecklenburgVorpommern => {
            year().map(|_| CreditLine::with_licence(MV_ATTRIBUTION, CC_BY_LICENCE_NOTE))
        }
        // The licence (dl-de/zero-2-0) asks for no note of changes.
        Region::NordrheinWestfalen => year().map(|year| {
            CreditLine::new(format!(
                "Geobasisdaten © Geobasis NRW (Bezirksregierung Köln) {year}, dl-de/zero-2-0"
            ))
        }),
        // The credit the AGNB prescribes (with the year and the note of changes) and the licence.
        Region::Niedersachsen => year().map(|year| {
            CreditLine::with_licence(
                format!("© GeoBasis-DE/LGLN {year}, Daten geändert, CC BY 4.0"),
                CC_BY_LICENCE_NOTE,
            )
        }),
        // The form the open-data page prescribes (no blank before the year), with the note of
        // changes, since freemapper derives map objects.
        Region::RheinlandPfalz => year().map(|year| {
            CreditLine::with_licence(
                format!(
                    "©GeoBasis-DE / LVermGeoRP{year}, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]"
                ),
                RP_LICENCE_NOTE,
            )
        }),
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
        .alkis_fetches(Region::Brandenburg.id())
        .map(|fetched| fetched.year())
        .max();
    let year = tile_year.or(alkis_year)?;
    let in_berlin = used
        .overlapped
        .iter()
        .any(|id| id.as_str() == Region::Berlin.id());
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
        || (used.orthophoto.is_some() && used.rasters_from(Provider::GeoSn.id()))
        || used.has_alkis_of(Region::Saxony.id())
}

/// The Geoportal Berlin lines: ALKIS once Berlin ALKIS objects were fetched, the K5 map and the
/// tree cadastre once their data was written, the street survey and the building heights once
/// their layers have features, and one line for the registers that drew objects, each named.
fn berlin(used: &UsedSources<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    if used.has_alkis_of(Region::Berlin.id()) {
        lines.push(BERLIN_ATTRIBUTION.to_string());
    }
    if used.extra_templates.contains(&"k5") {
        lines.push(BERLIN_K5_ATTRIBUTION.to_string());
    }
    // A stored layer without features contributed nothing (a tree layer without trees draws no
    // candidate).
    let has = |keep: &dyn Fn(&str) -> bool| {
        used.extra_layers
            .iter()
            .any(|&(file, count)| count > 0 && keep(file))
    };
    if has(&|file| ["strassenbaeume", "anlagenbaeume"].contains(&file)) {
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
pub(crate) fn tree_attribution(registers: &[&str]) -> Vec<String> {
    trees::all_tree_datasets()
        .filter(|d| registers.contains(&d.id.as_ref()))
        .map(|d| d.attribution.to_string())
        .collect()
}
