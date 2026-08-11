# Dashboard section plan and filter-law UI

Restore left-column digest UI from pure helpers and render sections only via a single
section-plan API (so section-set tests own the real path).

## Prerequisites

- [x] @step pure-rank-and-shortlist-helpers

## Context

Review findings 1–2 and tree drift: `dashboard_digest` helpers exist, but Dashboard UI
wiring (expand flags, shortlists, archived two-level state) was lost. Design documents
`archived_open` + `expanded_archived`. Re-bind the view and consume
`dashboard_left_section_names` so section-set scenarios are not pure-allowlist theater.

## Tasks

- [x] 1. Register `dashboard_digest` and restore `dashboard::State` open/expand flags +
         messages (live expand, `archived_open` / `expanded_archived`,
         `on_project_opened`)

- [x] 2. Build left-column section plan from `dashboard_left_section_names` (and related
         flags); view iterates the plan — no hardcoded Ideas/Stuck

- [x] 3. Wire ranked Changes/Explorations through `filter_law_slice` + overflow; New
         Exploration outside N; Archived Collapsed → Shortlist → Full via
         `archive_filter_law_slice`

- [x] 4. Handle toggle messages + reset on project open in `main`

- [x] 5. @spec shell/dashboard-digest Dashboard section set: Left column sections are Changes Explorations and Archived only

- [x] 6. @spec shell/dashboard-digest Dashboard section set: Ideas and Stuck sections are absent

- [x] 7. @spec shell/dashboard-digest Expand state lifecycle: Project switch resets expand flags

- [x] 8. @spec shell/dashboard-digest Archived density on Dashboard: Dashboard Archived starts collapsed

- [x] 9. @spec shell/dashboard-digest Filter-law shortlist: More than three changes shows three rows and remainder count

- [x] 10. @spec shell/dashboard-digest Filter-law shortlist: Three or fewer changes omits overflow chrome
