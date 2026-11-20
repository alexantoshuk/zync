use anyhow::{bail, Context, Result};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

#[derive(Debug, thiserror::Error)]
pub enum GhError {
    #[error("{0}")]
    Message(String),
}

const GH_INSTALL_URL: &str = "https://cli.github.com";

/// Absolute path to `gh` when PATH is stale after a fresh install.
static GH_BIN: OnceLock<PathBuf> = OnceLock::new();

fn gh_program() -> PathBuf {
    GH_BIN
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("gh"))
}

fn is_gh_missing(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::NotFound
}

fn gh_available() -> bool {
    match Command::new(gh_program()).arg("--version").output() {
        Err(e) if is_gh_missing(&e) => false,
        Ok(_) | Err(_) => true,
    }
}

fn remember_gh_if_found(dir: &Path) {
    let candidate = if cfg!(windows) {
        dir.join("gh.exe")
    } else {
        dir.join("gh")
    };
    if candidate.is_file() {
        let _ = GH_BIN.set(candidate);
    }
}

/// `gh` may be installed but missing from this process PATH (common after winget).
fn locate_fresh_gh() {
    if gh_available() {
        return;
    }
    #[cfg(windows)]
    {
        for dir in [
            r"C:\Program Files\GitHub CLI",
            r"C:\Program Files (x86)\GitHub CLI",
        ] {
            remember_gh_if_found(Path::new(dir));
            if GH_BIN.get().is_some() {
                return;
            }
        }
        // Machine + User Path from the registry (current process env is often stale).
        if let Ok(output) = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "[Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')",
            ])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                for dir in path.split(';').map(str::trim).filter(|s| !s.is_empty()) {
                    remember_gh_if_found(Path::new(dir));
                    if GH_BIN.get().is_some() {
                        return;
                    }
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        for dir in ["/opt/homebrew/bin", "/usr/local/bin"] {
            remember_gh_if_found(Path::new(dir));
            if GH_BIN.get().is_some() {
                return;
            }
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for dir in ["/usr/bin", "/usr/local/bin"] {
            remember_gh_if_found(Path::new(dir));
            if GH_BIN.get().is_some() {
                return;
            }
        }
    }
}

fn install_cmd() -> Option<(&'static str, Vec<&'static str>)> {
    #[cfg(windows)]
    {
        return Some((
            "winget",
            vec![
                "install",
                "--id",
                "GitHub.cli",
                "-e",
                "--accept-source-agreements",
                "--accept-package-agreements",
            ],
        ));
    }
    #[cfg(target_os = "macos")]
    {
        return Some(("brew", vec!["install", "gh"]));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if Command::new("apt-get").arg("--version").output().is_ok() {
            return Some(("sudo", vec!["apt-get", "install", "-y", "gh"]));
        }
        return None;
    }
    #[allow(unreachable_code)]
    None
}

