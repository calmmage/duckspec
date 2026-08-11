# Load session on scope bind

On scope bind, open the latest shared session for that scope so duckboard history appears.

## Prerequisites

- [x] @step navigator
- [x] @step session-sharing

## Context

Review finding 2: `apply_nav_select` creates `ChatPane::new` only and never loads disk
sessions. “Latest” should follow shared store ordering (most recent /
duckboard-compatible).

## Tasks

- [x] 1. Resolve the latest session for a scope from the shared store (same files as
         duckboard)

- [x] 2. On `SelectResult::Bound`, load that session into `ChatPane` with
         `DriveRole::Displayed` when present; empty new session only if none

- [x] 3. Re-engage stick-to-bottom on intentional open or switch (`chat/session-scroll`
         intent)

- [x] 4. Do not write on bind (displayed-only until a local turn drives)
