use crate::config::{require_enabled_config, save_local_config};
use crate::sync::config::{diff_config, pull_config, push_config, remote_has_synced_data, DiffStatus};
use crate::sync::threads::{export_threads, threads_db_available};
use crate::util::progress;
use anyhow::Result;
use chrono::Utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAction {
    Uploaded,
    Downloaded,
    InSync,
    Merged,
}

#[derive(Debug)]
pub struct SyncResult {
    pub action: SyncAction,
    pub uploaded: Vec<String>,
    pub downloaded: Vec<String>,
}

/// One-shot sync like VS Code "Settings Sync: Sync Now".
pub fn sync_now() -> Result<SyncResult> {
    progress("loading local config");
    let mut cfg = require_enabled_config()?;
    let mut uploaded = Vec::new();
    let mut downloaded = Vec::new();

    if cfg.wants_config_sync() {
        progress("checking remote store");
        let has_remote = remote_has_synced_data(&cfg)?;
        let diffs = if has_remote {
            progress("comparing local vs remote");
            diff_config(&cfg)?
        } else {
            progress("remote store empty or missing synced data");
            Vec::new()
        };

        let any_different = diffs.iter().any(|d| {
            matches!(
                d.status,
                DiffStatus::LocalNewerOrDifferent | DiffStatus::LocalOnly | DiffStatus::RemoteOnly
            )
        });
        let all_same = has_remote
            && !diffs.is_empty()
            && diffs
                .iter()
                .all(|d| matches!(d.status, DiffStatus::Same | DiffStatus::Missing));

        if !has_remote {
            progress("uploading local config (first push)");
            uploaded.extend(push_config(&cfg)?);
        } else if all_same && !any_different {
            progress("already in sync");
        } else {
            let remote_only_or_different = diffs.iter().any(|d| {
                matches!(
                    d.status,
                    DiffStatus::RemoteOnly | DiffStatus::LocalNewerOrDifferent
                )
            });
            let local_only = diffs.iter().any(|d| d.status == DiffStatus::LocalOnly);

            if remote_only_or_different {
                progress("pulling remote config");
                downloaded.extend(pull_config(&cfg)?);
            }
            if local_only || remote_only_or_different {
                progress("pushing local config");
                uploaded.extend(push_config(&cfg)?);
            }
        }
    }

    if cfg.wants_threads_sync() && threads_db_available() {
        progress("exporting threads (Phase 2)");
        if let Err(err) = export_threads() {
            eprintln!("{err:#}");
        }
    }

    progress("saving lastSyncedAt");
    cfg.last_synced_at = Some(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
    save_local_config(&cfg)?;

    let action = if !uploaded.is_empty() && !downloaded.is_empty() {
        SyncAction::Merged
    } else if !uploaded.is_empty() {
        SyncAction::Uploaded
    } else if !downloaded.is_empty() {
        SyncAction::Downloaded
    } else {
        SyncAction::InSync
    };

    Ok(SyncResult {
        action,
        uploaded,
        downloaded,
    })
}
