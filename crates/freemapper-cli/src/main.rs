// SPDX-License-Identifier: AGPL-3.0-or-later

//! `freemapper` command line interface.

mod adapter_check;
mod adapters;
mod corrections;
mod errors;
mod estimate;
mod find;
mod help;
mod hosts;
mod jobs;
mod lock;
mod mcp;
mod nominatim;
mod permits;
mod pipeline;
mod select;
mod select_page;
mod select_preview;
mod select_run;
mod sources;
mod text;
mod tiles;
mod update;

use std::io::IsTerminal as _;
use std::path::PathBuf;

use anyhow::Context;
use clap::{CommandFactory as _, FromArgMatches as _, Parser, Subcommand, ValueEnum};
use freemapper::area::MapArea;
use freemapper::build::content::ContentSettings;
use freemapper::fetch::{Cache, FetchMode, HttpFetcher};
use freemapper::places::{find_places, place_area};
use freemapper::project::{
    MapKind, MapOutcome, OcadOutcome, PdfSources, ProjectFile, TrainingOutcome, derive_project,
    export_candidates_pdf_from, export_map_preview_or_warn, export_ocad_from, export_pdf_from,
    export_starter_project_from, export_training_from, export_training_views, fetch_project,
    init_project, kept_map_lacks_paper_frames, map_path, parse_contour_interval, parse_face_slope,
    parse_steep_slope, set_content, set_contour_interval, set_face_slope, set_steep_slope,
    starter_documents, write_paper_frame_template,
};

use text::{n, text, v};

