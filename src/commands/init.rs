use crate::auth::{ensure_gh_auth, gh_create_private_repo, gh_repo_exists};
use crate::config::{load_local_config, save_local_config};
use crate::store::LocalConfig;
use crate::util::confirm_or_yes;
use anyhow::{bail, Result};

pub struct InitOptions {
    pub threads: bool,
    pub repo: Option<String>,
    pub yes: bool,
}

/// Setup only: auth, ensure store repo exists, write local config. Does not sync.
pub fn cmd_init(options: InitOptions) -> Result<()> {
    let login = ensure_gh_auth(options.yes)?;

    if let Some(mut cfg) = load_local_config()? {
        cfg.enabled = true;
        if options.threads {
            cfg.categories.threads = true;
        }
        if let Some(repo) = options.repo {
            cfg.remote.repo = repo;
        }
        save_local_config(&cfg)?;
        println!("Already initialized.");
        println!("Store: {}", cfg.remote.repo);
        println!("Sync when you want:  zync sync");
        return Ok(());
    }

    let repo = options
        .repo
        .unwrap_or_else(|| format!("{login}/zync-store"));

    if gh_repo_exists(&repo) {
        let prompt = format!("Use existing GitHub repo `{repo}` as the sync store?");
        if !confirm_or_yes(&prompt, options.yes)? {
            bail!("Aborted.");
        }
        println!("Using existing repo: {repo}");
    } else {
        let prompt = format!("Create private GitHub repo `{repo}`?");
        if !confirm_or_yes(&prompt, options.yes)? {
            bail!("Aborted.");
        }
        gh_create_private_repo(&repo, "zync — Zed Settings Sync")?;
        println!("Created private repo: {repo}");
    }

    let mut cfg = LocalConfig::create_default(repo.clone(), "main");
    if options.threads {
        cfg.categories.threads = true;
    }
    save_local_config(&cfg)?;

    println!("Initialized ({login}). Store: {repo}");
    println!("Nothing synced yet. When ready:");
    println!("  zync sync");
    println!("Remove local setup:");
    println!("  zync purge");
    Ok(())
}
