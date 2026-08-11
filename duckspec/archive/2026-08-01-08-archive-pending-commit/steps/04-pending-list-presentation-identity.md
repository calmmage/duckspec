# Pending list presentation identity

While an archive is pending, Change-list chrome: breadcrumbs `Changes / <id>` and select
reveals `picker`; finished archives keep Archive chrome.

## Prerequisites

- [x] @step pending-predicate-helpers
- [x] @step uncommitted-chrome-and-commit-send

## Context

Review finding 1 (option A): pending packages already sit on the Change list with
uncommitted chrome, but breadcrumbs and `SelectChange` still treat any name in
`archived_changes` as list-Archived. Fix presentation only — tabs/overview/chat stay on
archive identity (`prefix = archive/…`).

## Tasks

- [x] 1. In `crates/duckboard/src/area/change.rs`, treat list-archived only when the
         package is archived **and not** pending (`is_pending_commit_archive` + dirty
         paths): breadcrumbs root `Changes` while pending (including tab/VCS breadcrumb
         roots that currently key off “in archived_changes” or `archive/` path prefix);
         keep artifact tab ids / overview / chat on archive identity

- [x] 2. On `SelectChange`, expand `picker` for pending archives and `archived` only for
         finished archives / archived explorations (so plaque/`Commit` selection reveals
         the Change-section row)

- [x] 3. Unit tests: pending selection → breadcrumb root `Changes` and `expanded_sections`
         contains `picker`; finished archive still → `Archive` / `archived`