#[derive(Parser)]
#[command(
    name = "freemapper",
    version,
    about = "Builds OOM starter projects from geodata",
    arg_required_else_help = true
)]
struct Cli {
    /// Language of the program texts, `de` or `en` (default: FREEMAPPER_LANG, then the system
    /// language, then English).
    #[arg(long, global = true, value_name = "LANGUAGE")]
    lang: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a project folder for a map area.
    Init {
        /// Project folder to create.
        dir: PathBuf,
        #[command(flatten)]
        area: AreaArgs,
        /// Which part of an OSM area of several separate parts to use (1 is the largest).
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        part: Option<u32>,
        /// Map kind.
        #[arg(long, value_enum, default_value_t = Kind::Schoolyard)]
        kind: Kind,
        /// Scale denominator of a new project: schoolyard 1000 (default); sprint 4000 (default)
        /// or 5000; forest 15000 (default), 10000 or 7500. Other values are refused, because
        /// freemapper-print refuses them.
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        scale: Option<u32>,
        /// Map title for the peripheral information (default: the folder name).
        #[arg(long)]
        title: Option<String>,
        #[command(flatten)]
        content: ContentArgs,
        #[command(flatten)]
        contours: ContourArgs,
        #[command(flatten)]
        adapters: AdapterArgs,
    },
    /// Write the starter project (OOM map, templates, attribution) into a project folder.
    Export {
        /// Project folder.
        dir: PathBuf,
        /// Replace an existing map file and keep the old one as `<name>.omap.bak`.
        #[arg(long)]
        force: bool,
        /// Also write an OCAD starting point into `ocad/` (OCAD 12 format); an existing OOM map
        /// then stays unless you pass --force.
        #[arg(long)]
        ocad: bool,
        /// Also write `<name>.pdf`, the printable map (the map objects only, no candidates); an
        /// existing OOM map then stays unless you pass --force.
        #[arg(long)]
        pdf: bool,
        /// Also write `<name>-candidates.pdf`, the check print: the map with every candidate, true
        /// to position; an existing OOM map then stays unless you pass --force.
        #[arg(long)]
        pdf_candidates: bool,
        /// Also write the training map `<name>-training.omap` and its PDF
        /// `<name>-training.pdf`: the map and every candidate, cleaned up to be printed as they
        /// are; with --ocad also `ocad/<name>-training.ocd`. An existing OOM map then stays
        /// unless you pass --force, and --force also replaces an existing training map.
        #[arg(long)]
        training: bool,
        /// With --training: also write the training map without what a view leaves out, as
        /// `<name>-training-<view>.omap` and its PDF (with --ocad also an OCAD file). Views:
        /// `no-paths` (no roads, paths, stairs), `brown-only` (only brown symbols and the frame),
        /// `no-north-lines`, or a list `no:<codes>` / `only:<codes>`; combine with `+`. Repeat the
        /// option for several files
        #[arg(long = "view", requires = "training")]
        view: Vec<String>,
        /// Draw the scale bar on the PDFs (off by default, German maps usually have none; the
        /// scale number stays in the title block).
        #[arg(long)]
        pdf_scale_bar: bool,
        /// Source lines on the PDFs: `auto` prints the full list and shortens it only when it
        /// would take more than 5 lines, `full` never shortens, `credit-only` prints only the
        /// freemapper credit line (ODbL and dl-de/by-2-0 then ask you to name the sources when
        /// you pass the map on). ATTRIBUTION.txt always lists every source.
        #[arg(long, value_enum, default_value_t = SourcesArg::Auto)]
        pdf_sources: SourcesArg,
        #[command(flatten)]
        content: ContentArgs,
        #[command(flatten)]
        adapters: AdapterArgs,
    },
    /// Download the data for the map area (orthophoto, terrain model, nDSM, ALKIS buildings, OSM).
    Fetch {
        /// Project folder.
        dir: PathBuf,
        /// Use the cache only, without network access.
        #[arg(long)]
        offline: bool,
        #[command(flatten)]
        cache: CacheArgs,
        #[command(flatten)]
        adapters: AdapterArgs,
    },
    /// Compute the templates (orthophoto, hillshade, slope, local relief, nDSM), contours and landforms from the fetched data.
    Derive {
        /// Project folder.
        dir: PathBuf,
        #[command(flatten)]
        contours: ContourArgs,
        #[command(flatten)]
        cache: CacheArgs,
        #[command(flatten)]
        adapters: AdapterArgs,
    },
    /// Fetch, derive and export in one go.
    Run {
        /// Project folder.
        dir: PathBuf,
        /// Use the cache only, without network access.
        #[arg(long)]
        offline: bool,
        /// Replace an existing map file and keep the old one as `<name>.omap.bak`.
        #[arg(long)]
        force: bool,
        /// Also write an OCAD starting point into `ocad/` (OCAD 12 format).
        #[arg(long)]
        ocad: bool,
        /// Also write `<name>.pdf`, the printable map (the map objects only, no candidates).
        #[arg(long)]
        pdf: bool,
        /// Also write `<name>-candidates.pdf`, the check print: the map with every candidate, true
        /// to position.
        #[arg(long)]
        pdf_candidates: bool,
        /// Also write the training map `<name>-training.omap` and its PDF
        /// `<name>-training.pdf`: the map and every candidate, cleaned up to be printed as they
        /// are; with --ocad also `ocad/<name>-training.ocd`. An existing training map stays unless
        /// you pass --force.
        #[arg(long)]
        training: bool,
        /// With --training: also write the training map without what a view leaves out, as
        /// `<name>-training-<view>.omap` and its PDF (with --ocad also an OCAD file). Views:
        /// `no-paths` (no roads, paths, stairs), `brown-only` (only brown symbols and the frame),
        /// `no-north-lines`, or a list `no:<codes>` / `only:<codes>`; combine with `+`. An
        /// existing view file stays unless you pass --force
        #[arg(long = "view", requires = "training")]
        view: Vec<String>,
        /// Draw the scale bar on the PDFs (off by default; the scale number stays in the title block).
        #[arg(long)]
        pdf_scale_bar: bool,
        /// Source lines on the PDFs: `auto` prints the full list and shortens it only when it
        /// would take more than 5 lines, `full` never shortens, `credit-only` prints only the
        /// freemapper credit line (ODbL and dl-de/by-2-0 then ask you to name the sources when
        /// you pass the map on). ATTRIBUTION.txt always lists every source.
        #[arg(long, value_enum, default_value_t = SourcesArg::Auto)]
        pdf_sources: SourcesArg,
        /// Also write the permit notes (permits.md and permits.json) after the map; asks the
        /// BfN, the BKG and the open services of the state (manual, chapter 12). A failure
        /// there is a warning and the map stays.
        #[arg(long)]
        permits: bool,
        #[command(flatten)]
        content: ContentArgs,
        #[command(flatten)]
        contours: ContourArgs,
        #[command(flatten)]
        cache: CacheArgs,
        #[command(flatten)]
        adapters: AdapterArgs,
    },
    /// Write the permit notes of a project (permits.md and permits.json): the administrative
    /// units, protected areas and operators the map area touches, and whom that usually means
    /// asking. It asks the BfN and the BKG and, where the state of the map area has open
    /// services of its own, those through the hosts of its adapter (list in the manual,
    /// chapter 12); in North Rhine-Westphalia it downloads about 32 MB for this the first
    /// time, in Brandenburg about 4 MB, later only newer files. It reads the OSM extract
    /// already cached; --offline uses what was fetched before.
    Permits {
        /// Project folder.
        dir: PathBuf,
        /// Use the data fetched before only, without network access.
        #[arg(long)]
        offline: bool,
        #[command(flatten)]
        cache: CacheArgs,
    },
    /// Choose the map area in the browser by drawing it or picking a place. Saving writes
    /// freemapper.toml and creates the project folder if needed.
    Select {
        /// Project folder (created on saving if it does not exist).
        dir: PathBuf,
        /// Map kind of a new project.
        #[arg(long, value_enum, default_value_t = Kind::Schoolyard)]
        kind: Kind,
        /// Scale denominator of a new project: schoolyard 1000 (default); sprint 4000 (default)
        /// or 5000; forest 15000 (default), 10000 or 7500. Other values are refused, because
        /// freemapper-print refuses them.
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        scale: Option<u32>,
        /// Map title of a new project.
        #[arg(long)]
        title: Option<String>,
        /// Only print the address of the page; do not open the browser.
        #[arg(long)]
        no_browser: bool,
        /// Port on 127.0.0.1 (default: a free one).
        #[arg(long, default_value_t = 0)]
        port: u16,
        /// The page starts with "cache only, no network" ticked.
        #[arg(long)]
        offline: bool,
        /// The page starts with the OCAD starter project ticked.
        #[arg(long)]
        ocad: bool,
        /// The page starts with the printable PDF ticked.
        #[arg(long)]
        pdf: bool,
        /// The page starts with the check print (every candidate) ticked.
        #[arg(long)]
        pdf_candidates: bool,
        /// The page starts with the training map ticked.
        #[arg(long)]
        training: bool,
        /// The page starts with the scale bar on the PDFs ticked.
        #[arg(long)]
        pdf_scale_bar: bool,
        /// The page starts with this choice of source lines on the PDFs, as for `run`.
        #[arg(long, value_enum, default_value_t = SourcesArg::Auto)]
        pdf_sources: SourcesArg,
        #[command(flatten)]
        cache: CacheArgs,
    },
    /// List the OpenStreetMap places for a name, areas first, with the id for init --osm.
    Find {
        /// Name of the place, e.g. "Clara-Zetkin-Schule Strausberg".
        query: String,
        /// Print the matches as JSON, the same fields as the MCP tool find_place.
        #[arg(long)]
        json: bool,
    },
    /// Serve the tools to an AI agent over MCP (stdin/stdout).
    Mcp {
        #[command(flatten)]
        cache: CacheArgs,
    },
    /// Show the region adapters: the states and countries freemapper has data sources for.
    #[command(alias = "adapter")]
    Adapters {
        #[command(subcommand)]
        command: AdaptersCommand,
    },
    /// Read and correct the addresses of the data sources without a new release. A correction
    /// goes into your own adapter folder and replaces the built-in address.
    Sources {
        #[command(subcommand)]
        command: SourcesCommand,
    },
    /// Install a newer freemapper from the GitHub releases, after asking. A build from source says
    /// how to update it instead.
    Update {
        /// Only check: exit code 10 when a newer version exists, 0 when this one is current.
        #[arg(long, conflicts_with_all = ["yes", "rollback"])]
        check: bool,
        /// Install without asking; needed when the input is not a terminal.
        #[arg(long)]
        yes: bool,
        /// Install this version (x.y.z) instead of the newest one, also an older one.
        #[arg(long, value_name = "X.Y.Z")]
        version: Option<freemapper_update::Version>,
        /// Go back to the version the last update replaced.
        #[arg(long, conflicts_with = "version")]
        rollback: bool,
        /// Store what the daily check does when it finds a newer version (default: notify). Works
        /// in every build, because it only writes the setting.
        #[arg(long, value_enum, conflicts_with_all = ["check", "yes", "version", "rollback"])]
        mode: Option<UpdateMode>,
    },
}

