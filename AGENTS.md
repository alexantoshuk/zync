# Agent notes (zync)

## Workflow (order matters)

1. **Update `.ai/state.md` first** — set **Now** to the task about to start (and adjust **Next** if needed). Do this *before* planning or coding.
2. **Plan → wait for approval** — show a short plan; do not edit implementation files until the user okays it. Doc-only updates the user asked for directly (`AGENTS.md` / `.ai/*` / README) are fine without a separate plan.
3. **Implement** — code + docs/comments in the same turn (see below). Leave **Now** as the feature under test; you may note “implemented, awaiting user test/approval”.
4. **Close out state only after user approval** — a feature is **not** done when the agent finishes coding. Until then it stays in **Now** (e.g. “awaiting your test + approval”).
5. **Pre-commit hygiene** — before creating a commit the user asked for, quickly check and fix if needed:
   - docs (`README.md`, `AGENTS.md`, `.ai/product.md`, comments) match current behavior
   - no leftover experimental hacks / TODOs that were only for debugging
   - no obvious dead code, unused imports, or stale commands/flags
   - `.ai/state.md` updated for the approve/`commit` rule below
6. **`commit` = approve** — when the user asks to **commit**, treat that as approval of the work being committed. In the same turn: run step 5, clear finished items from **Now** in `.ai/state.md` (or leave only what’s still open), *then* create the git commit (include the updated `state.md`). Explicit “approved” / “lgtm” / similar also closes **Now**. Do **not** clear **Now** just because you finished implementing or because *you* ran git without the user asking.

## Docs stay in sync

On **every** meaningful change, update documentation in the same turn:

- `README.md` — user-facing install, commands, behavior
- `.ai/product.md` — product model and behavior (keep aligned with code)
- other docs if present (including `.ai/` when relevant)
- code comments / module docs when they describe public behavior or contracts

Do not leave README, comments, or docs describing stale behavior.

**If product behavior changes, update `.ai/product.md` in the same change.** That includes commands, sync rules, auth, conflicts, store layout, what is/isn’t synced, confirmations, etc. Read it before changing those areas; do not ship code that contradicts `product.md`.

## `.ai/` layout

| Path | Role |
|---|---|
| `.ai/product.md` | Product + behavior (source of truth for how zync works) |
| `.ai/state.md` | Short-term working memory (now / next) |
| `.ai/skills/*.md` | Optional topic skills (e.g. Zed extension API) — create when needed |

### `.ai/product.md`

Canonical description of the product, commands, sync algorithm, store layout, and auth. **Required update** whenever behavior changes. Prefer this file over inventing behavior from memory. User-facing details also belong in `README.md`.

### `.ai/state.md` — working memory

- **Now** — active work (update at the *start* of a task); stays until the user tests and approves
- **Next** — upcoming plans only

Not a changelog. Items leave **Now** after user approval — including when they say **commit** (see workflow step 6) — not after implement alone.

### `.ai/skills/`

Add a skill file only when it earns its keep (e.g. `.ai/skills/zed.md` for Zed extension / WASM / capabilities quirks). Prefer linking to official docs and capturing project-specific gotchas, not dumping generic tutorials.
