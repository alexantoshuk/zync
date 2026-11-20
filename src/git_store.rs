use crate::paths::zync_cache_dir;
use crate::store::LocalConfig;
use crate::util::{path_exists, progress, read_text_if_exists, write_text};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Skip network fetch if the store was refreshed this recently (shared by sync + status).
pub const STORE_REFRESH_TTL_SECS: u64 = 120;

fn run_git(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("git {}", args.join(" ")))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        bail!(
            "git {} failed: {}",
            args.join(" "),
            if err.trim().is_empty() {
                out.trim()
            } else {
                err.trim()
            }
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn store_clone_path(cfg: &LocalConfig) -> PathBuf {
    let safe = cfg.remote.repo.replace('/', "_");
    zync_cache_dir().join("stores").join(safe)
}

fn refresh_stamp_path(dir: &Path) -> PathBuf {
    // Inside .git so `git add -A` never commits it.
    dir.join(".git").join("zync-last-refresh")
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn read_last_refresh_secs(dir: &Path) -> Option<u64> {
    let raw = read_text_if_exists(&refresh_stamp_path(dir)).ok()??;
    raw.trim().parse().ok()
}

fn write_last_refresh(dir: &Path) -> Result<()> {
    let stamp = refresh_stamp_path(dir);
    write_text(&stamp, &format!("{}\n", now_unix_secs()))
}

fn secs_since_refresh(dir: &Path) -> Option<u64> {
    let last = read_last_refresh_secs(dir)?;
    let now = now_unix_secs();
    Some(now.saturating_sub(last))
}

fn ensure_branch_checked_out(dir: &Path, branch: &str) -> Result<()> {
    let upstream = format!("origin/{branch}");
    progress(&format!("git checkout {branch}"));
    run_git(dir, &["checkout", branch])?;
    // Local only — aligns HEAD with last fetched origin tip without network.
    progress(&format!("git reset --hard {upstream}"));
    let _ = run_git(dir, &["reset", "--hard", &upstream]);
    Ok(())
}

fn fetch_and_reset(dir: &Path, cfg: &LocalConfig) -> Result<()> {
    progress(&format!(
        "git fetch origin/{} ({})",
        cfg.remote.branch, cfg.remote.repo
    ));
    run_git(dir, &["fetch", "origin", &cfg.remote.branch])?;
    ensure_branch_checked_out(dir, &cfg.remote.branch)?;
    write_last_refresh(dir)?;
    Ok(())
}

/// Clone or refresh the store. Skips fetch when a recent refresh stamp exists (unless `force`).
pub fn refresh_store_clone(cfg: &LocalConfig, force: bool) -> Result<PathBuf> {
    let dir = store_clone_path(cfg);
    let git_dir = dir.join(".git");

    if path_exists(&git_dir) {
        if !force {
            if let Some(age) = secs_since_refresh(&dir) {
                if age < STORE_REFRESH_TTL_SECS {
                    progress(&format!(
                        "store fresh (refreshed {age}s ago, ttl {}s); skipping fetch",
                        STORE_REFRESH_TTL_SECS
                    ));
                    // Still point HEAD at last known origin tip (local).
                    let upstream = format!("origin/{}", cfg.remote.branch);
                    let _ = run_git(&dir, &["checkout", &cfg.remote.branch]);
                    let _ = run_git(&dir, &["reset", "--hard", &upstream]);
                    return Ok(dir);
                }
            }
        }

        fetch_and_reset(&dir, cfg)?;
        return Ok(dir);
    }

    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let url = format!("https://github.com/{}.git", cfg.remote.repo);
    progress(&format!("cloning store {url}"));
    let output = Command::new("git")
        .args([
            "clone",
            "--branch",
            &cfg.remote.branch,
            "--single-branch",
            &url,
            dir.to_str().unwrap_or_default(),
        ])
        .output()
        .context("git clone")?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("git clone failed: {}", err.trim());
    }
    write_last_refresh(&dir)?;
    Ok(dir)
}

/// Refresh if possible; otherwise create an empty local git repo pointed at origin.
pub fn prepare_store_dir(cfg: &LocalConfig, force: bool) -> Result<PathBuf> {
    match refresh_store_clone(cfg, force) {
        Ok(dir) => Ok(dir),
        Err(err) => {
            progress(&format!(
                "remote clone unavailable ({err:#}); initializing empty local store"
            ));
            init_empty_store_repo(
                cfg,
                "# zync store\n\nPrivate Settings Sync store for Zed.\n",
            )
        }
    }
}

pub fn init_empty_store_repo(cfg: &LocalConfig, readme: &str) -> Result<PathBuf> {
    let dir = store_clone_path(cfg);
    std::fs::create_dir_all(&dir)?;

    if !path_exists(&dir.join(".git")) {
        run_git(&dir, &["init", "-b", &cfg.remote.branch])?;
        let url = format!("https://github.com/{}.git", cfg.remote.repo);
        run_git(&dir, &["remote", "add", "origin", &url])?;
    }

    write_text(&dir.join("README.md"), readme)?;
    write_text(&dir.join(".gitignore"), "# zync store\n.DS_Store\n")?;
    Ok(dir)
}

pub fn commit_and_push(dir: &Path, message: &str, branch: &str) -> Result<()> {
    progress("git add");
    run_git(dir, &["add", "-A"])?;
    let status = run_git(dir, &["status", "--porcelain"])?;
    if status.is_empty() {
        progress("nothing to commit");
        return Ok(());
    }
    progress(&format!("git commit ({message})"));
    run_git(dir, &["commit", "-m", message])?;
    progress(&format!("git push origin/{branch}"));
    run_git(dir, &["push", "-u", "origin", branch])?;
    // Local tip matches what we just pushed — treat as refreshed.
    write_last_refresh(dir)?;
    Ok(())
}

pub fn read_store_file(dir: &Path, relative: &str) -> Result<Option<String>> {
    read_text_if_exists(&dir.join(relative))
}

pub fn write_store_file(dir: &Path, relative: &str, content: &str) -> Result<()> {
    write_text(&dir.join(relative), content)
}