#[derive(Subcommand)]
enum SourcesCommand {
    /// List the addresses of the manifests with their paths and where each manifest comes from.
    /// With an adapter id only that adapter's.
    List {
        /// The id of the adapter, as `adapters list` shows it.
        adapter: Option<String>,
        /// Print the list as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Print one address and nothing else. Exit code 1 when the adapter or the path is unknown.
    Get {
        /// The id of the adapter, as `adapters list` shows it.
        adapter: String,
        /// The path of the address, as `sources list` shows it.
        path: String,
    },
    /// Correct one address in your own adapter folder, after the whole manifest passed its check.
    /// An address on a host the manifest does not declare is refused unless you pass --add-host.
    Set {
        /// The id of the adapter, as `adapters list` shows it.
        adapter: String,
        /// The path of the address, as `sources list` shows it.
        path: String,
        /// The new address: https, no query.
        url: String,
        /// Also add the host of the new address to the hosts the manifest declares.
        #[arg(long)]
        add_host: bool,
    },
    /// Show what your adapter folder changes against the built-in manifest.
    Diff {
        /// The id of the adapter, as `adapters list` shows it.
        adapter: String,
    },
    /// Check the folder of your adapter of that id as `adapters check` does. Exit code 0: it
    /// passes; 1: it has findings; 2: the check could not run.
    Check {
        /// The id of the adapter, as `adapters list` shows it.
        adapter: String,
        /// Also ask the services of the adapter for the box --bbox names and say what each delivers. The
        /// requests stay on the hosts the manifest declares.
        #[arg(long, requires = "bbox")]
        live: bool,
        /// The box of the live check in degrees, as minLon,minLat,maxLon,maxLat. It lies inside the boundary
        /// of the adapter; 150 m by 150 m is enough.
        #[arg(
            long,
            value_name = "BOX",
            requires = "live",
            allow_hyphen_values = true
        )]
        bbox: Option<adapter_check::LiveBox>,
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
        #[command(flatten)]
        cache: CacheArgs,
    },
    /// Print the keys of a manifest that hold an address, with --json as a JSON Schema.
    Schema {
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Get the signed list of corrections of addresses from GitHub and store it, so that a
    /// service that moved works again without a new release. Only on request: nothing else
    /// asks GitHub for it unless you turn the daily check on with --mode on.
    Update {
        /// Turn the daily check for the list on or off (default: off) and do nothing else.
        #[arg(long, value_enum, conflicts_with = "remove")]
        mode: Option<OnOff>,
        /// Delete the stored list; from the next start the addresses of the program count.
        #[arg(long)]
        remove: bool,
    },
    /// Show the stored list of corrections and what each one does to this program.
    Corrections {
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum AdaptersCommand {
    /// List the adapters with where each comes from (built in, your folder, a project), its
    /// revision, the day it was last checked and the hosts it asks.
    List {
        /// Print the list as JSON.
        #[arg(long)]
        json: bool,
        /// Also read the adapters in the folder `adapters` of this project folder, as
        /// --adapters-from-project would.
        #[arg(long, value_name = "DIR")]
        project: Option<PathBuf>,
    },
    /// Check the folder of an adapter before it is used or passed on: whether it would load,
    /// what its author should know, and with --live whether its services answer. Exit code 0:
    /// it passes; 1: it has findings; 2: the check could not run.
    Check {
        /// The folder of the adapter; its name is the id of the adapter.
        dir: PathBuf,
        /// Also ask the services of the adapter for the box --bbox names and say what each
        /// delivers. The requests stay on the hosts the manifest declares.
        #[arg(long, requires = "bbox")]
        live: bool,
        /// The box of the live check in degrees, as minLon,minLat,maxLon,maxLat. It lies
        /// inside the boundary of the adapter; 150 m by 150 m is enough.
        #[arg(
            long,
            value_name = "BOX",
            requires = "live",
            allow_hyphen_values = true
        )]
        bbox: Option<adapter_check::LiveBox>,
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
        #[command(flatten)]
        cache: CacheArgs,
    },
    /// Ask every service the built-in adapters name once and say whether it answers: ok, slow,
    /// moved or error. Exit code 0 also when a service fails; the result is the answer.
    Probe {
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
        /// Read the status of last night from this file (it may be missing) and print the new
        /// status as JSON: a service is down after two bad nights in a row. The changes go to
        /// standard error.
        #[arg(long, value_name = "FILE")]
        status: Option<PathBuf>,
    },
    /// Write the built-in adapter of a state as an adapter folder you can change: its manifest
    /// and the files the manifest names, in `<dir>/<id>`. The folder must not exist yet or be
    /// empty. Change the addresses in its `manifest.toml`, check it with `adapters check`, and
    /// put it into your adapter folder: it then replaces the built-in adapter, with a warning.
    /// Removing the folder takes the built-in one back.
    Export {
        /// The id of the built-in adapter, as `adapters list` shows it.
        adapter: String,
        /// The folder in which the folder `<id>` is made.
        dir: PathBuf,
    },
    /// Allow a built-in adapter one more host: for a state whose service moved to a server
    /// its adapter does not name yet. The message about the refused address says which host.
    /// The host is checked like a host of a manifest, and freemapper still connects to
    /// servers on the internet only.
    AllowHost {
        /// The id of the built-in adapter, as `adapters list` shows it.
        adapter: String,
        /// The host to allow: a DNS name, no address, no port.
        host: String,
    },
    /// Take back a host that allow-host allowed.
    DisallowHost {
        /// The id of the built-in adapter.
        adapter: String,
        /// The host to take back.
        host: String,
    },
}

/// Whether the adapters a project brings along are loaded.
#[derive(clap::Args)]
struct AdapterArgs {
    /// Also load the adapters in the folder `adapters` of the project folder. freemapper names
    /// each of them with the hosts it asks. Use it for a project you trust: an adapter makes
    /// freemapper ask the servers it names.
    #[arg(long)]
    adapters_from_project: bool,
}

/// Where the map area comes from: exactly one of these.
#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct AreaArgs {
    /// GeoJSON file with the map area polygon (EPSG:4326).
    #[arg(long)]
    geojson: Option<PathBuf>,
    /// OSM object of the map area, e.g. way/992423119.
    #[arg(long)]
    osm: Option<String>,
    /// Name of the place, e.g. "Clara-Zetkin-Schule Strausberg"; freemapper uses the best match
    /// with an area.
    #[arg(long)]
    place: Option<String>,
}

impl AreaArgs {
    /// `part` picks one part of an OSM area of several separate parts (`--osm` only).
    fn area(self, part: Option<usize>) -> anyhow::Result<MapArea> {
        if part.is_some() && self.osm.is_none() {
            anyhow::bail!(text("cli-part-needs-osm", &[]));
        }
        if let Some(geojson) = self.geojson {
            let text = std::fs::read_to_string(&geojson)
                .with_context(|| text("cli-reading-file", &[("path", v(geojson.display()))]))?;
            let area = MapArea::from_geojson(&text)?;
            area.validate()?;
            return Ok(area);
        }
        let fetcher = HttpFetcher::new();
        if let Some(osm) = self.osm {
            return Ok(place_area(&fetcher, &osm, part)?);
        }
        let query = self.place.expect("clap requires one area source");
        let places = find_places(&fetcher, &query)?;
        let place = places
            .iter()
            .find(|p| p.has_area)
            .with_context(|| text("cli-no-area-for-place", &[("query", v(&query))]))?;
        eprintln!(
            "{}",
            text(
                "cli-using-place",
                &[("name", v(&place.display_name)), ("id", v(&place.osm_id))]
            )
        );
        // Nominatim allows one request per second.
        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(place_area(&fetcher, &place.osm_id, None)?)
    }
}

#[derive(clap::Args)]
struct CacheArgs {
    /// Cache folder for downloaded tiles (default: the system cache folder + `freemapper`).
    #[arg(long)]
    cache_dir: Option<PathBuf>,
}

impl CacheArgs {
    fn cache(self) -> anyhow::Result<Cache> {
        match self.cache_dir {
            Some(dir) => Ok(Cache::new(dir)),
            None => Cache::default_location().with_context(|| text("cli-no-cache-dir", &[])),
        }
    }

