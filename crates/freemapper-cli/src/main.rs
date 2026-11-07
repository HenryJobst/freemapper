//! `freemapper` command line interface.

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use freemapper::area::MapArea;
use freemapper::fetch::{Cache, FetchMode, HttpFetcher};
use freemapper::project::{
    MapKind, MapOutcome, derive_project, export_starter_project, fetch_project, init_project,
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
        /// GeoJSON file with the map area polygon (EPSG:4326).
        #[arg(long)]
        geojson: PathBuf,
        /// Map kind.
        #[arg(long, value_enum, default_value_t = Kind::Schoolyard)]
        kind: Kind,
        /// Scale denominator, e.g. 1000 for 1:1000 (default depends on the map kind).
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        scale: Option<u32>,
    },
    /// Write the starter project (OOM map, templates, attribution) into a project folder.
    Export {
        /// Project folder.
        dir: PathBuf,
        /// Replace an existing map file (the old one is kept as `<name>.omap.bak`).
        #[arg(long)]
        force: bool,
    },
    /// Download the tiles for the map area (orthophoto, terrain model) into the cache.
    Fetch {
        /// Project folder.
        dir: PathBuf,
        /// Use the cache only, without network access.
        #[arg(long)]
        offline: bool,
        #[command(flatten)]
        cache: CacheArgs,
    },
    /// Compute the templates (orthophoto, hillshade, slope) from the fetched tiles.
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
        /// Replace an existing map file (the old one is kept as `<name>.omap.bak`).
        #[arg(long)]
        force: bool,
        #[command(flatten)]
        cache: CacheArgs,
    },
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

fn report(message: &str) {
    eprintln!("{message}");
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
            geojson,
            kind,
            scale,
        } => {
            let text = std::fs::read_to_string(&geojson)
                .with_context(|| format!("reading {}", geojson.display()))?;
            let area = MapArea::from_geojson(&text)?;
            let today = chrono::Local::now().date_naive();
            init_project(&dir, &area, kind.into(), scale, today)?;
            println!("Created project {}", dir.display());
        }
        Commands::Export { dir, force } => {
            let map = export_starter_project(&dir, force)?;
            println!("Wrote {}", map.display());
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
                } => {
                    println!("Kept {} (it may contain your work)", map.display());
                    if !references_images {
                        println!(
                            "The map does not reference the image templates yet: add them in OOM \
                             (Templates > Open template…) or replace the map with --force \
                             (the old one is kept as .omap.bak)."
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
