# zync

**Settings Sync for Zed** — same mental model as VS Code:

1. Turn it on  
2. Sign in once  
3. Sync when you want (`zync sync`)

Uses your **GitHub CLI (`gh`) account** (the one you use for git). Zed’s own Sign In can’t power this yet (`read:user` only, not exposed to extensions).

## Install

```bash
cargo install --path .
```

Or build a release binary:

```bash
cargo build --release
# target/release/zync
```

## Commands

```bash
zync on          # Turn on Settings Sync
zync off         # Turn off on this machine
zync sync        # Sync now
zync status      # On/off + pending changes
zync login       # Sign in with gh
```

### First machine

```bash
cargo install --path .
zync on
# …edit settings in Zed…
zync sync
```

### Second machine

```bash
zync on      # same GitHub user → downloads your settings
zync sync
```

That’s it. No pasting tokens into `settings.json`.

## What syncs (defaults)

| Category | Default |
|---|---|
| Settings | on |
| Keybindings | on |
| Tasks / debug tasks | on |
| Extensions (`auto_install_extensions`) | on |
| Agent chat history | **off** (`zync on --threads` to enable later) |

## How it compares to VS Code

| VS Code | zync today |
|---|---|
| Account Sign In | `gh auth` / `zync login` |
| Turn On Settings Sync | `zync on` |
| Sync Now | `zync sync` |
| Always-on background | not yet (later with a Zed extension) |
| Conflict UI | backups (`.bak-zync-*`) + remote wins on conflict |

A Zed UI toggle (“Settings Sync: Turn On”) is the next step once this CLI path is solid — extensions can’t do it alone with the current API.

## Requirements

- Rust (for building) / a `zync` binary
- [GitHub CLI](https://cli.github.com)
- git