    /// The folder [`Self::cache`] uses, if there is one.
    fn folder(&self) -> Option<PathBuf> {
        self.cache_dir
            .clone()
            .or_else(|| Cache::default_location().map(|cache| cache.root().to_path_buf()))
    }
}

impl Commands {
    /// The project folder whose adapters this command loads: the folder of a command that got
    /// `--adapters-from-project`. The area page and the MCP server have no such switch.
    fn project_adapters(&self) -> Option<&std::path::Path> {
        match self {
            Commands::Init { dir, adapters, .. }
            | Commands::Export { dir, adapters, .. }
            | Commands::Fetch { dir, adapters, .. }
            | Commands::Derive { dir, adapters, .. }
            | Commands::Run { dir, adapters, .. }
                if adapters.adapters_from_project =>
            {
                Some(dir)
            }
            _ => None,
        }
    }

    /// The cache folder and `--offline` of a command after which the daily update check runs:
    /// the ones that use the network anyway (spec 3.3). `init` and `find` have no `--cache-dir`
    /// and record in the default cache folder.
    fn update_check(&self) -> Option<(Option<PathBuf>, bool)> {
        match self {
            Commands::Init { .. } | Commands::Find { .. } => Some((
                Cache::default_location().map(|cache| cache.root().to_path_buf()),
                false,
            )),
            Commands::Fetch { offline, cache, .. }
            | Commands::Run { offline, cache, .. }
            | Commands::Permits { offline, cache, .. }
            | Commands::Select { offline, cache, .. } => Some((cache.folder(), *offline)),
            Commands::Export { .. }
            | Commands::Derive { .. }
            | Commands::Mcp { .. }
            | Commands::Adapters { .. }
            | Commands::Sources { .. }
            | Commands::Update { .. } => None,
        }
    }
}

fn mode(offline: bool) -> FetchMode {
    if offline {
        FetchMode::Offline
    } else {
        FetchMode::Online
    }
}

fn report_pdf(path: &std::path::Path, candidates: bool, sources: PdfSources) {
    if sources.leaves_out_required_credits() {
        println!("{}", text("cli-pdf-without-credits", &[]));
    }
    let id = if candidates {
        "cli-wrote-pdf-candidates"
    } else {
        "cli-wrote-pdf"
    };
    println!("{}", text(id, &[("path", v(path.display()))]));
}

/// What the training export wrote. Its warnings are printed by the caller: `run` gets them as
/// warnings of the run.
fn report_training(outcome: &TrainingOutcome, sources: PdfSources) {
    if sources.leaves_out_required_credits() {
        println!("{}", text("cli-pdf-without-credits", &[]));
    }
    println!("{}", pipeline::training_summary(outcome));
}

/// What the OCAD export wrote and what the user does next in OCAD.
fn report_ocad(outcome: &OcadOutcome) {
    for warning in &outcome.warnings {
        eprintln!("{}", pipeline::line(warning));
    }
    let map = ("path", v(outcome.map.display()));
    if outcome.kept {
        println!("{}", text("cli-ocad-kept", &[map]));
        if outcome.old_candidates {
            let old = pipeline::old_ocad_candidates(outcome);
            println!(
                "{}",
                text("cli-ocad-old-candidates", &[("path", v(old.display()))])
            );
        }
    } else {
        println!(
            "{}",
            text("cli-ocad-wrote", &[map, ("count", n(outcome.objects))])
        );
    }
    for path in &outcome.candidates {
        wrote(path);
    }
    if outcome.skipped > 0 || outcome.dropped_parts > 0 {
        println!(
            "{}",
            text(
                "cli-ocad-left-out",
                &[
                    ("skipped", n(outcome.skipped)),
                    ("dropped", n(outcome.dropped_parts))
                ]
            )
        );
    }
    println!(
        "{}",
        text(
            "cli-ocad-load-symbols",
            &[("kind", v(outcome.map_kind.name()))]
        )
    );
}

/// Prints what a step of `run` wrote.
fn print_written(written: pipeline::Written, sources: PdfSources) {
    match written {
        pipeline::Written::Map(outcome) => {
            for path in &outcome.templates {
                wrote(path);
            }
            match &outcome.map {
                MapOutcome::Written(map) => {
                    wrote(map);
                    if let Some(preview) = &outcome.preview {
                        wrote(preview);
                    }
                    if let Some(line) = pipeline::light_summary(&outcome.light, "cli-map-light") {
                        println!("{line}");
                    }
                }
                MapOutcome::Kept {
                    map,
                    old_candidates,
                    references_images,
                    missing_buildings,
                    missing_canopies,
                    paper_frames,
                } => {
                    kept(map);
                    if let Some(line) = pipeline::light_kept(&outcome.light, "cli-map-light-kept") {
                        println!("{line}");
                    }
                    if *old_candidates {
                        println!("{}", text("cli-map-old-candidates", &[]));
                    }
                    if !references_images {
                        println!("{}", text("cli-map-lacks-templates", &[]));
                    }
                    if *missing_buildings {
                        println!("{}", text("cli-map-lacks-buildings", &[]));
                    }
                    if *missing_canopies {
                        println!("{}", text("cli-map-lacks-canopies", &[]));
                    }
                    if *paper_frames {
                        println!("{}", text("cli-paper-frame-kept-map", &[]));
                    }
                }
            }
        }
        pipeline::Written::Ocad(outcome) => report_ocad(outcome),
        pipeline::Written::Pdf(path) => report_pdf(path, false, sources),
        pipeline::Written::PdfCandidates(path) => report_pdf(path, true, sources),
        pipeline::Written::Training(outcome) => report_training(outcome, sources),
        pipeline::Written::TrainingKept(path) => println!(
            "{}",
            text("cli-kept-training", &[("path", v(path.display()))])
        ),
        pipeline::Written::Permits(path) => wrote_permits(path),
    }
}

fn report(message: &str) {
    eprintln!("{message}");
}

/// Prints a progress line or a warning of the engine in the language of the run.
fn report_note(note: freemapper::note::Note) {
    report(&pipeline::line(&note));
}

/// Says that the permit notes are in `path`.
fn wrote_permits(path: &std::path::Path) {
    println!(
        "{}",
        text("cli-wrote-permits", &[("path", v(path.display()))])
    );
}

/// Says that `path` was written.
fn wrote(path: &std::path::Path) {
    println!("{}", text("cli-wrote", &[("path", v(path.display()))]));
}

/// Takes the lock of the project folder for a command that writes into it, so no other freemapper
/// process (a terminal, the MCP server, the selection page) works in it at the same time. A
/// folder that does not exist is left to the command, which reports its usual error.
fn lock_folder(dir: &std::path::Path) -> anyhow::Result<Option<lock::FolderLock>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    Ok(Some(lock::acquire_for_command(dir, lock::Who::Cli)?))
}

/// Says that the map at `path` was kept.
fn kept(path: &std::path::Path) {
    println!("{}", text("cli-kept-map", &[("path", v(path.display()))]));
}

