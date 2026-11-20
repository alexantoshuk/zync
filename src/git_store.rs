use crate::paths::zync_cache_dir;
use crate::store::LocalConfig;
use crate::util::{path_exists, read_text_if_exists, write_text};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

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

pub fn ensure_store_clone(cfg: &LocalConfig) -> Result<PathBuf> {
    let dir = store_clone_path(cfg);
    let git_dir = dir.join(".git");

    if path_exists(&git_dir) {
        run_git(&dir, &["fetch", "origin", &cfg.remote.branch])?;
        run_git(&dir, &["checkout", &cfg.remote.branch])?;
        run_git(
            &dir,
            &["pull", "--ff-only", "origin", &cfg.remote.branch],
        )?;
        return Ok(dir);
    }

    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let url = format!("https://github.com/{}.git", cfg.remote.repo);
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
    Ok(dir)
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
    run_git(dir, &["add", "-A"])?;
    let status = run_git(dir, &["status", "--porcelain"])?;
    if status.is_empty() {
        return Ok(());
    }
    run_git(dir, &["commit", "-m", message])?;
    run_git(dir, &["push", "-u", "origin", branch])?;
    Ok(())
}

pub fn read_store_file(dir: &Path, relative: &str) -> Result<Option<String>> {
    read_text_if_exists(&dir.join(relative))
}

pub fn write_store_file(dir: &Path, relative: &str, content: &str) -> Result<()> {
    write_text(&dir.join(relative), content)
}
