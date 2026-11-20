//! Agent thread sync (Phase 2) — contract only until sqlite+zstd lands.

use crate::paths::zed_threads_db_path;
use crate::store::STORE_VERSION;
use crate::util::path_exists;
use anyhow::{bail, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadsIndex {
    pub version: u32,
    pub updated_at: String,
    pub threads: Vec<ThreadsIndexEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadsIndexEntry {
    pub id: String,
    pub summary: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_paths: Option<Vec<String>>,
    pub file: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ThreadsExportResult {
    pub index: ThreadsIndex,
    pub threads: Vec<()>,
}

pub fn threads_db_available() -> bool {
    path_exists(&zed_threads_db_path())
}

pub fn threads_db_location() -> String {
    zed_threads_db_path().display().to_string()
}

pub fn export_threads() -> Result<ThreadsExportResult> {
    let db = zed_threads_db_path();
    if !path_exists(&db) {
        return Ok(ThreadsExportResult {
            index: ThreadsIndex {
                version: STORE_VERSION,
                updated_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                threads: vec![],
            },
            threads: vec![],
        });
    }

    bail!(
        "Thread export is not implemented yet (Phase 2). DB found at: {}\n\
         Enable after adding sqlite+zstd reader.",
        db.display()
    );
}
