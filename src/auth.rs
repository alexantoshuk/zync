use anyhow::{bail, Context, Result};
use std::io::Write;
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

/// Ensure `gh` is logged in; if not, run interactive `gh auth login`.
pub fn ensure_gh_auth() -> Result<String> {
    match gh_login() {
        Ok(login) => Ok(login),
        Err(_) => {
            println!("Sign in to GitHub to turn on sync (same account you use for git).\n");
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
    }
}

#[allow(dead_code)]
pub fn gh_api_with_body(endpoint: &str, method: Option<&str>, body: &str) -> Result<String> {
    let mut args = vec!["api", endpoint];
    if let Some(m) = method {
        args.push("--method");
        args.push(m);
    }
    args.push("--input");
    args.push("-");

    let mut child = Command::new("gh")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn gh api")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(body.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!(GhError::Message(format!(
            "gh {} failed: {}",
            args.join(" "),
            err.trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
