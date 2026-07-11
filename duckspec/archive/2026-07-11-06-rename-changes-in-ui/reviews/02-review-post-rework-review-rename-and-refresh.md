# Post-rework review: rename and refresh

Re-reviewed after steps 03–04 addressed the post-implementation findings. Prior majors on
cold-handle refresh and hollow apply-path tests are fixed; change is archive-ready with
optional polish only.

## Scope

Prior review `01-review-post-implementation-…`, steps 03–04, and current code in
`chat_store::apply_title_labels`, `apply_session_title_inner`,
`start_exploration_title_refresh`, `AgentSession::pending_title_refresh`, and
`AgentEvent::Ready`. Check/audit green; 6/6 scenarios linked; all four steps complete.

## Summary

```
| # | sev | lens | title | → next |
| --- | --- | --- | --- | --- |
| 1 | minor | quality | Rename still second-click only | ignore |
```

## Findings

### 1. Rename still second-click only - quality/minor

**Where:** `crates/duckboard/src/area/change.rs` SelectChange re-click / exploration
`view_list` (same as finding 3 in review 01)

**Why:** Refresh has a visible ↻; rename remains discoverable only by re-clicking a
selected row. Low compounding cost; deliberately left out of steps 03–04.

**Action:** Optional pencil control later, or leave as-is.

## Prior findings (resolved)

- **Cold-handle refresh** — `pending_title_refresh` + flush on `Ready`; interaction
  ensured on refresh so hover-only rows work.

- **Hollow apply path** — `apply_title_labels` shared by tests and
  `apply_session_title_inner`; overwrite / empty / no-content scenarios retargeted.

## Verdict

Accept as done / archive-ready. Remaining rename affordance is optional polish, not a
blocker.
