# Agent notes (zync)

## Plan before code

Before changing code, **always show a short plan and wait for approval**. Do not edit implementation files until the user okays the plan. Doc-only updates to `AGENTS.md` / `.ai/*` / README when the user asked for them directly are fine without a separate plan.

## Docs stay in sync

On **every** meaningful change, update documentation in the same turn:

- `README.md` — user-facing install, commands, behavior
- other docs if present (including `.ai/` when relevant)
- code comments / module docs when they describe public behavior or contracts

Do not leave README, comments, or docs describing the old TypeScript/`watch` world (or any other stale behavior).

## `.ai/` layout

| Path | Role |
|---|---|
| `.ai/state.md` | Short-term working memory (now / next) |
| `.ai/skills/*.md` | Optional topic skills (e.g. Zed extension API) — create when needed |

### `.ai/state.md` — working memory

- what we are working on **now**
- what is planned **next**

Do **not** keep a changelog of finished work there — completed work lives in **git history**. When something ships, remove it from `.ai/state.md` (or replace with the new current focus).

### `.ai/skills/`

Add a skill file only when it earns its keep (e.g. `.ai/skills/zed.md` for Zed extension / WASM / capabilities quirks). Prefer linking to official docs and capturing project-specific gotchas, not dumping generic tutorials.
