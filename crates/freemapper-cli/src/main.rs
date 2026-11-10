//! `freemapper` command line interface.

mod mcp;
mod nominatim;
mod select;
mod tiles;

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use freemapper::area::MapArea;
use freemapper::fetch::{Cache, FetchMode, HttpFetcher};
use freemapper::places::{find_places, place_area};
use freemapper::project::{
    MapKind, MapOutcome, OcadOutcome, PdfSources, derive_project, export_candidates_pdf,
    export_ocad, export_pdf, export_starter_project, fetch_project, init_project, map_path,
    run_project,
};

#[derive(Parser)]
#[command(
    name = "freemapper",
    version,
    about = "Builds OOM starter projects from geodata",
    arg_required_else_help = true
)]
struct Cli {
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
        /// or 5000. Other values are refused, because freemapper-print refuses them.
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        scale: Option<u32>,
        /// Map title for the peripheral information (default: the folder name).
        #[arg(long)]
        title: Option<String>,
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
    },
    /// Compute the templates (orthophoto, hillshade, slope, nDSM) and contours from the fetched data.
    Derive {
        /// Project folder.
        dir: PathBuf,
        #[command(flatten)]
        cache: CacheArgs,
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
        /// or 5000. Other values are refused, because freemapper-print refuses them.
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
        #[command(flatten)]
        cache: CacheArgs,
    },
    /// Serve the tools to an AI agent over MCP (stdin/stdout).
    Mcp {
        #[command(flatten)]
        cache: CacheArgs,
    },
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
            anyhow::bail!(
                "--part picks a part of an OSM area and needs --osm; with --place, look up the \
                 id first and use --osm <id> --part <n>"
            );
        }
        if let Some(geojson) = self.geojson {
            let text = std::fs::read_to_string(&geojson)
                .with_context(|| format!("reading {}", geojson.display()))?;
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
        let place = places.iter().find(|p| p.has_area).with_context(|| {
            format!(
                "no area found for {query:?}. If the place is an area of several separate \
                     parts, pick one with --osm <id> --part <n>, or use --geojson"
            )
        })?;
        eprintln!("Using {} ({})", place.display_name, place.osm_id);
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
            None => Cache::default_location()
                .context("no cache directory on this system; pass --cache-dir"),
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
        println!(
            "PDF without source credits: ODbL and dl-de/by-2-0 ask you to name the sources when you pass the map on; see ATTRIBUTION.txt"
        );
    }
    if candidates {
        println!(
            "Wrote {} (training map with every candidate drawn as a map object; check it in the terrain)",
            path.display()
        );
    } else {
        println!("Wrote {} (the map without candidates)", path.display());
    }
}

/// What the OCAD export wrote and what the user does next in OCAD.
fn report_ocad(outcome: &OcadOutcome) {
    for warning in &outcome.warnings {
        eprintln!("{warning}");
    }
    if outcome.kept {
        println!(
            "Kept {} (it may contain your work; --force replaces it and keeps the old one as .ocd.bak)",
            outcome.map.display()
        );
    } else {
        println!(
            "Wrote {} ({} objects)",
            outcome.map.display(),
            outcome.objects
        );
    }
    println!("Wrote {}", outcome.candidates.display());
    if outcome.skipped > 0 || outcome.dropped_parts > 0 {
        println!(
            "Left out of the OCAD map: {} objects (no symbol for OCAD, no area, or outside OCAD's \
             drawing area of ±2 m) and {} degenerate line parts or holes.",
            outcome.skipped, outcome.dropped_parts
        );
    }
    println!(
        "In OCAD, choose Symbol > Load Colors and Symbols From… and pick the official ISSprOM \
         2019 or school symbol set; it replaces freemapper's placeholder symbols by number."
    );
}

