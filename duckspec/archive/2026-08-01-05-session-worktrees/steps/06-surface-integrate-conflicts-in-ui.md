# Surface integrate conflicts in UI

Show conflict/fail paths to the user on archive auto-merge and explicit merge — not logs
only.

## Prerequisites

- [x] @step integrate-to-main

## Context

From review finding 1: `run_integrate_to_main` only `tracing::warn`s on conflict; spec
requires surfacing paths.

## Tasks

- [x] 1. On `IntegrateOutcome::Conflict` / `Failed` from `run_integrate_to_main`, surface
         paths or message in the UI (e.g. system message on the scope’s active chat
         session)

- [x] 2. Keep sidecar binding on conflict; do not claim success

- [x] 3. @spec worktree/stack-and-merge Integrate to Main: Conflict stops without claiming success
