mod auth;
mod commands;
mod config;
mod git_store;
mod paths;
mod store;
mod sync;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{cmd_default, cmd_login, cmd_status, cmd_sync, cmd_turn_off, cmd_turn_on};
use commands::turn_on::TurnOnOptions;

#[derive(Parser)]
#[command(
    name = "zync",
    version,
    about = "Settings Sync for Zed — same idea as VS Code Settings Sync"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Turn on Settings Sync (sign in + first sync)
    #[command(alias = "enable")]
    On {
        /// Also sync agent chat history
        #[arg(long)]
        threads: bool,
        /// Override store repo (default: <you>/zync-store)
        #[arg(long)]
        repo: Option<String>,
    },
    /// Turn off Settings Sync on this machine
    #[command(alias = "disable")]
    Off,
    /// Sync now
    Sync,
    /// Show whether sync is on and if anything is pending
    Status,
    /// Sign in with GitHub CLI
    Login,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => cmd_default(),
        Some(Commands::On { threads, repo }) => cmd_turn_on(TurnOnOptions { threads, repo }),
        Some(Commands::Off) => cmd_turn_off(),
        Some(Commands::Sync) => cmd_sync(),
        Some(Commands::Status) => cmd_status(),
        Some(Commands::Login) => cmd_login(),
    }
}
