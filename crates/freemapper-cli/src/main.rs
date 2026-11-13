//! `freemapper` command line interface.

mod adapter_check;
mod adapters;
mod errors;
mod estimate;
mod find;
mod help;
mod hosts;
mod jobs;
mod lock;
mod mcp;
mod nominatim;
mod pipeline;
mod select;
mod select_page;
mod select_preview;
mod select_run;
mod text;
mod tiles;
mod update;

use std::path::PathBuf;

use anyhow::Context;
use clap::{CommandFactory as _, FromArgMatches as _, Parser, Subcommand, ValueEnum};
use freemapper::area::MapArea;
use freemapper::build::content::ContentSettings;
use freemapper::fetch::{Cache, FetchMode, HttpFetcher};
use freemapper::places::{find_places, place_area};
use freemapper::project::{
    MapKind, MapOutcome, OcadOutcome, PdfSources, ProjectFile, derive_project,
    export_candidates_pdf_from, export_map_preview_or_warn, export_ocad_from, export_pdf_from,
    export_starter_project_from, fetch_project, init_project, map_path, set_content,
    starter_documents,
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
        /// Also write `<name>-candidates.pdf`, the training map (the map plus every candidate
        /// drawn as a map object); an existing OOM map then stays unless you pass --force.
        #[arg(long)]
        pdf_candidates: bool,
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
        /// Also write `<name>-candidates.pdf`, the training map (the map plus every candidate).
        #[arg(long)]
        pdf_candidates: bool,
        /// Draw the scale bar on the PDFs (off by default; the scale number stays in the title block).
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
        cache: CacheArgs,
        #[command(flatten)]
        adapters: AdapterArgs,
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
        /// The page starts with the training PDF (every candidate) ticked.
        #[arg(long)]
        pdf_candidates: bool,
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
            | Commands::Select { offline, cache, .. } => Some((cache.folder(), *offline)),
            Commands::Export { .. }
            | Commands::Derive { .. }
            | Commands::Mcp { .. }
            | Commands::Adapters { .. }
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

/// What the OCAD export wrote and what the user does next in OCAD.
fn report_ocad(outcome: &OcadOutcome) {
    for warning in &outcome.warnings {
        eprintln!("{}", pipeline::line(warning));
    }
    let map = ("path", v(outcome.map.display()));
    if outcome.kept {
        println!("{}", text("cli-ocad-kept", &[map]));
    } else {
        println!(
            "{}",
            text("cli-ocad-wrote", &[map, ("count", n(outcome.objects))])
        );
    }
    wrote(&outcome.candidates);
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
    println!("{}", text("cli-ocad-load-symbols", &[]));
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
                }
                MapOutcome::Kept {
                    map,
                    references_images,
                    missing_buildings,
                    missing_canopies,
                } => {
                    kept(map);
                    if !references_images {
                        println!("{}", text("cli-map-lacks-templates", &[]));
                    }
                    if *missing_buildings {
                        println!("{}", text("cli-map-lacks-buildings", &[]));
                    }
                    if *missing_canopies {
                        println!("{}", text("cli-map-lacks-canopies", &[]));
                    }
                }
            }
        }
        pipeline::Written::Ocad(outcome) => report_ocad(outcome),
        pipeline::Written::Pdf(path) => report_pdf(path, false, sources),
        pipeline::Written::PdfCandidates(path) => report_pdf(path, true, sources),
    }
}

fn report(message: &str) {
    eprintln!("{message}");
}

/// Prints a progress line or a warning of the engine in the language of the run.
fn report_note(note: freemapper::note::Note) {
    report(&pipeline::line(&note));
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
    if !matches!(
        cli.command,
        Commands::Adapters { .. } | Commands::Update { .. }
    ) {
        // The MCP server and the area page read the person's settings and never write them.
        let may_write = !matches!(cli.command, Commands::Mcp { .. } | Commands::Select { .. });
        adapter_lines = adapters::install(cli.command.project_adapters(), may_write);
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
            adapters: _,
        } => {
            let area = area.area(part.map(|p| p as usize))?;
            let today = chrono::Local::now().date_naive();
            let project = init_project(&dir, &area, kind.into(), scale, today, title)?;
            println!(
                "{}",
                text("cli-created-project", &[("path", v(dir.display()))])
            );
            content.save(&dir)?;
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
            pdf_scale_bar,
            pdf_sources,
            content,
            adapters: _,
        } => {
            let _lock = lock_folder(&dir)?;
            content.save(&dir)?;
            // Built once for every output of this command (ADR 0091).
            let documents = starter_documents(&dir)?;
            let existing = map_path(&dir);
            if (ocad || pdf || pdf_candidates) && existing.exists() && !force {
                kept(&existing);
            } else {
                wrote(&export_starter_project_from(&dir, &documents, force)?);
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
            cache,
            adapters: _,
        } => {
            let _lock = lock_folder(&dir)?;
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
            pdf_scale_bar,
            pdf_sources,
            content,
            cache,
            adapters: _,
        } => {
            let _lock = lock_folder(&dir)?;
            content.save(&dir)?;
            let options = pipeline::RunOptions {
                force,
                offline,
                ocad,
                pdf,
                pdf_candidates,
                pdf_scale_bar,
                pdf_sources: pdf_sources.into(),
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
    fn select_takes_the_run_flags_as_page_defaults() {
        let cli = Cli::try_parse_from([
            "freemapper",
            "select",
            "/tmp/p",
            "--ocad",
            "--pdf",
            "--pdf-candidates",
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
            pdf_scale_bar,
            pdf_sources,
            ..
        } = cli.command
        else {
            panic!("not select");
        };
        assert!(offline && ocad && pdf && pdf_candidates && pdf_scale_bar);
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
