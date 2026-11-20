use crate::sync::engine::{sync_now, SyncAction};
use anyhow::Result;

pub fn cmd_sync() -> Result<()> {
    let result = sync_now()?;
    match result.action {
        SyncAction::InSync => println!("In sync."),
        SyncAction::Uploaded => {
            let list = if result.uploaded.is_empty() {
                "ok".into()
            } else {
                result.uploaded.join(", ")
            };
            println!("Uploaded: {list}");
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