/// What goes straight onto the map (ADR 0100): the choice is saved in `freemapper.toml`.
#[derive(clap::Args)]
struct ContentArgs {
    /// What goes straight onto the map: `safe` leaves every object where the build puts it,
    /// `balanced` (default) puts the objects of the cadastre and the registers on the map, `full`
    /// puts everything on the map. The choice is saved in freemapper.toml
    #[arg(long, value_name = "PROFILE")]
    content: Option<String>,
    /// Send a group of objects to `map`, `candidates` or `off`, as `GROUP=PLACE`, e.g.
    /// `walls=map`; may be repeated. Groups: walls, fences, hedges, slopes, stairs, trees,
    /// water-features, monuments, furniture, passages, contours, landforms, vegetation,
    /// out-of-bounds, paths, streets. Saved in freemapper.toml
    #[arg(long = "content-set", value_name = "GROUP=PLACE")]
    content_set: Vec<String>,
}

impl ContentArgs {
    /// Saves what was asked for in the project of `dir`; nothing when nothing was asked.
    fn save(&self, dir: &std::path::Path) -> anyhow::Result<()> {
        let existing = ProjectFile::load(dir)?;
        let settings = ContentSettings::from_request(
            self.content.as_deref(),
            &self.content_set,
            existing.content.as_ref(),
        )
        .map_err(|detail| anyhow::anyhow!(text("cli-content-invalid", &[("detail", v(detail))])))?;
        if let Some(settings) = settings {
            let summary = settings.summary();
            set_content(dir, Some(settings))?;
            println!("{}", text("cli-content-saved", &[("summary", v(summary))]));
        }
        Ok(())
    }
}

/// The contour interval (ADR 0116): saved in `freemapper.toml`, used by the next derive.
#[derive(clap::Args)]
struct ContourArgs {
    /// Contour interval in metres, e.g. `2.5`, or `auto` for the map kind's (a forest map: 5 m,
    /// 2.5 m where it is flat, 10 m where it is steep). Saved in freemapper.toml; derive traces
    /// the contours at it
    #[arg(long = "contour-interval", value_name = "METRES")]
    contour_interval: Option<String>,
    /// Slope angle in degrees from which the terrain counts as steep when earth banks and cliffs
    /// are looked for, e.g. `55`, or `auto` for the default of 27°. Raise it (55 to 65) in rock
    /// towns and gorges, where almost every slope is steep and banks and cliffs run together
    /// into one patch. Saved in freemapper.toml; derive looks for banks and cliffs with it
    #[arg(long = "steep-slope", value_name = "DEGREES")]
    steep_slope: Option<String>,
    /// Slope angle in degrees from which the ground is a rock face (black area, symbol 206),
    /// e.g. `75`, or `auto` for the default of 70°. 65 gives more and larger faces, also on
    /// steep slopes; 75 keeps only true walls. Saved in freemapper.toml; derive looks for rock
    /// faces with it
    #[arg(long = "face-slope", value_name = "DEGREES")]
    face_slope: Option<String>,
}

impl ContourArgs {
    /// Saves the interval asked for in the project of `dir`; nothing when none was asked.
    fn save(&self, dir: &std::path::Path) -> anyhow::Result<()> {
        let Some(asked) = &self.contour_interval else {
            return self.save_slopes(dir);
        };
        let interval = parse_contour_interval(asked)?;
        set_contour_interval(dir, interval)?;
        match interval {
            Some(interval) => println!(
                "{}",
                text("cli-contour-interval-saved", &[("interval", v(interval))])
            ),
            None => println!("{}", text("cli-contour-interval-auto", &[])),
        }
        self.save_slopes(dir)
    }

    /// Saves the steep slope and the face slope asked for.
    fn save_slopes(&self, dir: &std::path::Path) -> anyhow::Result<()> {
        self.save_steep_slope(dir)?;
        self.save_face_slope(dir)
    }

    /// Saves the face slope asked for; nothing when none was asked.
    fn save_face_slope(&self, dir: &std::path::Path) -> anyhow::Result<()> {
        let Some(asked) = &self.face_slope else {
            return Ok(());
        };
        let slope = parse_face_slope(asked)?;
        set_face_slope(dir, slope)?;
        match slope {
            Some(slope) => println!("{}", text("cli-face-slope-saved", &[("slope", v(slope))])),
            None => println!("{}", text("cli-face-slope-auto", &[])),
        }
        Ok(())
    }

