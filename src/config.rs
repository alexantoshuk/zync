use crate::paths::zync_local_config_path;
use crate::store::LocalConfig;
use crate::util::{read_text_if_exists, write_text};
use anyhow::{bail, Result};

pub fn load_local_config() -> Result<Option<LocalConfig>> {
    let Some(raw) = read_text_if_exists(&zync_local_config_path())? else {
        return Ok(None);
    };
    let cfg: LocalConfig = serde_json::from_str(&raw)?;
    Ok(Some(cfg))
}

pub fn require_local_config() -> Result<LocalConfig> {
    load_local_config()?.ok_or_else(|| {
        anyhow::anyhow!("Not initialized. Run `zync init` first.")
    })
}

pub fn require_enabled_config() -> Result<LocalConfig> {
    let cfg = require_local_config()?;
    if !cfg.enabled {
        bail!("Local config is disabled. Run `zync init` or `zync purge` then `zync init`.");
    }
    Ok(cfg)
}

pub fn save_local_config(cfg: &LocalConfig) -> Result<()> {
    let mut text = serde_json::to_string_pretty(cfg)?;
    text.push('\n');
    write_text(&zync_local_config_path(), &text)
}
