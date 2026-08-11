# Stack base and require-merged gate

Optional `base_scope` with cycle rejection, sidecar seed from the base revision, and
opt-in block until the base is on Main.

## Prerequisites

- [x] @step worktree-manager-and-placement-policy
- [x] @step active-root-plumbing

## Tasks

- [x] 1. Persist `base_scope` and require-base-merged; UI to stack on another scope and
         toggle the flag; reject cycles

- [x] 2. `ensure_sidecar` seeds from the base scope’s work revision when set; clearing
         base returns trunk-based create policy

- [x] 3. Block starting an agent turn when require-base-merged is set and the base is not
         integrated to Main; surface the base scope

- [x] 4. @spec worktree/stack-and-merge Stack base: Stacked sidecar is created from the base scope’s work revision

- [x] 5. @spec worktree/stack-and-merge Stack base: Cyclic base is rejected

- [x] 6. @spec worktree/stack-and-merge Stack base: Clearing base returns create policy to trunk

- [x] 7. @spec worktree/stack-and-merge Require base merged: Blocked send while base unmerged and flag set

- [x] 8. @spec worktree/stack-and-merge Require base merged: Send allowed when flag unset even if base unmerged

- [x] 9. @spec worktree/stack-and-merge Require base merged: Send allowed after base integrated to Main
