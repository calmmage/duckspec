# Review: Sticky last-user post-implementation

Sticky last-user is largely faithful to proposal and design (source, top-edge visibility,
top-of-chat slot, height cap). Two agreed follow-ups: stop wiping the content-top cache on
every rematerialize, and drop width-reflow remeasure from the design’s v1 measurement
table.

## Scope

Proposal, design, `chat/sticky-last-user` spec/doc, steps 01–02, `agent_chat.rs`
helpers/view/measure, `interaction.rs` session cache/sync, `main.rs` chrome measure batch,
and unit tests. `ds audit sticky-last-user-prompt` clean (9/9 linked). No prior reviews.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Content-top cache cleared on every materialize | Invalidate only when last-user source idx/body changes; keep collapse/focus invalidation | `/ds-step` |
| 2 | Design claims width-affecting remeasure; code does not | Soften design: v1 invalidates on source change, collapse, focus fold only | `/ds-design` |
```

## Findings

### 1. Content-top cache cleared on every materialize

**Where:** `sync_sticky_last_user` in `crates/duckboard/src/area/interaction.rs`;
`sticky_last_user_visible` hide-when-top-unknown in
`crates/duckboard/src/widget/agent_chat.rs`; streaming rematerialize cadence via
`materialize_chat_ui`

**Evidence:** After every materialize, `sticky_last_user_content_top` is set to `None`.
Visibility treats missing top as hidden. Stream turns rematerialize on cadence while
stick-to-bottom follows the answer — the main sticky use case — so the panel can flicker
off until measure returns. Appending Answer content below the last user does not change
that block’s content top.

**Impact:** Sticky can blink or briefly vanish during long agent turns when the user is
following the live reply — the scenario the feature exists for.

**Discussion:** (A) Keep last top until a new measure overwrites — avoids hide but can
briefly use a stale top after real earlier-layout shifts. (B) Invalidate only when
last-user idx or body text changes; collapse and focus fold still clear — matches when top
actually needs refresh. (C) Leave as-is — simple, worse UX. **B** chosen.

**Resolution:** Do not clear content top solely because the transcript rematerialized.
Clear when last-user source identity (block index) or body text changes; keep collapse-
and focus-fold invalidation. Remeasure after those invalidations as today.

**Next:** `/ds-step` — adjust `sync_sticky_last_user` (and any needed prior-idx tracking)
so streaming rematerialize does not force-hide the sticky

### 2. Design claims width-affecting remeasure; code does not

**Where:** `design.md` Measurement and state table (“width-affecting chat layout”);
implementation invalidates on materialize/collapse/focus fold only — no window/pane width
path

**Evidence:** Design lists width-affecting layout as a remeasure trigger. Code never
clears or remeasures solely on chat column or window resize. Wrap height above the last
user can change without cache invalidation.

**Impact:** Rare: resize while sticky is (or should be) visible can leave a slightly stale
top until another invalidation. Much less common than stream rematerialize flicker.

**Discussion:** (A) Wire width/layout invalidate and remeasure — matches design, more
surface. (B) Soften design for v1 — honest, leave rare edge. (C) Leave mismatch
undocumented. **B** chosen.

**Resolution:** Amend design measurement table: v1 invalidation is source identity/body
change, collapse toggle, and focus fold only. Width reflow is not a v1 remeasure trigger.

**Next:** `/ds-design` — update Measurement and state (and any related prose) to match
agreed v1 invalidation set

## Resolved concerns

Live visual smoke was not run in the apply session; pure visibility helpers and layout
wiring cover the contract. Not treated as a product finding.

Unconditional materialize wipe vs design “remeasure on rematerialize” was read as
over-implementation of hide, not a missing remeasure path — closed under finding 1.

## Outcome

Not ready to archive. Amend design for v1 invalidation (finding 2), then implement
selective cache invalidation (finding 1). Spec scenarios remain aligned; no contract
rewrite required for these resolutions.
