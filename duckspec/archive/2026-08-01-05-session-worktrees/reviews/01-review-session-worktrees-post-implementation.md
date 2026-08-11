# Review: Session worktrees post-implementation

Mechanical integrity is clean (check, audit, 29 unit tests), but three user-facing
fidelity gaps need a step pass before freeze: surface integrate conflicts, walk the Files
explorer from the active work root, and stop auto-landing stacked dependents when a base
merges.

## Scope

Reviewed `session-worktrees` proposal, design, caps under `worktree/`, steps 01–05,
`crates/duckboard/src/worktree.rs`, main/change wiring (placement UI, active root, archive
integrate, merge button), standing VCS instructions, and `worktree::tests` (FakeOps). Ran
`ds check`, `ds audit session-worktrees` (ok), and unit tests (29 passed).

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Integrate conflicts not user-visible | Surface paths in UI (not logs only) | /ds-step |
| 2 | Files explorer walks main, not active root | Explorer follows `active_work_root` | /ds-step |
| 3 | Dependents full-integrate when base merges | No auto on dependents (C) — only the finished scope integrates | /ds-step |
```

## Findings

### 1. Integrate conflicts not user-visible

**Where:** Spec `worktree/stack-and-merge` Integrate to Main (surface paths);
`run_integrate_to_main` in `crates/duckboard/src/main.rs` (tracing only on conflict/fail).

**Evidence:** Conflict keeps the sidecar binding (correct) but only `tracing::warn`s.
Archive auto-merge and explicit **merge** share this path. Unit tests assert
`IntegrateOutcome::Conflict`, not human-visible output.

**Impact:** Archive/merge can look like a silent no-op; user never sees conflict paths.

**Discussion:** Softening the spec to log-only weakens human-driven merge. UI surface
(system message on the scope’s chat, or a short plaque listing paths) matches the contract
with small work.

**Resolution:** Conflicts must be user-visible (option A). Keep outcome semantics; add UI
feedback with paths.

**Next:** `/ds-step` - implement conflict/fail surfacing for integrate (archive + explicit
merge).

### 2. Files explorer walks main, not active root

**Where:** Design active-root table (explorer from focused root); `refresh_project_files`
in `crates/duckboard/src/main.rs` uses `project.project_root` only. Changed files / file
open already use active root.

**Evidence:** Explorer tree is always the main project walk while Worktree scopes use a
sidecar for open/diff.

**Impact:** Tree and open target disagree; easy to think main files are “this idea’s”
tree.

**Discussion:** Keeping explorer on main was rejected; walking `active_work_root` matches
design (sparse sidecars may look thinner — acceptable).

**Resolution:** Explorer follows `active_work_root` (option A).

**Next:** `/ds-step` - point `refresh_project_files` (and related reveal assumptions) at
the active work root.

### 3. Dependents full-integrate when base merges

**Where:** Design/spec stack-after-parent (MAY rebase/update when clean);
`update_dependents_after_base_merge` → `integrate_to_main` on each dependent.

**Evidence:** Clean merge of base A also lands clean dependents B into Main and forgets
their sidecars — stronger than “B keeps WIP on top of new Main.”

**Impact:** Unfinished stacked work can land on Main as a side effect of finishing A.

**Discussion:** Rebase-only is better stacking UX later but more CLI work. Full auto-land
needs explicit product intent. **No auto on dependents (C)** is simplest and safest: only
the finished scope integrates; B waits for explicit merge/archive.

**Resolution:** C — no auto-integrate of dependents after base merge.

**Next:** `/ds-step` - remove/disable dependent auto-integrate; base merge finishes only
that scope (metadata-only follow-up optional).

## Resolved concerns

**CLI untested beyond FakeOps.** Production `CliWorktreeOps` is not exercised in automated
tests. Accepted for v1 (unit + FakeOps + audit sufficient to freeze after the three step
fixes); optional later integration tests / dogfood checklist, not a blocking finding.

## Outcome

Not ready to archive. Earliest route is **`/ds-step`**: conflict UI, active-root explorer,
and no dependent auto-land. Design/spec stay directionally valid for those three;
contracts do not need a soft-down for conflict surfacing.
