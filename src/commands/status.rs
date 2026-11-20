use crate::auth::{gh_login, interactive_login};
use crate::config::load_local_config;
use crate::git_store::prepare_store_dir;
use crate::sync::config::{diff_config, DiffStatus};
use crate::sync::threads::{threads_db_available, threads_db_location};
use crate::util::progress;
use anyhow::Result;

pub struct LoginOptions {
    pub yes: bool,
}

pub fn cmd_status(force_refresh: bool) -> Result<()> {
    progress("checking GitHub CLI login");
    let login = gh_login().ok();
    progress("loading local config");
    let cfg = load_local_config()?;

    let Some(cfg) = cfg.filter(|c| c.enabled) else {
        println!("Initialized: no");
        if let Some(login) = login {
            println!("Signed in as: {login}");
        } else {
            println!("Signed in as: (not signed in)");
        }
        println!("\nSet up with:  zync init");
        return Ok(());
    };

    println!("Initialized: yes");
    println!(
        "Account: {}",
        login.as_deref().unwrap_or("(gh unavailable)")
    );
    println!("Store: {}", cfg.remote.repo);
    if let Some(at) = &cfg.last_synced_at {
        println!("Last synced: {at}");
    }

    let cats = &cfg.categories;
    let syncing: Vec<&str> = [
        cats.settings.then_some("settings"),
        cats.keybindings.then_some("keybindings"),
        cats.tasks.then_some("tasks"),
        cats.extensions.then_some("extensions"),
        cats.threads.then_some("threads"),
    ]
    .into_iter()
    .flatten()
    .collect();
    println!("Syncing: {}", syncing.join(", "));

    match (|| -> anyhow::Result<_> {
        let dir = prepare_store_dir(&cfg, force_refresh)?;
        progress("comparing with remote");
        Ok(diff_config(&cfg, &dir)?)
    })() {
        Ok(diffs) => {
            let drifted: Vec<_> = diffs
                .iter()
                .filter(|d| !matches!(d.status, DiffStatus::Same | DiffStatus::Missing))
                .collect();
            if drifted.is_empty() {
                println!("Status: in sync");
            } else {
                println!("Status: pending changes");
                for d in drifted {
                    println!("  {}: {}", d.file, d.status.as_str());
                }
                println!("Run:  zync sync");
            }
        }
        Err(_) => println!("Status: waiting for first sync (or offline)"),
    }

    if cats.threads {
        println!(
            "Threads DB: {} ({})",
            threads_db_location(),
            if threads_db_available() {
                "found"
            } else {
                "missing"
            }
        );
    }

    Ok(())
}

pub fn cmd_login(options: LoginOptions) -> Result<()> {
    let login = interactive_login(options.yes)?;
    println!("Signed in as {login}.");
    println!("Set up sync:  zync init");
    Ok(())
}
