# Session worktrees - Design

Per-scope working trees with an explicit **Main vs Worktree** placement, auto-fork when
useful, auto-merge when safe (including on archive), stable human-readable workspace
names, and optional **stack-on** bases so idea B can sit on idea A’s tree.

## Approach

```
  scope placement (UI):  Main  |  Worktree  |  Auto
                              │
         ┌────────────────────┼────────────────────┐
         ▼                    ▼                    ▼
   project_root        .duckboard/worktrees/   policy picks
   (small fixes)       <stable-name>/          Main vs sidecar
                              ▲
                              │ optional base = parent scope
                     create from parent WC rev
                     (stacked ideas)

  focus Scope X → active_work_root(X)
       ├─► changed_files / diffs / explorer
       ├─► file: tabs (relpath under that root)
       └─► agent_subscription(…, work_root)
```

**Isolation unit = scope** (exploration / change row). Chat tabs under one scope share one
root. Caps/codex always Main.

**Workflow gate:** placement + auto-fork only when `Config.vcs.workflow` is `Jj` or
`Worktrees`. Plain `Git` = single tree.

```
| Workflow | Create / forget | Status / diff |
| --- | --- | --- |
| `Jj` | `jj workspace add/forget` | gix on workspace path |
| `Worktrees` | `git worktree add/remove` | gix |
```

## Placement control (Main vs Worktree)

Each scope has a persisted **placement** the UI can set (toggle / menu on the CHANGE row
or change header — minimal chrome):

```
| Placement | Meaning |
| --- | --- |
| **Main** | Always `project_root`. No sidecar, no merge path. Default for small fixes. |
| **Worktree** | Always own sidecar (create if missing). |
| **Auto** | Default for new explorations under Jj/Worktrees: stay on Main until Main is already “in use” dirty by another scope, then fork this scope to a sidecar. |
```

Pinning **Main** must win over auto-fork so two scopes can share Main on purpose (user
accepts interleaved dirty / path-scoped commits). Pinning **Worktree** must win even when
Main is clean.

```rust
enum ScopePlacement { Main, Worktree, Auto }

struct ScopeWorktree {
    scope_key: String,
    placement: ScopePlacement,
    work_root: PathBuf,
    backend: WorktreeBackend,
    /// Stable name: see Naming
    name: String,
    is_main: bool,
    /// Optional stack base: another scope_key whose WC rev seeds this sidecar
    base_scope: Option<String>,
}
```

## Naming (documented, human-stable)

Stable id used for path, jj workspace name, and git branch:

```
duck-<scope_key>
```

- `scope_key` = exploration id or change folder name (already kebab/slug-safe in duckspec)
- Path: `<project>/.duckboard/worktrees/duck-<scope_key>/`
- jj: `jj workspace add --name duck-<scope_key> <path>`
- git: branch `duck/<scope_key>` + worktree at the same path

Document in design/spec and in standing VCS instructions so `jj workspace list` /
`git worktree list` outside duckboard matches the UI. Rename of display title does **not**
rename the worktree id (scope_key is stable; exploration display_name is not).

## Stacked ideas (base scope)

Optional **base** = another active scope (feature 1) when starting feature 2:

```
  Scope A (Worktree or Main, dirty OK)
       │
       │  base_scope = A
       ▼
  Scope B sidecar created from A's current WC revision
  (not from trunk-only)
```

Rules:

- If B has `base_scope = A` and A still has unmerged sidecar work, B’s default
  create/update base is **A’s work_root rev**, not main trunk alone

- UI: “Stack on…” when creating/focusing an exploration, or set on the change header

- **Hard block (optional, same control):** “Require base merged” — while set, first send /
  agent work on B is blocked with chrome pointing at A until A is integrated to Main (or
  user clears the flag / changes base)

- Default for stacked work: **allow** exploration on A’s tree (not block); block is opt-in
  for strict serial queues

- When A merges to Main, B may rebase onto Main automatically when clean; conflict → stop
  and show paths

- Cycles in `base_scope` rejected

## Auto-create & switch