fn report(message: &str) {
    eprintln!("{message}");
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
enum Kind {
    Schoolyard,
    Sprint,
}

impl From<Kind> for MapKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Schoolyard => MapKind::Schoolyard,
            Kind::Sprint => MapKind::Sprint,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init {
            dir,
            area,
            part,
            kind,
            scale,
            title,
        } => {
            let area = area.area(part.map(|p| p as usize))?;
            let today = chrono::Local::now().date_naive();
            let project = init_project(&dir, &area, kind.into(), scale, today, title)?;
            println!("Created project {}", dir.display());
            if let Some(note) = project.map_kind.scale_note(project.scale_denominator) {
                println!("{note}");
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
        } => {
            let existing = map_path(&dir);
            if (ocad || pdf || pdf_candidates) && existing.exists() && !force {
                println!("Kept {} (it may contain your work)", existing.display());
            } else {
                let map = export_starter_project(&dir, force)?;
                println!("Wrote {}", map.display());
            }
            if ocad {
                report_ocad(&export_ocad(&dir, force)?);
            }
            if pdf {
                report_pdf(
                    &export_pdf(&dir, pdf_scale_bar, pdf_sources.into())?,
                    false,
                    pdf_sources.into(),
                );
            }
            if pdf_candidates {
                report_pdf(
                    &export_candidates_pdf(&dir, pdf_scale_bar, pdf_sources.into())?,
                    true,
                    pdf_sources.into(),
                );
            }
        }
        Commands::Fetch {
            dir,
            offline,
            cache,
        } => {
            let project = fetch_project(
                &dir,
                &HttpFetcher::new(),
                &cache.cache()?,
                mode(offline),
                &mut report,
            )?;
            println!("Fetched {} tiles", project.tiles.len());
        }
        Commands::Derive { dir, cache } => {
            for path in derive_project(&dir, &cache.cache()?, &mut report)? {
                println!("Wrote {}", path.display());
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
            cache,
        } => {
            let outcome = run_project(
                &dir,
                &HttpFetcher::new(),
                &cache.cache()?,
                mode(offline),
                force,
                &mut report,
            )?;
            for path in outcome.templates {
                println!("Wrote {}", path.display());
            }
            match outcome.map {
                MapOutcome::Written(map) => println!("Wrote {}", map.display()),
                MapOutcome::Kept {
                    map,
                    references_images,
                    missing_buildings,
                    missing_canopies,
                } => {
                    println!("Kept {} (it may contain your work)", map.display());
                    if !references_images {
                        println!(
                            "The map does not reference the image templates yet. Add them in \
                             OOM (Templates > Open template…) or replace the map with --force, \
                             which keeps the old one as .omap.bak."
                        );
                    }
                    if missing_buildings {
                        println!(
                            "The map has no ALKIS buildings yet. To add them, replace it with \
                             --force, which keeps the old one as .omap.bak."
                        );
                    }
                    if missing_canopies {
                        println!(
                            "The map lacks the ALKIS canopies and carports (522) that are now \
                             map objects. To add them, run with --force, which keeps the old map \
                             as .omap.bak."
                        );
                    }
                }
            }
            if ocad {
                report_ocad(&export_ocad(&dir, force)?);
            }
            if pdf {
                report_pdf(
                    &export_pdf(&dir, pdf_scale_bar, pdf_sources.into())?,
                    false,
                    pdf_sources.into(),
                );
            }
            if pdf_candidates {
                report_pdf(
                    &export_candidates_pdf(&dir, pdf_scale_bar, pdf_sources.into())?,
                    true,
                    pdf_sources.into(),
                );
            }
        }
        Commands::Select {
            dir,
            kind,
            scale,
            title,
            no_browser,
            port,
            cache,
        } => {
            let cache = cache.cache()?;
            let defaults = select::Defaults {
                kind: kind.into(),
                scale,
                title,
                from_agent: false,
            };
            tokio::runtime::Runtime::new()?.block_on(async {
                let selector =
                    select::Selector::start(dir.clone(), defaults, cache, port, None).await?;
                println!("{}", selector.url);
                std::io::Write::flush(&mut std::io::stdout())?;
                eprintln!(
                    "Area selection for {}: press \"Done\" on the page or Ctrl+C to stop.",
                    dir.display()
                );
                if !no_browser && let Err(e) = select::open_browser(&selector.url) {
                    eprintln!("Could not open the browser ({e}); open the address above.");
                }
                tokio::select! {
                    finished = selector.finished() => finished?,
                    _ = tokio::signal::ctrl_c() => {}
                }
                anyhow::Ok(())
            })?;
        }
        Commands::Mcp { cache } => {
            let cache = cache.cache()?;
            tokio::runtime::Runtime::new()?.block_on(mcp::serve(cache))?;
        }
    }
    Ok(())
}
