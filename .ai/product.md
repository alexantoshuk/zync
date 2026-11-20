# zync — product & behavior

## Product

**zync** is a standalone Rust CLI that syncs Zed editor settings across machines, in the spirit of VS Code Settings Sync.

- **Transport:** private GitHub repo (default `<gh-login>/zync-store`) via `git` + **GitHub CLI (`gh`)** auth  
- **Mode:** on demand only — no background watcher  
- **Compare:** SHA-256 of normalized file contents (not mtime)  
- **Conflicts:** remote wins by default; previous local file saved as `.bak-zync-*`. Override with `zync sync --force-push` (local overwrites cloud)  
- **Future:** thin Zed extension may shell out to this binary; full sync inside WASM extensions is not viable with current Zed APIs  

## Commands (behavior)

| Command | Behavior |
|---|---|
| `zync` | Print help and exit |
| `zync init` | Setup only: confirm/`gh` session, create or select store repo, write local zync config. **Does not sync** |
| `zync sync` | Refresh store (fetch unless fresh), compare hashes, pull and/or push. `--refresh` forces fetch. `--force-push` skips pull and overwrites cloud with local (confirm / `--yes`) |
| `zync status` | Show init state + pending diffs against store clone. Reuses recent refresh; `--refresh` forces fetch |
| `zync login` | Ensure `gh` auth (confirm before reusing existing session) |
| `zync purge` | Print plan + confirm; delete local config + store cache. `--remote` also deletes GitHub repo |
| `--yes` | Skip confirmation prompts on `init` / `login` / `purge` |

Destructive or identity-sensitive actions (reuse `gh`, create/use repo, purge) require confirmation unless `--yes`.

## Local state

- **Config:** platform zync config dir → `config.json` (`deviceId`, `remote.repo`/`branch`, categories, `lastSyncedAt`, …)  
- **Cache:** platform zync cache dir → `stores/<owner_repo>/` — git clone of the store  
- Env overrides: `ZYNC_CONFIG_DIR`, `ZYNC_CACHE_DIR`, `ZED_CONFIG_DIR`, `ZED_DATA_DIR`  

`purge` removes local config + cache; it does **not** modify Zed `settings.json` / keymap on disk.

## Remote store layout

```
manifest.json          # version, updatedAt/By, content flags, per-file SHA-256 hashes
config/settings.json
config/keymap.json
config/tasks.json      # optional
config/debug.json      # optional
threads/…              # Phase 2 (not implemented)
```

## What syncs

| Category | Default | Mechanism |
|---|---|---|
| Settings | on | `settings.json` |
| Keybindings | on | `keymap.json` |
| Tasks / debug | on | `tasks.json`, `debug.json` |
| Extensions | on | On push: merge installed extension folder names into `settings.auto_install_extensions`, then sync settings. Binaries are not copied |
| Agent threads | off | Store shape reserved; export/import not implemented |

## Sync algorithm (`zync sync`)

1. Load local zync config (must be initialized).  
2. **Prepare store dir:** clone if missing; else if last refresh older than **TTL 120s** (or `--refresh`) → `git fetch` + checkout + `reset --hard origin/<branch>`; if still fresh → skip fetch (local checkout/reset only). Stamp lives in `.git/zync-last-refresh` (shared by `sync` and `status`). Successful push also updates the stamp.  
3. Read remote `manifest.json` hashes; hash local Zed files.  
4. Decide (unless `--force-push`):
   - no remote hashes → push local  
   - all same → no-op  
   - remote-only / different → pull (backup locals), then push  
   - local-only → push  
   With `--force-push`: skip pull; always push local over remote (after confirm).  
5. Update `lastSyncedAt`.  

Progress lines go to **stderr** (`… …`).

## Auth model

Zed’s own Sign In is not used. Identity = current `gh` user. Interactive `gh auth login` only if no session; otherwise confirm reuse.

If `gh` is not installed, `init` / `login` explain that and offer to install it (`winget` on Windows, `brew` on macOS, `apt-get` on Linux when available); `--yes` accepts the offer. Before offering install, also look for an already-installed `gh` outside the current PATH (common after winget: `C:\Program Files\GitHub CLI`, plus Machine/User Path from the registry). Manual install: https://cli.github.com.

## Long-term shape

```
Zed UI (optional extension)  →  process:exec  →  zync binary  →  git + gh + local Zed files
```

The CLI remains the source of truth for sync logic.
