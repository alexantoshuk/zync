use crate::git_store::{commit_and_push, read_store_file, write_store_file};
use crate::paths::{
    detect_platform, zed_debug_path, zed_keymap_path, zed_settings_path, zed_tasks_path,
};
use crate::store::{
    paths as store_paths, LocalConfig, Manifest, ManifestContents, ManifestHashes,
    ManifestUpdatedBy, STORE_VERSION,
};
use crate::sync::extensions::merge_extensions_into_settings;
use crate::util::{
    normalize_json_or_text, path_exists, progress, read_text_if_exists, sha256_text, write_text,
};
use anyhow::Result;
use chrono::Utc;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKey {
    Settings,
    Keymap,
    Tasks,
    Debug,
}

impl ConfigKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Settings => "settings",
            Self::Keymap => "keymap",
            Self::Tasks => "tasks",
            Self::Debug => "debug",
        }
    }

    fn all() -> [Self; 4] {
        [Self::Settings, Self::Keymap, Self::Tasks, Self::Debug]
    }
}

#[derive(Debug, Clone, Copy)]
enum Category {
    Settings,
    Keybindings,
    Tasks,
}

struct ConfigFile {
    key: ConfigKey,
    local: fn() -> PathBuf,
    remote: &'static str,
    category: Category,
}

const CONFIG_FILES: [ConfigFile; 4] = [
    ConfigFile {
        key: ConfigKey::Settings,
        local: zed_settings_path,
        remote: store_paths::SETTINGS,
        category: Category::Settings,
    },
    ConfigFile {
        key: ConfigKey::Keymap,
        local: zed_keymap_path,
        remote: store_paths::KEYMAP,
        category: Category::Keybindings,
    },
    ConfigFile {
        key: ConfigKey::Tasks,
        local: zed_tasks_path,
        remote: store_paths::TASKS,
        category: Category::Tasks,
    },
    ConfigFile {
        key: ConfigKey::Debug,
        local: zed_debug_path,
        remote: store_paths::DEBUG,
        category: Category::Tasks,
    },
];

fn category_enabled(cfg: &LocalConfig, category: Category) -> bool {
    match category {
        Category::Settings => cfg.categories.settings,
        Category::Keybindings => cfg.categories.keybindings,
        Category::Tasks => cfg.categories.tasks,
    }
}

#[derive(Debug, Clone)]
pub struct ConfigDiff {
    pub file: String,
    #[allow(dead_code)]
    pub local_hash: Option<String>,
    #[allow(dead_code)]
    pub remote_hash: Option<String>,
    pub status: DiffStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStatus {
    Same,
    LocalNewerOrDifferent,
    RemoteOnly,
    LocalOnly,
    Missing,
}

impl DiffStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Same => "same",
            Self::LocalNewerOrDifferent => "local-newer-or-different",
            Self::RemoteOnly => "remote-only",
            Self::LocalOnly => "local-only",
            Self::Missing => "missing",
        }
    }
}

fn get_hash(hashes: &ManifestHashes, key: ConfigKey) -> Option<String> {
    match key {
        ConfigKey::Settings => hashes.settings.clone(),
        ConfigKey::Keymap => hashes.keymap.clone(),
        ConfigKey::Tasks => hashes.tasks.clone(),
        ConfigKey::Debug => hashes.debug.clone(),
    }
}

fn set_hash(hashes: &mut ManifestHashes, key: ConfigKey, value: String) {
    match key {
        ConfigKey::Settings => hashes.settings = Some(value),
        ConfigKey::Keymap => hashes.keymap = Some(value),
        ConfigKey::Tasks => hashes.tasks = Some(value),
        ConfigKey::Debug => hashes.debug = Some(value),
    }
}

pub fn collect_local_config_hashes(cfg: Option<&LocalConfig>) -> Result<ManifestHashes> {
    let mut hashes = ManifestHashes::default();
    for file in &CONFIG_FILES {
        if let Some(cfg) = cfg {
            if !category_enabled(cfg, file.category) {
                continue;
            }
        }
        if let Some(text) = read_text_if_exists(&(file.local)())? {
            let normalized = normalize_json_or_text(&text);
            set_hash(&mut hashes, file.key, sha256_text(&normalized));
        }
    }
    Ok(hashes)
}