    /// Saves the steep slope asked for; nothing when none was asked.
    fn save_steep_slope(&self, dir: &std::path::Path) -> anyhow::Result<()> {
        let Some(asked) = &self.steep_slope else {
            return Ok(());
        };
        let slope = parse_steep_slope(asked)?;
        set_steep_slope(dir, slope)?;
        match slope {
            Some(slope) => println!("{}", text("cli-steep-slope-saved", &[("slope", v(slope))])),
            None => println!("{}", text("cli-steep-slope-auto", &[])),
        }
        Ok(())
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum SourcesArg {
    Auto,
    Full,
    CreditOnly,
}

impl From<SourcesArg> for PdfSources {
    fn from(arg: SourcesArg) -> Self {
        match arg {
            SourcesArg::Auto => PdfSources::Auto,
            SourcesArg::Full => PdfSources::Full,
            SourcesArg::CreditOnly => PdfSources::CreditOnly,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum UpdateMode {
    Notify,
    Install,
    Off,
}

impl From<UpdateMode> for freemapper_update::Mode {
    fn from(mode: UpdateMode) -> Self {
        match mode {
            UpdateMode::Notify => Self::Notify,
            UpdateMode::Install => Self::Install,
            UpdateMode::Off => Self::Off,
        }
    }
}

/// A setting that is on or off.
#[derive(Clone, Copy, ValueEnum)]
enum OnOff {
    On,
    Off,
}

#[derive(Clone, Copy, ValueEnum)]
enum Kind {
    Schoolyard,
    Sprint,
    Forest,
}

impl From<Kind> for MapKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Schoolyard => MapKind::Schoolyard,
            Kind::Sprint => MapKind::Sprint,
            Kind::Forest => MapKind::Forest,
        }
    }
}

/// Whether the program was most likely started by a double click in the file manager: on Windows,
/// without any argument, on a console. A bare `freemapper` typed in a terminal looks the same.
fn opened_by_double_click(
    windows: bool,
    arg_count: usize,
    stdin_tty: bool,
    stdout_tty: bool,
) -> bool {
    windows && arg_count == 1 && stdin_tty && stdout_tty
}

/// The project folder of a double click: the first `karte-<date>` (then `karte-<date>-2`, …) in
/// `<root>` that does not exist yet, so a second start does not reopen the first map.
fn double_click_folder(root: &std::path::Path, today: chrono::NaiveDate) -> PathBuf {
    let base = format!("karte-{today}");
    let mut folder = root.join(&base);
    let mut number = 2;
    while folder.exists() {
        folder = root.join(format!("{base}-{number}"));
        number += 1;
    }
    folder
}

/// The arguments a double click stands for: `select <folder>`, with the folder below the person's
/// documents (else the home folder, else the working folder). Said in a line first, because the
/// console window is all the person sees of the program besides the browser.
fn double_click_args(program: std::ffi::OsString) -> Vec<std::ffi::OsString> {
    let root = dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_default()
        .join("freemapper");
    let folder = double_click_folder(&root, chrono::Local::now().date_naive());
    println!(
        "{}",
        text("cli-double-click-start", &[("path", v(folder.display()))])
    );
    vec![program, "select".into(), folder.into_os_string()]
}

fn main() -> std::process::ExitCode {
    // The language first: the help and the parse errors need it.
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if let Err(message) = text::set_lang(help::lang_from_args(&args).as_deref()) {
        // A usage error, as clap reports one.
        eprintln!("error: {message}");
        return std::process::ExitCode::from(2);
    }
    let lang = text::lang();
    let usage_error = |message: &str| {
        eprintln!("{}: {message}", text("cli-error-prefix", &[]));
        eprintln!("\n{}", text("cli-parse-hint", &[]));
        std::process::ExitCode::from(2)
    };
    let parse_error = |error: clap::Error| {
        if !error.use_stderr() {
            // Help and version.
            let _ = error.print();
            return std::process::ExitCode::SUCCESS;
        }
        match help::parse_error_message(&lang, &error) {
            Some(message) => usage_error(&message),
            None => {
                let _ = error.print();
                std::process::ExitCode::from(2)
            }
        }
    };
    let mut command = help::localized(Cli::command(), &lang);
    let args = if opened_by_double_click(
        cfg!(windows),
        args.len(),
        std::io::stdin().is_terminal(),
        std::io::stdout().is_terminal(),
    ) {
        double_click_args(args.into_iter().next().unwrap_or_default())
    } else {
        args
    };
    let matches = match command.clone().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(error) => return parse_error(error),
    };
    if let Some(("help", sub)) = matches.subcommand() {
        let path: Vec<String> = sub
            .get_many::<String>("command")
            .map(|words| words.cloned().collect())
            .unwrap_or_default();
        return match help::help_of(&mut command, &path) {
            Ok(help) => {
                print!("{help}");
                std::process::ExitCode::SUCCESS
            }
            Err(word) => usage_error(&text("cli-parse-invalid-subcommand", &[("value", v(word))])),
        };
    }
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => return parse_error(error),
    };
    // The MCP server's stdout is the protocol, and a stale file may be an older server still
    // running; it is left alone.
    if !matches!(cli.command, Commands::Mcp { .. }) {
        update::background::remove_stale_files();
    }
    let check = cli.command.update_check().and_then(|(cache, offline)| {
        update::background::start_background_check(&update::background::CheckContext::of_command(
            cache, offline,
        ))
    });
    let result = run(cli);
    let command_ok = result.is_ok();
    let code = match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!(
                "{}: {}",
                text("cli-error-prefix", &[]),
                errors::message(error.as_ref(), &text::lang())
            );
            std::process::ExitCode::FAILURE
        }
    };
    // After the command's own output, its error included.
    if let Some(report) = check.and_then(|check| check.finish(command_ok)) {
        report.print();
    }
    code
}

