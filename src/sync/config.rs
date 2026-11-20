use crate::git_store::{
    commit_and_push, ensure_store_clone, init_empty_store_repo, read_store_file, write_store_file,
};
use crate::paths::{
    detect_platform, zed_debug_path, zed_keymap_path, zed_settings_path, zed_tasks_path,
};
use crate::store::{
    paths as store_paths, LocalConfig, Manifest, ManifestContents, ManifestHashes,
    ManifestUpdatedBy, STORE_VERSION,
};
use crate::sync::extensions::merge_extensions_into_settings;
use crate::util::{normalize_json_or_text, path_exists, read_text_if_exists, sha256_text, write_text};
use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;

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

fn ensure_dir(cfg: &LocalConfig) -> Result<std::path::PathBuf> {
    match ensure_store_clone(cfg) {
        Ok(dir) => Ok(dir),
        Err(_) => init_empty_store_repo(
            cfg,
            "# zync store\n\nPrivate Settings Sync store for Zed.\n",
        ),
    }
}

pub fn remote_has_synced_data(cfg: &LocalConfig) -> Result<bool> {
    match ensure_store_clone(cfg) {
        Ok(dir) => {
            let Some(raw) = read_store_file(&dir, store_paths::MANIFEST)? else {
                return Ok(false);
            };
            let manifest: Manifest = serde_json::from_str(&raw)?;
            Ok(get_hash(&manifest.hashes, ConfigKey::Settings).is_some()
                || get_hash(&manifest.hashes, ConfigKey::Keymap).is_some()
                || get_hash(&manifest.hashes, ConfigKey::Tasks).is_some()
                || get_hash(&manifest.hashes, ConfigKey::Debug).is_some()
                || manifest.hashes.threads_index.is_some())
        }
        Err(_) => Ok(false),
    }
}

pub fn push_config(cfg: &LocalConfig) -> Result<Vec<String>> {
    if cfg.categories.extensions {
        let _ = merge_extensions_into_settings()?;
    }

    let dir = ensure_dir(cfg)?;
    let mut pushed = Vec::new();
    let mut hashes = ManifestHashes::default();

    for file in &CONFIG_FILES {
        if !category_enabled(cfg, file.category) {
            continue;
        }
        let Some(text) = read_text_if_exists(&(file.local)())? else {
            continue;
        };
        let normalized = normalize_json_or_text(&text);
        write_store_file(&dir, file.remote, &normalized)?;
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
    write_store_file(&dir, store_paths::MANIFEST, &manifest_json)?;

    let msg = format!(
        "zync: sync from {}",
        hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".into())
    );
    commit_and_push(&dir, &msg, &cfg.remote.branch)?;

    Ok(pushed)
}

pub fn pull_config(cfg: &LocalConfig) -> Result<Vec<String>> {
    let dir = ensure_store_clone(cfg)?;
    let mut pulled = Vec::new();

    for file in &CONFIG_FILES {
        if !category_enabled(cfg, file.category) {
            continue;
        }
        let Some(remote) = read_store_file(&dir, file.remote)? else {
            continue;
        };

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

pub fn diff_config(cfg: &LocalConfig) -> Result<Vec<ConfigDiff>> {
    let remote_hashes = match ensure_store_clone(cfg) {
        Ok(dir) => match read_store_file(&dir, store_paths::MANIFEST)? {
            Some(raw) => {
                let manifest: Manifest = serde_json::from_str(&raw)?;
                manifest.hashes
            }
            None => ManifestHashes::default(),
        },
        Err(_) => ManifestHashes::default(),
    };

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
