use crate::auth::{ensure_gh_auth, gh_login};
use crate::config::load_local_config;
use crate::sync::config::{diff_config, DiffStatus};
use crate::sync::threads::{threads_db_available, threads_db_location};
use anyhow::Result;

pub fn cmd_status() -> Result<()> {
    let login = gh_login().ok();
    let cfg = load_local_config()?;

    let Some(cfg) = cfg.filter(|c| c.enabled) else {
        println!("Settings Sync: Off");
        if let Some(login) = login {
            println!("Signed in as: {login}");
        } else {
            println!("Signed in as: (not signed in)");
        }
        println!("\nTurn on with:  zync on");
        return Ok(());
    };

    println!("Settings Sync: On");
    println!(
        "Account: {}",
        login.as_deref().unwrap_or("(gh unavailable)")
    );
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

    match diff_config(&cfg) {
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

pub fn cmd_default() -> Result<()> {
    let cfg = load_local_config()?;
    if cfg.as_ref().is_none_or(|c| !c.enabled) {
        println!("Zed Settings Sync is off.");
        println!("Run:  zync on");
        return Ok(());
    }
    cmd_status()
}

pub fn cmd_login() -> Result<()> {
    let login = ensure_gh_auth()?;
    println!("Signed in as {login}.");
    println!("Turn on sync:  zync on");
    Ok(())
}
