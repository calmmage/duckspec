# Review: Dashboard digest filter law freeze

Post-fixup pass: prior findings closed, warm activity ownership documented in design, and
implementation matches the settled digest contracts. Ready to freeze.

## Scope

Reviewed `proposal.md`, amended `design.md` (warm map + archive two-level state),
`caps/shell/dashboard-digest` (spec + doc), all six steps,
`crates/duckboard/src/dashboard_digest.rs`, `crates/duckboard/src/area/dashboard.rs`,
mtime load path in `data.rs`, warm-map wiring in `main.rs`, prior review
`01-review-dashboard-digest-filter-law.md`, `ds check` / `ds audit` (clean), and the
linked unit tests (21 scenarios).

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
```

*(No accepted findings this pass.)*

## Findings

*(None.)*

## Resolved concerns

**Review 01 #1 — Archived two-level state.** Design documents `archived_open` +
`expanded_archived` and Collapsed → Shortlist → Full; code and toggle handlers match
(closing clears `expanded_archived`).

**Review 01 #2 — Section-set fidelity.** `view_items_panel` builds the left column only
from `dashboard_left_section_names`; linked section-set tests exercise that plan API.

**Review 01 #3 / design amend A — Warm chat activity.** Design Signal plumbing and
responsibilities name `dashboard::State.chat_activity`, full recompute on open/reload/
external session events, upsert on write/flush, and paint-path read-only ranking. Code
matches: no `collect_chat_activity` / `load_sessions_for` in `view_items_panel`.

**Brief lag after some session writes.** Not every `mark_driven_and_persist` site upserts
immediately; flush tick and turn-end cover the streaming path. Design explicitly accepts
brief lag until recompute/upsert and forbids paint-path compensation — intentional for v1.

Ranking (recency + needs_work), day-seeded 2+1 for live sections, archive prefix shortlist
without day spin, derived-only scores, expand reset on project open, and untouched audit
panel were rechecked and still match design/spec.

## Outcome

Ready to freeze. No design, spec, or step work remains for this change.
