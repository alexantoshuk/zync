mod auth;
mod commands;
mod config;
mod git_store;
mod paths;
mod store;
mod sync;
mod util;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use commands::init::InitOptions;
use commands::purge::PurgeOptions;
use commands::status::LoginOptions;
use commands::{cmd_init, cmd_login, cmd_purge, cmd_status, cmd_sync};

#[derive(Parser)]
#[command(
    name = "zync",
    version,
    about = "Settings Sync for Zed — sync on demand via GitHub"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize sync (sign in + store repo + local config; does not sync)
    Init {
        /// Also enable agent chat history category (export not implemented yet)
        #[arg(long)]
        threads: bool,
        /// Override store repo (default: <you>/zync-store)
        #[arg(long)]
        repo: Option<String>,
        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Remove local zync setup (config + store cache; confirms first)
    Purge {
        /// Also delete the GitHub store repository
        #[arg(long)]
        remote: bool,
        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Sync now
    Sync,
    /// Show whether sync is initialized and if anything is pending
    Status,
    /// Sign in with GitHub CLI
    Login {
        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
    },
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
        None => {
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
        Some(Commands::Init {
            threads,
            repo,
            yes,
        }) => cmd_init(InitOptions {
            threads,
            repo,
            yes,
        }),
        Some(Commands::Purge { remote, yes }) => cmd_purge(PurgeOptions { remote, yes }),
        Some(Commands::Sync) => cmd_sync(),
        Some(Commands::Status) => cmd_status(),
        Some(Commands::Login { yes }) => cmd_login(LoginOptions { yes }),
    }
}
