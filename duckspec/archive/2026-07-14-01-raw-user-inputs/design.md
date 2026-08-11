# Raw user inputs in the change package - Design

Duckboard exports a precise-secretary markdown ledger of verbatim user chat into
`duckspec/changes/<name>/inputs.md`, refreshes it while the change is active, and lets
archive keep it by moving the whole change folder.

## Approach

```
chat_store                     change package
chats/<change>/*.json          duckspec/changes/<name>/
  (local, full session)          proposal.md / design.md / …
        │                        inputs.md  ← this change
        │ export filter                 │
        │ (User text, !priming)         │
        └──────── rewrite ──────────────┘
                                        │
                                 ds archive (rename folder)
                                        ▼
                              duckspec/archive/…/inputs.md
```

- **Canonical chat bits** stay in `chat_store` (`Role`, `ContentBlock`, tools).

- **Package ledger** is a derived, human- and agent-readable markdown view — user words
  only, structure only (precise-secretary).

- **Writer:** duckboard only (it owns sessions). `ds archive` needs no new logic for
  archaeology; the file is just another path under the change dir.

- **Scope:** only `ScopeKind::Change` (session.scope == change name, and
  `duckspec/changes/<name>/` exists). Exploration chat is exported only after promotion
  merges into the change scope.

- **Refresh model:** full rebuild from all sessions in that scope after durable session
  writes (not append-only). Skip disk write when bytes are unchanged to limit VCS noise.

```
save_session(change-scoped)
        │
        ▼
load_sessions_for(change)
        │
        ▼
render_inputs_markdown(sessions)  ── pure
        │
        ▼
if changed: write_atomic(…/inputs.md)
```

Also re-export once after exploration→change `merge_scope` so pre-promotion words appear
immediately.

## Ledger format (`inputs.md`)

Soft package file (no new `ds` schema type in v1). Freeform markdown agents can read;
`ds check` does not validate it specially.

```markdown
# Raw user inputs

Verbatim user messages for change `<name>`. Not a summary.

## <session label>

<timestamp>

<user text, exact>

<timestamp>

<user text, exact>
```

- One `##` section per session, ordered by `created_at_nanos` ascending (story order for
  archaeology).

- Session label: `title` if set, else `display_name` / stable id.

- Message order: session message order.

- Body: only `Role::User` messages with `is_priming == false`.

- Text: concatenate `ContentBlock::Text` in order; ignore non-text user blocks.

- Preserve text **byte-for-byte** (no trim of intentional whitespace beyond what was
  stored; no paraphrase).

- Blank line between messages. No auto takeaways, no assistant answers, no tools, no
  reasoning.

- If zero qualifying messages: **do not create** the file (or remove empty file if a prior
  export became empty — prefer leave absent).

## Export core (`chat_store` / small sibling)

Pure render + path helper next to session I/O (e.g. `chat_store` or `inputs_ledger.rs`
under duckboard):

```rust
/// Project-relative / absolute path: duckspec/changes/<name>/inputs.md
pub fn inputs_path(project_root: &Path, change: &str) -> PathBuf;

/// Full rebuild from loaded sessions (already sorted or sort inside).
pub fn render_inputs_markdown(change: &str, sessions: &[ChatSession]) -> String;

/// Load scope sessions, render, write atomically if changed and change dir exists.
pub fn export_inputs_for_change(
    change: &str,
    project_root: &Path,
) -> anyhow::Result<ExportOutcome>;

pub enum ExportOutcome {
    Written,
    Unchanged,
    SkippedNoChangeDir,
    SkippedEmpty,
}
```

Reuse atomic write pattern from `write_atomic` in `chat_store.rs` (temp + rename beside
target; use a stable temp name that cannot collide with real artifacts).

## Write triggers (duckboard)

Hook **after successful** `save_session` when the session’s scope is a change name (and
change directory exists):

```rust
// after save_session(&session, Some(root)) for change scope:
let _ = export_inputs_for_change(&session.scope, root);
```

Call sites already centralize persistence in `chat_store::save_session` and helpers in
`area/interaction.rs` (`persist_session_snapshot`, flush paths). Prefer **one chokepoint**
inside `save_session` (or a thin wrapper `save_session_and_export`) so eager/coalesced
flushes and turn-end saves all refresh the ledger without hunting every caller.

After promotion (`merge_scope` / `promote_*` in `main.rs`): one explicit
`export_inputs_for_change(change_name, root)`.

Do **not** export for Caps/Codex/Exploration scopes.

## Agent re-grounding

**Orientation** (`scope.rs` / `CurrentScopeHook` for `ScopeKind::Change`):

- If `duckspec/changes/{name}/inputs.md` exists and is non-empty, add one short sentence +
  path: raw human inputs for this change; read before inventing motivation; do not rewrite
  that file.

**Templates (light):** propose / design / explore (and optionally review) mention the same
path when present — “prefer quotes from `inputs.md` over paraphrase.” No new stage; no
requirement that the file always exist.

## Capability layout (for later spec)

New capability under chat, e.g. `chat/inputs-ledger` (name flexible at spec time): export
rules, file location, filter, refresh triggers, orientation mention. Adjacent to
`chat/persistence` (store stays as-is; ledger is derived).

## Impact

- **duckboard:** export module + save/promote hooks + orientation line
- **Templates / session orientation:** small copy + path mention
- **duckpond / `ds` archive / create:** no change required for v1 archive path
- **VCS:** `inputs.md` dirty on user sends for active change-scoped chats
- **CLI-only workflows:** no ledger without duckboard (documented limit)

## Decisions

- **Filename `inputs.md`** — short, package-native, archive-safe. Alternatives:
  `raw-inputs.md`, `human.md` (rejected: longer / vaguer).

- **Derived full rebuild** — not append-only log. Alternatives: append events (rejected:
  harder to dedupe, messier after session edit/delete).

- **Duckboard-only writer** — sessions live in app data. Alternative: `ds` subcommand
  reading chat data (rejected for v1: couples CLI to duckboard data layout).

- **User-only, drop priming** — precise-secretary + existing `is_priming`. Alternative:
  full transcript (rejected by proposal non-goals).

- **Include bare confirms / slash-only user lines** — audit purity over scannability.
  Alternative: filter tokens (can add later if noisy).

- **Soft artifact** — no `ds create inputs` / schema type in v1. Alternative: first-class
  create command (premature).

- **Agent questions** — deferred; not in render v1.

## Risks

- **VCS churn on every user message** → skip write when content hash/bytes equal; coalesce
  already exists on session save (~1s) so export rides that.

- **Human-edited `inputs.md` overwritten** → document as machine-owned export; re-export
  clobbers. If needed later: generate into `inputs.generated.md` — not v1.

- **Promotion race** → explicit export after merge; save_session chokepoint covers later
  turns.

- **Large chats** → acceptable; ledger is text-only and smaller than full JSON sessions.