fn read_remote_hashes(dir: &Path) -> Result<ManifestHashes> {
    let Some(raw) = read_store_file(dir, store_paths::MANIFEST)? else {
        return Ok(ManifestHashes::default());
    };
    let manifest: Manifest = serde_json::from_str(&raw)?;
    Ok(manifest.hashes)
}

pub fn remote_has_synced_data(dir: &Path) -> Result<bool> {
    let hashes = read_remote_hashes(dir)?;
    Ok(get_hash(&hashes, ConfigKey::Settings).is_some()
        || get_hash(&hashes, ConfigKey::Keymap).is_some()
        || get_hash(&hashes, ConfigKey::Tasks).is_some()
        || get_hash(&hashes, ConfigKey::Debug).is_some()
        || hashes.threads_index.is_some())
}

/// Push using an already-prepared store directory (no extra fetch).
pub fn push_config(cfg: &LocalConfig, dir: &Path) -> Result<Vec<String>> {
    if cfg.categories.extensions {
        progress("merging installed extensions into settings");
        let _ = merge_extensions_into_settings()?;
    }

    let mut pushed = Vec::new();
    let mut hashes = ManifestHashes::default();

    for file in &CONFIG_FILES {
        if !category_enabled(cfg, file.category) {
            continue;
        }
        let Some(text) = read_text_if_exists(&(file.local)())? else {
            continue;
        };
        progress(&format!("staging {}", file.key.as_str()));
        let normalized = normalize_json_or_text(&text);
        write_store_file(dir, file.remote, &normalized)?;
        set_hash(&mut hashes, file.key, sha256_text(&normalized));
        pushed.push(file.key.as_str().to_string());
    }

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
            threads: cfg.wants_threads_sync(),
        },
        hashes,
    };

    let mut manifest_json = serde_json::to_string_pretty(&manifest)?;
    manifest_json.push('\n');
    write_store_file(dir, store_paths::MANIFEST, &manifest_json)?;

    let msg = format!(
        "zync: sync from {}",
        hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".into())
    );
    commit_and_push(dir, &msg, &cfg.remote.branch)?;

    Ok(pushed)
}

/// Pull using an already-refreshed store directory (no extra fetch).
pub fn pull_config(cfg: &LocalConfig, dir: &Path) -> Result<Vec<String>> {
    let mut pulled = Vec::new();

    for file in &CONFIG_FILES {
        if !category_enabled(cfg, file.category) {
            continue;
        }
        let Some(remote) = read_store_file(dir, file.remote)? else {
            continue;
        };

        progress(&format!("applying {}", file.key.as_str()));
        let dest = (file.local)();
        if path_exists(&dest) {
            let stamp = Utc::now()
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
                .replace([':', '.'], "-");
            let backup = format!("{}.bak-zync-{stamp}", dest.display());
            let existing = read_text_if_exists(&dest)?.unwrap_or_default();
            write_text(std::path::Path::new(&backup), &existing)?;
        }
        write_text(&dest, &remote)?;
        pulled.push(file.key.as_str().to_string());
    }

    Ok(pulled)
}

/// Diff against an already-refreshed store directory (no extra fetch).
pub fn diff_config(cfg: &LocalConfig, dir: &Path) -> Result<Vec<ConfigDiff>> {
    progress("reading remote manifest");
    let remote_hashes = read_remote_hashes(dir)?;

    progress("hashing local config files");
    let local_hashes = collect_local_config_hashes(Some(cfg))?;

    Ok(ConfigKey::all()
        .into_iter()
        .map(|key| {
            let local_hash = get_hash(&local_hashes, key);
            let remote_hash = get_hash(&remote_hashes, key);
            let status = match (&local_hash, &remote_hash) {
                (None, None) => DiffStatus::Missing,
                (Some(l), Some(r)) if l == r => DiffStatus::Same,
                (Some(_), None) => DiffStatus::LocalOnly,
                (None, Some(_)) => DiffStatus::RemoteOnly,
                _ => DiffStatus::LocalNewerOrDifferent,
            };
            ConfigDiff {
                file: key.as_str().to_string(),
                local_hash,
                remote_hash,
                status,
            }
        })
        .collect())
}
