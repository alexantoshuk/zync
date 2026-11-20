use crate::sync::engine::{sync_now, SyncAction, SyncOptions};
use crate::util::confirm_or_yes;
use anyhow::{bail, Result};

pub struct CmdSyncOptions {
    pub refresh: bool,
    pub force_push: bool,
    pub yes: bool,
}

pub fn cmd_sync(options: CmdSyncOptions) -> Result<()> {
    if options.force_push {
        let prompt =
            "Overwrite cloud store with LOCAL settings (even if cloud is newer)? This cannot be undone easily.";
        if !confirm_or_yes(prompt, options.yes)? {
            bail!("Aborted.");
        }
    }

    let result = sync_now(SyncOptions {
        force_refresh: options.refresh,
        force_push: options.force_push,
    })?;
    match result.action {
        SyncAction::InSync => println!("In sync."),
        SyncAction::Uploaded => {
            let list = if result.uploaded.is_empty() {
                "ok".into()
            } else {
                result.uploaded.join(", ")
            };
            if options.force_push {
                println!("Force-pushed: {list}");
            } else {
                println!("Uploaded: {list}");
            }
        }
        SyncAction::Downloaded => {
            println!("Downloaded: {}", result.downloaded.join(", "));
            println!("Reload Zed if it was open.");
        }
        SyncAction::Merged => {
            let down = if result.downloaded.is_empty() {
                "—".into()
            } else {
                result.downloaded.join(", ")
            };
            let up = if result.uploaded.is_empty() {
                "—".into()
            } else {
                result.uploaded.join(", ")
            };
            println!("Synced (↓ {down} / ↑ {up}).");
            println!("Reload Zed if it was open.");
        }
    }
    Ok(())
}
