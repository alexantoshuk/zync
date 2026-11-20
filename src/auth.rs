use anyhow::{bail, Context, Result};
use std::process::{Command, Stdio};

#[derive(Debug, thiserror::Error)]
pub enum GhError {
    #[error("{0}")]
    Message(String),
}

fn run_gh(args: &[&str]) -> Result<String> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                GhError::Message(
                    "GitHub CLI (gh) not found. Install https://cli.github.com and run `gh auth login`."
                        .into(),
                )
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
    let status = Command::new("gh")
        .args(["auth", "login"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("spawn gh auth login")?;
    if !status.success() {
        bail!("GitHub sign-in was cancelled or failed.");
    }
    gh_login()
}

/// Ensure `gh` is logged in.
/// If a session already exists, ask to reuse it (unless `yes`).
pub fn ensure_gh_auth(yes: bool) -> Result<String> {
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
