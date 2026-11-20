use crate::paths::{zed_data_dir, zed_settings_path};
use crate::util::{path_exists, read_text_if_exists, write_text};
use anyhow::Result;
use serde_json::{Map, Value};

fn installed_extensions_dir() -> std::path::PathBuf {
    zed_data_dir().join("extensions").join("installed")
}

pub fn list_installed_extension_ids() -> Result<Vec<String>> {
    let dir = installed_extensions_dir();
    if !path_exists(&dir) {
        return Ok(Vec::new());
    }
    let mut ids: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    ids.sort();
    Ok(ids)
}

/// Merge installed extension IDs into settings.auto_install_extensions.
pub fn merge_extensions_into_settings() -> Result<bool> {
    let ids = list_installed_extension_ids()?;
    if ids.is_empty() {
        return Ok(false);
    }

    let settings_path = zed_settings_path();
    let raw = read_text_if_exists(&settings_path)?.unwrap_or_else(|| "{}".into());
    let mut settings: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Ok(false),
    };

    let obj = match settings.as_object_mut() {
        Some(o) => o,
        None => return Ok(false),
    };

    let mut existing: Map<String, Value> = match obj.get("auto_install_extensions") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };

    let mut changed = false;
    for id in ids {
        if existing.get(&id) != Some(&Value::Bool(true)) {
            existing.insert(id, Value::Bool(true));
            changed = true;
        }
    }

    if !changed {
        return Ok(false);
    }

    obj.insert("auto_install_extensions".into(), Value::Object(existing));
    let mut text = serde_json::to_string_pretty(&settings)?;
    text.push('\n');
    write_text(&settings_path, &text)?;
    Ok(true)
}
