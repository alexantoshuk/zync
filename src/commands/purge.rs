use crate::auth::{gh_delete_repo, gh_repo_exists};
use crate::config::load_local_config;
use crate::git_store::store_clone_path;
use crate::paths::{zync_cache_dir, zync_local_config_path};
use crate::util::{confirm_or_yes, path_exists};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub struct PurgeOptions {
    /// Also delete the GitHub store repository.
    pub remote: bool,
    /// Skip interactive confirmation.
    pub yes: bool,
}

/// Undo local `init` (config + store cache). With `--remote`, delete the cloud repo too.
pub fn cmd_purge(options: PurgeOptions) -> Result<()> {
    let cfg = load_local_config()?;
    let repo = cfg.as_ref().map(|c| c.remote.repo.clone());
    let config_path = zync_local_config_path();
    let cache_dir: PathBuf = cfg
        .as_ref()
        .map(store_clone_path)
        .unwrap_or_else(|| zync_cache_dir().join("stores"));

    if options.remote && repo.is_none() {
        bail!(
            "No local config with a store repo. Cannot purge remote; run after `zync init`, or omit `--remote`."
        );
    }

    println!("This will:");
    if path_exists(&config_path) {
        println!("  - delete local config  {}", config_path.display());
    } else {
        println!("  - local config         (none at {})", config_path.display());
    }
    if path_exists(&cache_dir) {
        println!("  - delete store cache   {}", cache_dir.display());
    } else {
        println!("  - store cache          (none at {})", cache_dir.display());
    }
    if options.remote {
        let repo = repo.as_deref().unwrap();
        println!("  - DELETE GitHub repo   {repo}  (permanent)");
    } else {
        println!("  - cloud store          kept (use --remote to delete)");
    }
    println!("Zed settings/keymap files on disk are not touched.");

    if !confirm_or_yes("Proceed?", options.yes)? {
        println!("Aborted.");
        return Ok(());
    }

    if options.remote {
        let repo = repo.as_deref().unwrap();
        if gh_repo_exists(repo) {
            gh_delete_repo(repo)?;
            println!("Deleted remote: {repo}");
        } else {
            println!("Remote repo not found (already gone?): {repo}");
        }
    }

    if path_exists(&config_path) {
        std::fs::remove_file(&config_path)?;
        println!("Removed {}", config_path.display());
    }

    remove_path_if_exists(&cache_dir)?;

    println!("Local zync setup purged.");
    if !options.remote {
        println!("Cloud store kept. Use `zync purge --remote` to delete it too.");
    }
    println!("Set up again with:  zync init");
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> Result<()> {
    if !path_exists(path) {
        return Ok(());
    }
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    println!("Removed {}", path.display());
    Ok(())
}
