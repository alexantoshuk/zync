# zync

**Settings Sync for Zed** — same idea as VS Code, as a standalone Rust CLI:

1. `zync init` — sign in with `gh`, create/find a private store repo  
2. Edit settings in Zed as usual  
3. `zync sync` when you want to push/pull  

Auth uses your **GitHub CLI (`gh`)** account (the one you use for git). Zed Sign In is not enough for this yet. Reusing an existing `gh` session, creating/using the store repo, or deleting the remote all ask for confirmation (skip with `--yes`).

Sync is **on demand** (no background watcher). Comparison is by **content hash** (SHA-256), not by date. On conflict, **remote wins** by default (local backup `.bak-zync-*`); overwrite cloud with `zync sync --force-push`. Store refresh (`git fetch` + hard reset) is shared: within **120s** after a fetch/push, `sync`/`status` skip another fetch (use `--refresh` to force). Progress on stderr.

## Install

```bash
cargo install --path .
```

Or:

```bash
cargo build --release
# target/release/zync   (Windows: target/release/zync.exe)
```

## Commands

```bash
zync                    # print help
zync init               # set up only (sign in + store repo + local config; no sync)
zync init --threads     # also enable agent chat history category (Phase 2 NYI)
zync init --repo owner/name   # override store (default: <you>/zync-store)
zync init --yes         # skip confirmation prompts
zync sync               # sync now (push/pull settings)
zync sync --refresh     # force git fetch even if store was refreshed recently
zync sync --force-push  # overwrite cloud with local (even if cloud is newer; confirms)
zync sync --force-push --yes
zync status             # initialized? + pending changes
zync status --refresh   # force git fetch
zync login              # sign in with gh (confirms before reusing existing session)
zync login --yes        # skip confirmation prompts
zync purge              # remove local setup (prints plan + confirms); cloud kept
zync purge --remote     # local + delete GitHub store repo
zync purge --yes        # skip confirmation
zync purge --remote --yes
```

### First machine

```bash
cargo install --path .
zync init
# …edit settings in Zed…
zync sync
```

### Second machine

```bash
zync init    # same GitHub user → points at existing store
zync sync    # downloads settings
```

Reload Zed after a download if it was open.

### Tear down

```bash
zync purge              # this machine only
zync purge --remote     # also delete the cloud store
```

## What syncs (defaults)

| Category | Default | Notes |
|---|---|---|
| Settings | on | `settings.json` |
| Keybindings | on | `keymap.json` |
| Tasks / debug | on | `tasks.json`, `debug.json` |
| Extensions | on | merges installed IDs into `auto_install_extensions` |
| Agent chat history | **off** | store format ready; export/import not implemented yet |

Store layout lives in a private GitHub repo (default `<you>/zync-store`): `manifest.json` + `config/*`.

## VS Code → zync

| VS Code | zync |
|---|---|
| Account Sign In | `gh auth` / `zync login` |
| Turn On Settings Sync | `zync init` (setup only) + `zync sync` when you want |
| Sync Now | `zync sync` |
| Turn Off | `zync purge` (removes local setup; optional `--remote`) |
| Conflict UI | `.bak-zync-*` backups + remote wins |

A Zed UI toggle is a later step: a thin extension would call this binary. Extensions alone can’t do full settings sync with the current API.

## Requirements

- [Rust](https://rustup.rs) (to build) or a prebuilt `zync` binary  
- [GitHub CLI](https://cli.github.com) (`gh`) — `zync init` / `zync login` offer to install if missing (`winget` / `brew` / `apt`), then `gh auth login`  
- `git`

## Agent / contributor notes

- `AGENTS.md` — workflow: state first → plan → code; before commit: docs/dead-code check; `commit` = approval  
- `.ai/product.md` — product model and behavior  
- `.ai/state.md` — current focus / next (not a changelog)  
- `.ai/skills/` — optional topic skills (e.g. Zed API), added when needed  
