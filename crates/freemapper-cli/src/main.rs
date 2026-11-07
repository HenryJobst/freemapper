//! `freemapper` command line interface.

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use freemapper::area::MapArea;
use freemapper::project::{MapKind, export_starter_project, init_project};

#[derive(Parser)]
#[command(
    name = "freemapper",
    version,
    about = "Builds OOM starter projects from geodata"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
        #[arg(long)]
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
        Some(Commands::Init {
            dir,
            geojson,
            kind,
            scale,
        }) => {
            let text = std::fs::read_to_string(&geojson)
                .with_context(|| format!("reading {}", geojson.display()))?;
            let area = MapArea::from_geojson(&text)?;
            let today = chrono::Local::now().date_naive();
            init_project(&dir, &area, kind.into(), scale, today)?;
            println!("Created project {}", dir.display());
        }
        Some(Commands::Export { dir, force }) => {
            let map = export_starter_project(&dir, force)?;
            println!("Wrote {}", map.display());
        }
        None => {}
    }
    Ok(())
}
