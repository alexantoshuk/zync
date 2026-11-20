use crate::store::Platform;
use std::path::PathBuf;

pub fn detect_platform() -> Platform {
    match std::env::consts::OS {
        "windows" => Platform::Windows,
        "macos" => Platform::Macos,
        "linux" => Platform::Linux,
        _ => Platform::Unknown,
    }
}

fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Zed user config directory (settings, keymap, tasks).
pub fn zed_config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ZED_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    match detect_platform() {
        Platform::Windows => {
            let appdata = std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join("AppData").join("Roaming"));
            appdata.join("Zed")
        }
        Platform::Macos => home_dir().join(".config").join("zed"),
        Platform::Linux | Platform::Unknown => {
            let base = std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".config"));
            base.join("zed")
        }
    }
}

/// Zed data directory (extensions, threads.db, logs).
pub fn zed_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ZED_DATA_DIR") {
        return PathBuf::from(dir);
    }
    match detect_platform() {
        Platform::Windows => {
            let local = std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join("AppData").join("Local"));
            local.join("Zed")
        }
        Platform::Macos => home_dir()
            .join("Library")
            .join("Application Support")
            .join("Zed"),
        Platform::Linux | Platform::Unknown => {
            let base = std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".local").join("share"));
            base.join("zed")
        }
    }
}

pub fn zed_settings_path() -> PathBuf {
    zed_config_dir().join("settings.json")
}

pub fn zed_keymap_path() -> PathBuf {
    zed_config_dir().join("keymap.json")
}

pub fn zed_tasks_path() -> PathBuf {
    zed_config_dir().join("tasks.json")
}

pub fn zed_debug_path() -> PathBuf {
    zed_config_dir().join("debug.json")
}

pub fn zed_threads_db_path() -> PathBuf {
    zed_data_dir().join("threads").join("threads.db")
}

pub fn zync_config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ZYNC_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    match detect_platform() {
        Platform::Windows => {
            let appdata = std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join("AppData").join("Roaming"));
            appdata.join("zync")
        }
        _ => {
            let base = std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".config"));
            base.join("zync")
        }
    }
}

pub fn zync_local_config_path() -> PathBuf {
    zync_config_dir().join("config.json")
}

pub fn zync_cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ZYNC_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    match detect_platform() {
        Platform::Windows => {
            let local = std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join("AppData").join("Local"));
            local.join("zync")
        }
        Platform::Macos => home_dir().join("Library").join("Caches").join("zync"),
        Platform::Linux | Platform::Unknown => {
            let base = std::env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".cache"));
            base.join("zync")
        }
    }
}
