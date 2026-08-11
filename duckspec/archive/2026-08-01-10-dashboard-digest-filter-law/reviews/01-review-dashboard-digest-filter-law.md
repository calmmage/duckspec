# Review: Dashboard digest filter law

Implementation matches the attention/shortlist intent, but design under-specifies Archived
open state and two step-level fixes are needed before freeze: single section-plan owner
for the left column, and warm-cached chat activity.

## Scope

Reviewed `proposal.md`, `design.md`, `caps/shell/dashboard-digest` (spec + doc), all four
steps, `crates/duckboard/src/dashboard_digest.rs`,
`crates/duckboard/src/area/dashboard.rs`, mtime wiring in `data.rs`, `ds check` /
`ds audit` (clean, 21/21 linked), and the 21 unit tests.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Archived two-level state missing from design | Document `archived_open` + shortlist/full expand | `/ds-design` |
| 2 | Section-set tests bypass the view | View consumes one section-plan API tests already cover | `/ds-step` |
| 3 | Chat activity reloaded every paint | Cache on open/watcher/session write; rank from warm map | `/ds-step` |
```

## Findings

### 1. Archived two-level state missing from design

**Where:** `design.md` Filter-law UX expand-flag sketch; `dashboard.rs` `archived_open` +
`expanded_archived`

**Evidence:** Design shows a single `expanded_archived` defaulting false for “section
starts collapsed.” Code needs header-only collapse (`archived_open`) and, when open,
prefix shortlist vs full list (`expanded_archived`). Spec scenarios still hold.

**Impact:** A later pass may collapse both into one flag and break shortlist-after-open.

**Discussion:** Dismissing as free naming leaves durable ambiguity. Amending design is
small and keeps the earliest layer accurate without changing behavior.

**Resolution:** Amend design to name both flags and the Collapsed → Shortlist → Full path;
no 2+1 on archive.

**Next:** `/ds-design` - update expand-state section and any related diagrams/tables.

### 2. Section-set tests bypass the view

**Where:** `caps/shell/dashboard-digest` Dashboard section set scenarios;
`dashboard_digest::{dashboard_left_section_names,is_dashboard_left_section}`;
`view_items_panel` hardcodes section titles

**Evidence:** Spec WHEN is “left column is shown.” Tests only assert pure allowlist
helpers. The view never calls those helpers—it hardcodes Changes / Explorations /
Archived.

**Impact:** Adding an Ideas section would not fail the linked scenarios; coverage is
audit-green theater for this requirement.

**Discussion:** Softening the spec weakens the UX contract. Wiring the view through one
section plan (tests already own) restores fidelity with a small refactor.

**Resolution:** Single presentation plan for left-column sections; view renders from it;
existing tests stay meaningful.

**Next:** `/ds-step` - plan/implement view consumption of the section plan (after design
amend).

### 3. Chat activity reloaded every paint

**Where:** `dashboard.rs` `view_items_panel` → `collect_chat_activity` → per-scope
`load_sessions_for`; contrast load-time `ChangeData.shallow_mtime_nanos`

**Evidence:** Every Dashboard paint loads session files for all active change names and
exploration ids. Design assumed cheap warm signals at view time.

**Impact:** Cost scales with inventory; risk of hitches on large projects.

**Discussion:** Throttling is a half measure. Leaving as v1 risk is viable for small
inventories but fights Dashboard-as-home. Caching on open / watcher / session write
matches “warm inputs.”

**Resolution:** Maintain a warm activity map; recompute on project open, project data
refresh/watcher, and session writes; Dashboard ranking only reads the map.

**Next:** `/ds-step` - plan/implement activity cache wiring (after design amend).

## Resolved concerns

Ranking (recency + needs_work), day-seeded 2+1 for live sections, archive prefix shortlist
without day spin, derived-only scores, expand reset on project open, and audit-panel
non-touch were rechecked and match the settled design.

## Outcome

Not ready to freeze. Amend design for Archived open/expand state first, then step work for
section-plan wiring and chat-activity cache. Spec contracts stay; no `/ds-spec` required
from this pass.
