//! `freemapper` command line interface.

use clap::{Parser, Subcommand};

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
enum Commands {}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {}
    }
    Ok(())
}