/// Runs the command; an error comes back for `main` to word in the language of the run.
fn run(cli: Cli) -> anyhow::Result<std::process::ExitCode> {
    // The adapters of this run, before anything asks the registry. The list of the adapters
    // assembles its own, and an update asks none.
    let mut adapter_lines = Vec::new();
    // The signed list of corrections of addresses (issue #273, stage 4) comes before anything
    // reads a manifest, the list of the adapters and `sources` included. A command that uses the
    // network anyway may fetch a newer list first, once a day, when the person turned that on.
    if !matches!(cli.command, Commands::Update { .. } | Commands::Mcp { .. })
        && let Some((_, offline)) = cli.command.update_check()
    {
        corrections::refresh_daily_for_command(offline);
    }
    adapter_lines.extend(corrections::activate());
    if !matches!(
        cli.command,
        Commands::Adapters { .. } | Commands::Sources { .. } | Commands::Update { .. }
    ) {
        // The MCP server and the area page read the person's settings and never write them.
        let may_write = !matches!(cli.command, Commands::Mcp { .. } | Commands::Select { .. });
        adapter_lines.extend(adapters::install(cli.command.project_adapters(), may_write));
    }
    match cli.command {
        Commands::Init {
            dir,
            area,
            part,
            kind,
            scale,
            title,
            content,
            contours,
            adapters: _,
        } => {
            let area = area.area(part.map(|p| p as usize))?;
            // A bad interval is refused before the folder is made.
            if let Some(asked) = &contours.contour_interval {
                parse_contour_interval(asked)?;
            }
            if let Some(asked) = &contours.steep_slope {
                parse_steep_slope(asked)?;
            }
            if let Some(asked) = &contours.face_slope {
                parse_face_slope(asked)?;
            }
            let today = chrono::Local::now().date_naive();
            let project = init_project(&dir, &area, kind.into(), scale, today, title)?;
            println!(
                "{}",
                text("cli-created-project", &[("path", v(dir.display()))])
            );
            content.save(&dir)?;
            contours.save(&dir)?;
            if let Some(note) = project.map_kind.scale_note(project.scale_denominator) {
                println!("{}", pipeline::line(&note));
            }
        }
        Commands::Export {
            dir,
            force,
            ocad,
            pdf,
            pdf_candidates,
            training,
            view,
            pdf_scale_bar,
            pdf_sources,
            content,
            adapters: _,
        } => {
            let views =
                pipeline::parse_views(&view, &crate::text::lang()).map_err(anyhow::Error::msg)?;
            let _lock = lock_folder(&dir)?;
            content.save(&dir)?;
            // Built once for every output of this command (ADR 0091).
            let documents = starter_documents(&dir)?;
            let existing = map_path(&dir);
            if (ocad || pdf || pdf_candidates || training) && existing.exists() && !force {
                kept(&existing);
                // The kept map is not touched, but its paper frame template follows the project
                // file, as on every export.
                if write_paper_frame_template(&dir, &documents)? {
                    if let Some(note) = documents.paper_frame_note() {
                        report_note(note);
                    }
                    let map_text = String::from_utf8_lossy(&std::fs::read(&existing)?).into_owned();
                    if kept_map_lacks_paper_frames(&documents, &map_text) {
                        println!("{}", text("cli-paper-frame-kept-map", &[]));
                    }
                }
            } else {
                wrote(&export_starter_project_from(&dir, &documents, force)?);
                if let Some(note) = documents.paper_frame_note() {
                    report_note(note);
                }
                if let Some(line) = pipeline::light_summary(documents.light(), "cli-map-light") {
                    println!("{line}");
                }
                // The map is written: a failed preview is a warning, and the outputs still run.
                if let Some(preview) =
                    export_map_preview_or_warn(&dir, &documents, &mut report_note)
                {
                    wrote(&preview);
                }
            }
            if ocad {
                report_ocad(&export_ocad_from(&dir, &documents, force)?);
            }
            if pdf {
                report_pdf(
                    &export_pdf_from(&dir, &documents, pdf_scale_bar, pdf_sources.into())?,
                    false,
                    pdf_sources.into(),
                );
            }
            if pdf_candidates {
                report_pdf(
                    &export_candidates_pdf_from(
                        &dir,
                        &documents,
                        pdf_scale_bar,
                        pdf_sources.into(),
                    )?,
                    true,
                    pdf_sources.into(),
                );
            }
            if training {
                let outcome = export_training_from(
                    &dir,
                    &documents,
                    force,
                    pdf_scale_bar,
                    pdf_sources.into(),
                    ocad,
                )?;
                report_training(&outcome, pdf_sources.into());
                for warning in pipeline::training_warnings(&outcome) {
                    report(&warning);
                }
                if !views.is_empty() {
                    for files in export_training_views(
                        &dir,
                        &documents,
                        &views,
                        force,
                        pdf_scale_bar,
                        pdf_sources.into(),
                        ocad,
                    )? {
                        println!(
                            "{}",
                            text(
                                "cli-summary-training-view",
                                &[
                                    ("map", v(files.map.display())),
                                    ("pdf", v(files.pdf.display())),
                                ],
                            )
                        );
                    }
                }
            }
        }
        Commands::Fetch {
            dir,
            offline,
            cache,
            adapters: _,
        } => {
            let _lock = lock_folder(&dir)?;
            let project = fetch_project(
                &dir,
                &HttpFetcher::new(),
                &cache.cache()?,
                mode(offline),
                &mut report_note,
            )?;
            println!(
                "{}",
                text("cli-fetched-tiles", &[("count", n(project.tiles.len()))])
            );
        }
        Commands::Derive {
            dir,
            contours,
            cache,
            adapters: _,
        } => {
            let _lock = lock_folder(&dir)?;
            contours.save(&dir)?;
            for path in derive_project(&dir, &cache.cache()?, &mut report_note)? {
                wrote(&path);
            }
        }
        Commands::Run {
            dir,
            offline,
            force,
            ocad,
            pdf,
            pdf_candidates,
            training,
            view,
            pdf_scale_bar,
            pdf_sources,
            permits,
            content,
            contours,
            cache,
            adapters: _,
        } => {
            let training_views =
                pipeline::parse_views(&view, &crate::text::lang()).map_err(anyhow::Error::msg)?;
            let _lock = lock_folder(&dir)?;
            content.save(&dir)?;
            contours.save(&dir)?;
            let options = pipeline::RunOptions {
                force,
                offline,
                ocad,
                pdf,
                pdf_candidates,
                training,
                training_views,
                pdf_scale_bar,
                pdf_sources: pdf_sources.into(),
                permits,
                fail_fast: true,
            };
            pipeline::run(
                &dir,
                &options,
                &cache.cache()?,
                &HttpFetcher::new(),
                &pipeline::RunControl::default(),
                &mut |event| match event {
                    pipeline::Event::Line(line) | pipeline::Event::Warning(line) => report(line),
                    pipeline::Event::Phase { .. } => {}
                    pipeline::Event::Written(written) => {
                        print_written(written, options.pdf_sources)
                    }
                },
            )?;
        }
        Commands::Permits {
            dir,
            offline,
            cache,
        } => {
            let written = permits::run_permits(
                &dir,
                offline,
                &cache.cache()?,
                &HttpFetcher::new(),
                lock::Who::Cli,
                &text::lang(),
                &mut report_note,
            )?;
            for warning in &written.warnings {
                report(warning);
            }
            wrote_permits(&written.markdown);
        }
        Commands::Select {
            dir,
            kind,
            scale,
            title,
            no_browser,
            port,
            offline,
            ocad,
            pdf,
            pdf_candidates,
            training,
            pdf_scale_bar,
            pdf_sources,
            cache,
        } => {
            let cache = cache.cache()?;
            let defaults = select::Defaults {
                kind: kind.into(),
                scale,
                title,
                from_agent: false,
                run: pipeline::RunOptions {
                    offline,
                    ocad,
                    pdf,
                    pdf_candidates,
                    training,
                    pdf_scale_bar,
                    pdf_sources: pdf_sources.into(),
                    ..Default::default()
                },
            };
            tokio::runtime::Runtime::new()?.block_on(async {
                let selector = select::Selector::start(
                    dir.clone(),
                    defaults,
                    cache,
                    port,
                    Some(std::sync::Arc::new(jobs::JobHub::default())),
                )
                .await?;
                println!("{}", selector.url);
                std::io::Write::flush(&mut std::io::stdout())?;
                eprintln!(
                    "{}",
                    text("cli-select-running", &[("path", v(dir.display()))])
                );
                if !no_browser && let Err(e) = select::open_browser(&selector.url) {
                    eprintln!("{}", text("cli-select-no-browser", &[("error", v(e))]));
                }
                tokio::select! {
                    finished = selector.finished() => finished?,
                    _ = tokio::signal::ctrl_c() => {}
                }
                anyhow::Ok(())
            })?;
        }
        Commands::Find { query, json } => find::run(&query, json)?,
        Commands::Adapters {
            command: AdaptersCommand::List { json, project },
        } => adapters::list(json, project.as_deref()),
        Commands::Adapters {
            command:
                AdaptersCommand::Check {
                    dir,
                    live,
                    bbox,
                    json,
                    cache,
                },
        } => {
            return adapter_check::run(&dir, bbox.filter(|_| live), json, || cache.cache());
        }
        Commands::Adapters {
            command: AdaptersCommand::Probe { json, status },
        } => {
            let (targets, hosts) = freemapper::probe::builtin_targets();
            let outcomes = freemapper::probe::probe(&HttpFetcher::new(), &targets, &hosts);
            if let Some(file) = status {
                let previous = std::fs::read_to_string(&file)
                    .ok()
                    .and_then(|text| serde_json::from_str(&text).ok())
                    .unwrap_or_default();
                let today = freemapper::clock::today().format("%Y-%m-%d").to_string();
                let status = freemapper::service_status::update(&outcomes, &previous, &today);
                for line in freemapper::service_status::transitions(&status) {
                    eprintln!("{line}");
                }
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else if json {
                println!("{}", freemapper::probe::to_json(&outcomes));
            } else {
                for line in freemapper::probe::lines(&outcomes) {
                    println!("{line}");
                }
            }
        }
        Commands::Sources { command } => match command {
            SourcesCommand::List { adapter, json } => sources::list(adapter.as_deref(), json)?,
            SourcesCommand::Get { adapter, path } => sources::get(&adapter, &path)?,
            SourcesCommand::Set {
                adapter,
                path,
                url,
                add_host,
            } => sources::set(&adapter, &path, &url, add_host)?,
            SourcesCommand::Diff { adapter } => sources::diff(&adapter)?,
            SourcesCommand::Check {
                adapter,
                live,
                bbox,
                json,
                cache,
            } => {
                return sources::check(&adapter, bbox.filter(|_| live), json, || cache.cache());
            }
            SourcesCommand::Schema { json } => sources::schema(json)?,
            SourcesCommand::Update { mode, remove } => {
                corrections::update(mode.map(|mode| matches!(mode, OnOff::On)), remove)?
            }
            SourcesCommand::Corrections { json } => corrections::show(json)?,
        },
        Commands::Adapters {
            command: AdaptersCommand::Export { adapter, dir },
        } => adapters::export(&adapter, &dir)?,
        Commands::Adapters {
            command: AdaptersCommand::AllowHost { adapter, host },
        } => adapters::change_allowed_host(&adapter, &host, true)?,
        Commands::Adapters {
            command: AdaptersCommand::DisallowHost { adapter, host },
        } => adapters::change_allowed_host(&adapter, &host, false)?,
        Commands::Mcp { cache } => {
            let cache = cache.cache()?;
            tokio::runtime::Runtime::new()?.block_on(mcp::serve(cache, adapter_lines))?;
        }
        Commands::Update {
            check,
            yes,
            version,
            rollback,
            mode,
        } => {
            return update::command::run(update::command::Request {
                check,
                yes,
                version,
                rollback,
                mode: mode.map(Into::into),
            });
        }
    }
    Ok(std::process::ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_start_on_a_windows_console_counts_as_a_double_click() {
        assert!(opened_by_double_click(true, 1, true, true));
        // Another system, an argument, a pipe in or out: an ordinary run.
        assert!(!opened_by_double_click(false, 1, true, true));
        assert!(!opened_by_double_click(true, 2, true, true));
        assert!(!opened_by_double_click(true, 1, false, true));
        assert!(!opened_by_double_click(true, 1, true, false));
    }

    #[test]
    fn a_double_click_takes_the_first_free_folder_of_the_day() {
        let root = tempfile::tempdir().unwrap();
        let day = chrono::NaiveDate::from_ymd_opt(2026, 11, 21).unwrap();
        let first = double_click_folder(root.path(), day);
        assert_eq!(first, root.path().join("karte-2026-11-21"));
        std::fs::create_dir(&first).unwrap();
        let second = double_click_folder(root.path(), day);
        assert_eq!(second, root.path().join("karte-2026-11-21-2"));
        std::fs::create_dir(&second).unwrap();
        assert_eq!(
            double_click_folder(root.path(), day),
            root.path().join("karte-2026-11-21-3")
        );
    }

    #[test]
    fn select_takes_the_run_flags_as_page_defaults() {
        let cli = Cli::try_parse_from([
            "freemapper",
            "select",
            "/tmp/p",
            "--ocad",
            "--pdf",
            "--pdf-candidates",
            "--training",
            "--pdf-scale-bar",
            "--pdf-sources",
            "full",
            "--offline",
        ])
        .unwrap();
        let Commands::Select {
            offline,
            ocad,
            pdf,
            pdf_candidates,
            training,
            pdf_scale_bar,
            pdf_sources,
            ..
        } = cli.command
        else {
            panic!("not select");
        };
        assert!(offline && ocad && pdf && pdf_candidates && training && pdf_scale_bar);
        assert_eq!(PdfSources::from(pdf_sources), PdfSources::Full);
        // Without flags the page starts like `run`: everything off, sources automatic.
        let cli = Cli::try_parse_from(["freemapper", "select", "/tmp/p"]).unwrap();
        let Commands::Select {
            ocad, pdf_sources, ..
        } = cli.command
        else {
            panic!("not select");
        };
        assert!(!ocad);
        assert_eq!(PdfSources::from(pdf_sources), PdfSources::Auto);
    }

    /// `--adapters-from-project` belongs to the commands that work on a project folder from
    /// the terminal. It names the folder whose adapters are loaded.
    #[test]
    fn project_adapters_are_loaded_only_with_the_switch() {
        let project = |args: &[&str]| {
            let cli = Cli::try_parse_from(args).unwrap();
            cli.command
                .project_adapters()
                .map(std::path::Path::to_path_buf)
        };
        for command in ["export", "fetch", "derive", "run"] {
            assert_eq!(
                project(&["freemapper", command, "/tmp/p"]),
                None,
                "{command}"
            );
            assert_eq!(
                project(&["freemapper", command, "/tmp/p", "--adapters-from-project"]),
                Some(PathBuf::from("/tmp/p")),
                "{command}"
            );
        }
        let init = ["freemapper", "init", "/tmp/p", "--osm", "way/1"];
        assert_eq!(project(&init), None);
        assert_eq!(
            project(&[&init[..], &["--adapters-from-project"]].concat()),
            Some(PathBuf::from("/tmp/p"))
        );
    }

    /// The MCP server works for an agent on folders the agent names: it never loads the
    /// adapters of a project, and has no switch that would.
    #[test]
    fn mcp_ignores_project_adapters() {
        let cli = Cli::try_parse_from(["freemapper", "mcp"]).unwrap();
        assert_eq!(cli.command.project_adapters(), None);
        assert!(Cli::try_parse_from(["freemapper", "mcp", "--adapters-from-project"]).is_err());
    }

    /// A run started from the area page is like one of the MCP server: no project adapters.
    #[test]
    fn page_run_ignores_project_adapters() {
        let cli = Cli::try_parse_from(["freemapper", "select", "/tmp/p"]).unwrap();
        assert_eq!(cli.command.project_adapters(), None);
        assert!(
            Cli::try_parse_from(["freemapper", "select", "/tmp/p", "--adapters-from-project"])
                .is_err()
        );
    }

    /// The daily update check follows the commands that use the network anyway, not the ones
    /// that read or write the adapter settings.
    #[test]
    fn the_daily_update_check_follows_the_network_commands() {
        let check = |args: &[&str]| {
            Cli::try_parse_from(args.iter().copied())
                .unwrap()
                .command
                .update_check()
        };
        assert!(check(&["freemapper", "find", "x"]).is_some());
        assert!(check(&["freemapper", "fetch", "/tmp/p"]).is_some());
        assert_eq!(
            check(&["freemapper", "fetch", "/tmp/p", "--offline"]).map(|(_, offline)| offline),
            Some(true)
        );
        for args in [
            &["freemapper", "sources", "list"][..],
            &["freemapper", "sources", "get", "a", "b"],
            &["freemapper", "adapters", "list"],
            &["freemapper", "derive", "/tmp/p"],
        ] {
            assert!(check(args).is_none(), "{args:?}");
        }
    }

    #[test]
    fn adapters_list_takes_json_and_a_project() {
        let cli = Cli::try_parse_from(["freemapper", "adapters", "list"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Adapters {
                command: AdaptersCommand::List {
                    json: false,
                    project: None
                }
            }
        ));
        let cli = Cli::try_parse_from([
            "freemapper",
            "adapters",
            "list",
            "--json",
            "--project",
            "/tmp/p",
        ])
        .unwrap();
        let Commands::Adapters {
            command: AdaptersCommand::List { json, project },
        } = cli.command
        else {
            panic!("not adapters list");
        };
        assert!(json);
        assert_eq!(project, Some(PathBuf::from("/tmp/p")));
        // `adapters` alone asks for a subcommand.
        assert!(Cli::try_parse_from(["freemapper", "adapters"]).is_err());
    }
}
