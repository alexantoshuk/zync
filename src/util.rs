use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn sha256_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex_encode(&hasher.finalize())
}

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn read_text_if_exists(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("read {}", path.display())),
    }
}

pub fn write_text(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("mkdir {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

pub fn path_exists(path: &Path) -> bool {
    path.exists()
}

/// Normalize JSON (pretty + LF) when possible; otherwise normalize newlines.
pub fn normalize_json_or_text(text: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(v) => {
            let mut out = serde_json::to_string_pretty(&v).unwrap_or_else(|_| text.to_string());
            out.push('\n');
            out
        }
        Err(_) => text.replace("\r\n", "\n"),
    }
}
