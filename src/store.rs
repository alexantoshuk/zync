use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const STORE_VERSION: u32 = 1;

pub mod paths {
    pub const MANIFEST: &str = "manifest.json";
    pub const SETTINGS: &str = "config/settings.json";
    pub const KEYMAP: &str = "config/keymap.json";
    pub const TASKS: &str = "config/tasks.json";
    pub const DEBUG: &str = "config/debug.json";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Macos,
    Linux,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestUpdatedBy {
    pub device_id: String,
    pub hostname: String,
    pub platform: Platform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestContents {
    #[serde(default = "default_true")]
    pub config: bool,
    #[serde(default)]
    pub threads: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestHashes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keymap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<String>,
    #[serde(rename = "threadsIndex", skip_serializing_if = "Option::is_none")]
    pub threads_index: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: u32,
    pub updated_at: String,
    pub updated_by: ManifestUpdatedBy,
    pub contents: ManifestContents,
    #[serde(default)]
    pub hashes: ManifestHashes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCategories {
    #[serde(default = "default_true")]
    pub settings: bool,
    #[serde(default = "default_true")]
    pub keybindings: bool,
    #[serde(default = "default_true")]
    pub tasks: bool,
    #[serde(default = "default_true")]
    pub extensions: bool,
    #[serde(default)]
    pub threads: bool,
}

impl Default for SyncCategories {
    fn default() -> Self {
        Self {
            settings: true,
            keybindings: true,
            tasks: true,
            extensions: true,
            threads: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub repo: String,
    #[serde(default = "default_branch")]
    pub branch: String,
}

fn default_branch() -> String {
    "main".into()
}

/// Deprecated fields kept for older local configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacySync {
    #[serde(default = "default_true")]
    pub config: bool,
    #[serde(default)]
    pub threads: bool,
    #[serde(default = "default_true", rename = "stripMachinePaths")]
    pub strip_machine_paths: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalConfig {
    pub version: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub remote: RemoteConfig,
    pub device_id: Uuid,
    #[serde(default)]
    pub categories: SyncCategories,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<LegacySync>,
}

impl LocalConfig {
    pub fn create_default(repo: String, branch: &str) -> Self {
        Self {
            version: STORE_VERSION,
            enabled: true,
            remote: RemoteConfig {
                repo,
                branch: branch.to_string(),
            },
            device_id: Uuid::new_v4(),
            categories: SyncCategories::default(),
            last_synced_at: None,
            sync: None,
        }
    }

    pub fn wants_config_sync(&self) -> bool {
        let c = &self.categories;
        c.settings || c.keybindings || c.tasks || c.extensions
    }

    pub fn wants_threads_sync(&self) -> bool {
        self.categories.threads || self.sync.as_ref().is_some_and(|s| s.threads)
    }
}
