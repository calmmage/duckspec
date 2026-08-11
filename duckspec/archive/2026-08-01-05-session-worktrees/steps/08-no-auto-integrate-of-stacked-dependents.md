# No auto-integrate of stacked dependents

When a base scope merges to Main, only that scope integrates; stacked dependents wait for
explicit merge or archive.

## Prerequisites

- [x] @step integrate-to-main
- [x] @step stack-base-and-require-merged-gate

## Context

From review finding 3 (resolution C): `update_dependents_after_base_merge` must not call
`integrate_to_main` on dependents. Spec MAY rebase/update is intentionally not exercised
as full land in this cut.

## Tasks

- [x] 1. Change `update_dependents_after_base_merge` so it does not call
         `integrate_to_main` on dependents (optional: leave `base_scope` metadata as-is)

- [x] 2. Update the post-base-merge unit test so a clean dependent is **not** forced onto
         Main

- [x] 3. @spec worktree/stack-and-merge Stack after parent merge: After base merges cleanly, dependent may rebase onto Main when clean