/// If `gh` is missing, offer to install it (winget / brew / apt), then continue.
pub fn ensure_gh_present(yes: bool) -> Result<()> {
    if gh_available() {
        return Ok(());
    }
    // Already installed but PATH not refreshed in this terminal.
    locate_fresh_gh();
    if gh_available() {
        return Ok(());
    }

    println!("GitHub CLI (gh) is not installed (needed for zync auth).");
    println!("Docs: {GH_INSTALL_URL}\n");

    match install_cmd() {
        Some((program, args)) => {
            let cmd_preview = format!("{program} {}", args.join(" "));
            let prompt = format!("Install GitHub CLI now via `{cmd_preview}`?");
            if !crate::util::confirm_or_yes(&prompt, yes)? {
                bail!("GitHub CLI (gh) is required. Install from {GH_INSTALL_URL}, then re-run.");
            }
            println!();
            crate::util::progress(&format!("running `{cmd_preview}`"));
            let status = Command::new(program)
                .args(&args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .with_context(|| format!("spawn `{cmd_preview}`"))?;
            if !status.success() {
                bail!("Install failed. Install manually from {GH_INSTALL_URL}, then re-run.");
            }
            locate_fresh_gh();
            if !gh_available() {
                bail!(
                    "GitHub CLI was installed, but `gh` is not on PATH in this terminal yet. Open a new terminal and re-run."
                );
            }
            println!();
            Ok(())
        }
        None => {
            bail!("GitHub CLI (gh) not found. Install from {GH_INSTALL_URL}, then re-run.");
        }
    }
}

fn run_gh(args: &[&str]) -> Result<String> {
    let output = Command::new(gh_program())
        .args(args)
        .output()
        .map_err(|e| {
            if is_gh_missing(&e) {
                GhError::Message(format!(
                    "GitHub CLI (gh) not found. Install from {GH_INSTALL_URL} and run `gh auth login`."
                ))
            } else {
                GhError::Message(format!("gh {}: {e}", args.join(" ")))
            }
        })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        bail!(GhError::Message(format!(
            "gh {} failed: {}",
            args.join(" "),
            if err.trim().is_empty() {
                out.trim()
            } else {
                err.trim()
            }
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Active GitHub login from `gh`.
pub fn gh_login() -> Result<String> {
    let login = run_gh(&["api", "user", "--jq", ".login"])?;
    if login.is_empty() {
        bail!(GhError::Message(
            "gh returned empty login; run `gh auth login`.".into()
        ));
    }
    Ok(login)
}

pub fn gh_repo_exists(repo: &str) -> bool {
    run_gh(&["repo", "view", repo, "--json", "name"]).is_ok()
}

pub fn gh_create_private_repo(repo: &str, description: &str) -> Result<()> {
    let mut parts = repo.split('/');
    let owner = parts.next().unwrap_or("");
    let name = parts.next().unwrap_or("");
    if owner.is_empty() || name.is_empty() || parts.next().is_some() {
        bail!(GhError::Message(format!("Invalid repo id: {repo}")));
    }

    let login = gh_login()?;
    if owner != login {
        bail!(GhError::Message(format!(
            "Can only auto-create repos under your login ({login}); got {owner}. Create {repo} manually, then re-run."
        )));
    }

    run_gh(&[
        "repo",
        "create",
        name,
        "--private",
        "--description",
        description,
        "--confirm",
    ])?;
    Ok(())
}

pub fn gh_delete_repo(repo: &str) -> Result<()> {
    run_gh(&["repo", "delete", repo, "--yes"])?;
    Ok(())
}

fn run_gh_auth_login() -> Result<String> {
    println!("Sign in to GitHub (same account you use for git).\n");
    let status = Command::new(gh_program())
        .args(["auth", "login"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            if is_gh_missing(&e) {
                anyhow::anyhow!(
                    "GitHub CLI (gh) not found. Install from {GH_INSTALL_URL} and re-run."
                )
            } else {
                anyhow::Error::new(e).context("spawn gh auth login")
            }
        })?;
    if !status.success() {
        bail!("GitHub sign-in was cancelled or failed.");
    }
    gh_login()
}

/// Ensure `gh` is logged in.
/// If a session already exists, ask to reuse it (unless `yes`).
pub fn ensure_gh_auth(yes: bool) -> Result<String> {
    ensure_gh_present(yes)?;
    match gh_login() {
        Ok(login) => {
            let prompt = format!("GitHub CLI is signed in as `{login}`. Continue with this account?");
            if !crate::util::confirm_or_yes(&prompt, yes)? {
                bail!("Aborted.");
            }
            Ok(login)
        }
        Err(_) => run_gh_auth_login(),
    }
}

/// Interactive login command: reuse existing session or run `gh auth login`.
pub fn interactive_login(yes: bool) -> Result<String> {
    ensure_gh_present(yes)?;
    match gh_login() {
        Ok(login) => {
            let prompt = format!("Already signed in as `{login}`. Use this account?");
            if crate::util::confirm_or_yes(&prompt, yes)? {
                return Ok(login);
            }
            run_gh_auth_login()
        }
        Err(_) => run_gh_auth_login(),
    }
}