```
on select / first send in S:
  if workflow ∉ {Jj, Worktrees}: Main only
  resolve placement(S):
    Main     → work_root = project_root
    Worktree → ensure_sidecar(S, base=S.base_scope); work_root = sidecar
    Auto     → if main in use by other scope && dirty:
                 ensure_sidecar(...); else Main
  set active_work_root; refresh UI; rebind cold agents
```

Mid-stream sessions: no rebind while streaming.

## Merge / finish (automatic when safe)

```
| Event | Behavior |
| --- | --- |
| Scope stays active | no merge |
| Exploration remove / explicit “merge to main” | integrate sidecar → Main |
| **Archive success** | **automatically** attempt integrate if this change has a sidecar (or unmerged stack commits) |
| Clean / ff-only / empty vs target | auto-complete, `forget` sidecar, clear binding |
| Conflict / non-ff | stop; surface paths; never silent success |
```

Target of integrate is Main’s WC, after stacking parents as needed (merge A before B if B
bases on A — or rebase B onto post-A Main).

## Scope binding & worktree manager

Persist bindings outside the repo (app data), keyed by `project_hash` + scope key.

```rust
enum WorktreeBackend { Jj, Git }

fn ensure_sidecar(
    project_root: &Path,
    scope_key: &str,
    backend: WorktreeBackend,
    base: Option<&ScopeWorktree>,
) -> Result<ScopeWorktree>;

fn forget(binding: &ScopeWorktree) -> Result<()>;

fn is_dirty(work_root: &Path) -> bool; // reuse vcs::changed_files non-empty

fn integrate_to_main(binding: &ScopeWorktree) -> Result<IntegrateOutcome>;
```

Sidecar location and names follow **Naming**. Create from clean trunk base, or from
`base`’s WC revision when stacked.

Exploration → change promotion copies placement, binding, and `base_scope` with the
existing remount path.

## Active-root plumbing

```
| Surface | Today | After |
| --- | --- | --- |
| `refresh_changed_files` | `project_root` only | `active_work_root` |
| `agent_subscription` | all sessions `root = project_root` | per-scope root; fold path into subscription identity |
| `AgentHandle.working_dir` | project root | same as subscription root |
| File / diff tabs | paths under project | relpath via active root; reload or mark missing on switch |
| File watcher | one root | main + every live sidecar |
| PTY cwd | project root | **unchanged** (Main only) |
| Obvious Commit chrome | global dirty | dirty of **active** root only |
```

`changed_files` remains one list for the focused scope’s root. CHANGE rows may badge
sidecar / dirty / stacked base.

## Impact

- `crates/duckboard/src/worktree.rs` (new) + app-data bindings

- `main.rs` / `agent.rs` / watcher / changed-files active-root plumbing

- CHANGE-row or header **placement** control + optional **Stack on** / **require base
  merged**

- Archive success path triggers auto-merge attempt

- `.gitignore` / ignore for `.duckboard/worktrees/`

- Standing VCS instructions: document `duck-<scope_key>`; drop “plumbing not automatic” on
  Worktrees; Jj workspace awareness

- Complements `path-scoped-commits` (path subsets on one tree vs which tree)

## Decisions

- **Explicit Main placement** — small work never forced through worktree/merge

- **Auto still forks when Main is busy** — unless pinned Main

- **Archive → auto merge attempt** — conflict still human

- **Stable `duck-<scope_key>` names** — documented for out-of-band jj/git

- **Stack via `base_scope`** — default work on parent tree; optional hard block until
  parent merged

- **Scope-level isolation, not per chat tab** — worktrees are heavy

- **Jj / Worktrees only** — plain Git stays single WC

- **PTYs stay on Main** — agents are the isolation target

## Risks

- Shared Main with two pinned-Main scopes → intentional interleave; path-scoped commits
  still apply

- Stack rebase after parent merge can conflict → same conflict UI as archive merge

- Display rename ≠ worktree rename → document; avoid silent path churn

- **jj + gix on sidecar paths** → verify early; fall back to `jj` CLI if needed

- **Agent resume ids tied to cwd** → clear on root change when harness requires it

- **Orphan worktrees** → reconcile bindings on project open

- **Disk weight** → acceptable at idea scale

## Open questions

*(none — archive auto-merge, naming, Main pin, and stacking defaults resolved)*
