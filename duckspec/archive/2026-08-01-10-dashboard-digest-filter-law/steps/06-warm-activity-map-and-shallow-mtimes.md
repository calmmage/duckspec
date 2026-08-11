# Warm activity map and shallow mtimes

Cache chat activity off the paint path; restore load-time shallow mtimes for ranking
inputs.

## Prerequisites

- [x] @step dashboard-section-plan-and-filter-law-ui

## Context

Review finding 3: every Dashboard paint called `collect_chat_activity` →
`load_sessions_for` per scope. Design wants warm inputs. Also restore
`ChangeData.shallow_mtime_nanos` if missing after tree drift.

## Tasks

- [x] 1. Restore `ChangeData.shallow_mtime_nanos` on project load/reload (dir + immediate
         children)

- [x] 2. Own a warm `HashMap` of scope → activity nanos (app/`dashboard` state or
         project-adjacent cache)

- [x] 3. Recompute map on project open, project reload/watcher reconcile, and session
         write/flush paths that affect scopes

- [x] 4. Dashboard ranking only reads the warm map — no `collect_chat_activity` /
         `load_sessions_for` inside `view_items_panel`

- [x] 5. Confirm mtime + activity still feed `change_rank_inputs` /
         `exploration_rank_inputs`

- [x] 6. Smoke that the paint path does not open session files (unit-level spy if cheap)
