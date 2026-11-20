# zync

**Settings Sync for Zed** — same idea as VS Code, as a standalone Rust CLI:

1. `zync on` — sign in with `gh`, create/find a private store repo  
2. Edit settings in Zed as usual  
3. `zync sync` when you want to push/pull  

Auth uses your **GitHub CLI (`gh`)** account (the one you use for git). Zed Sign In is not enough for this yet.

Sync is **on demand** (no background `watch`). Comparison is by **content hash** (SHA-256), not by date. On conflict, **remote wins**; the previous local file is kept as `.bak-zync-*`.

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
zync              # status if on, else hint to run `zync on`
zync on           # turn on (sign in + first sync)
zync on --threads # also enable agent chat history (Phase 2, not implemented yet)
zync on --repo owner/name   # override store repo (default: <you>/zync-store)
zync off          # turn off on this machine (cloud data kept)
zync sync         # sync now
zync status       # on/off + pending changes
zync login        # sign in with gh
```

Aliases: `on`/`enable`, `off`/`disable`.

### First machine

```bash
cargo install --path .
zync on
# …edit settings in Zed…
zync sync
```

### Second machine

```bash
zync on    # same GitHub user → pulls existing store
zync sync
```

Reload Zed after a download if it was open.

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
| Turn On Settings Sync | `zync on` |
| Sync Now | `zync sync` |
| Always-on background | not yet (possible later via a Zed extension) |
| Conflict UI | `.bak-zync-*` backups + remote wins |

A Zed UI toggle is a later step: a thin extension would call this binary. Extensions alone can’t do full settings sync with the current API.

## Requirements

- [Rust](https://rustup.rs) (to build) or a prebuilt `zync` binary  
- [GitHub CLI](https://cli.github.com) (`gh auth login`)  
- `git`

## Agent / contributor notes

- `AGENTS.md` — plan-before-code, keep docs in sync, `.ai/` layout  
- `.ai/state.md` — current focus / next (not a changelog)  
- `.ai/skills/` — optional topic skills (e.g. Zed API), added when needed  
