use crate::auth::{ensure_gh_auth, gh_create_private_repo, gh_repo_exists};
use crate::config::{load_local_config, save_local_config};
use crate::git_store::{commit_and_push, init_empty_store_repo, write_store_file};
use crate::paths::detect_platform;
use crate::store::{
    paths as store_paths, LocalConfig, Manifest, ManifestContents, ManifestHashes,
    ManifestUpdatedBy, STORE_VERSION,
};
use crate::sync::{remote_has_synced_data, sync_now};
use anyhow::Result;
use chrono::Utc;

pub struct TurnOnOptions {
    pub threads: bool,
    pub repo: Option<String>,
}

pub fn cmd_turn_on(options: TurnOnOptions) -> Result<()> {
    let login = ensure_gh_auth()?;
    let repo = options
        .repo
        .unwrap_or_else(|| format!("{login}/zync-store"));

    let cfg = match load_local_config()? {
        None => {
            if !gh_repo_exists(&repo) {
                gh_create_private_repo(&repo, "zync — Zed Settings Sync")?;
            }
            let mut cfg = LocalConfig::create_default(repo, "main");
            if options.threads {
                cfg.categories.threads = true;
            }

            let dir =
                init_empty_store_repo(&cfg, "# zync\n\nZed Settings Sync store.\n")?;
            let hostname = hostname::get()
                .map(|h| h.to_string_lossy().into_owned())
                .unwrap_or_else(|_| "unknown".into());
            let manifest = Manifest {
                version: STORE_VERSION,
                updated_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                updated_by: ManifestUpdatedBy {
                    device_id: cfg.device_id.to_string(),
                    hostname,
                    platform: detect_platform(),
                },
                contents: ManifestContents {
                    config: true,
                    threads: cfg.categories.threads,
                },
                hashes: ManifestHashes::default(),
            };
            let mut json = serde_json::to_string_pretty(&manifest)?;
            json.push('\n');
            write_store_file(&dir, store_paths::MANIFEST, &json)?;
            let _ = commit_and_push(&dir, "zync: turn on", &cfg.remote.branch);
            save_local_config(&cfg)?;
            cfg
        }
        Some(mut cfg) => {
            cfg.enabled = true;
            if options.threads {
                cfg.categories.threads = true;
            }
            save_local_config(&cfg)?;
            cfg
        }
    };

    let had_remote = remote_has_synced_data(&cfg)?;
    let result = sync_now()?;

    println!("Settings Sync is on ({login}).");
    if !had_remote && !result.uploaded.is_empty() {
        println!("Uploaded: {}", result.uploaded.join(", "));
    } else if !result.downloaded.is_empty() {
        println!("Downloaded: {}", result.downloaded.join(", "));
        println!("Reload Zed if it was open.");
    } else if matches!(result.action, crate::sync::engine::SyncAction::InSync) {
        println!("Already in sync.");
    } else if !result.uploaded.is_empty() {
        println!("Synced: {}", result.uploaded.join(", "));
    }

    println!("\nSync once:");
    println!("  zync sync");
    println!("Turn off:");
    println!("  zync off");
    Ok(())
}
