# Review: Style flip editor desync after Hybrid C

Accepted one implementation gap: Hybrid C stores open-region-only lines in the Answer
editor, but style flips do not rematerialize, so Classic/Focus can paint wrong content
until a later rebuild. Prior Hybrid C paint/band work stands. Weak pure-helper tests for
paint/band are dismissed for v1.

## Scope

Proposal, amended design (Hybrid C), caps `chat/viewer-style` and `chat/focus-answer`,
steps 01–06, Focus geometry, Answer presentation / Settings, Hybrid C editor slicing
(`answer_editor_desired_lines`, `rebuild_chat_editor`, `focus_open_region_view`). Prior
review `01-review-focus-paint-hybrid-and-classic-identity.md`. Mechanical: `ds check` /
`ds audit` clean; Focus/viewer-style unit tests green.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Style flip desyncs Hybrid C editor content | Force stamp + rematerialize on effective style change | `/ds-step` |
```

## Findings

### 1. Style flip desyncs Hybrid C editor content

**Where:** `answer_editor_desired_lines` / `rebuild_chat_editor` in
`crates/duckboard/src/widget/agent_chat.rs` and
`crates/duckboard/src/area/interaction.rs`; Settings `ViewerStyleSelected` in
`crates/duckboard/src/area/settings.rs`; design “Style flip: settled Answers update
immediately”; Classic identity bar.

**Evidence:** Under Focus sectioned layout, the chat editor holds open-region lines only
so the open-region TextEdit can reuse Classic paint. Desired content is applied only in
`materialize_chat_ui`. View mode reads `config.chat.effective_viewer_style()` each frame,
so layout flips immediately. Settings save updates config and does not rematerialize.
`ax.viewer_style` is stamped on interaction update, not on Settings alone. After
Classic→Focus, the open-region TextEdit can still hold the full Answer body. After
Focus→Classic, Classic can show only the open-region slice until some later materialize.

**Impact:** Classic identity can break right after leaving Focus (truncated Answer) — the
product bar from the prior review. Focus open region can show non-gate body after entering
Focus. Step 05 “restyle without restart” covers layout mode only, not Hybrid C editor
content.

**Discussion:** (A) On effective style change, stamp sessions and force
`materialize_chat_ui` — smallest fix matching design “update immediately.” (B) Keep full
body always in `chat_editors` and use a separate open-region editor map — cleaner
ownership, larger change. (C) Accept rematerialize lag — conflicts with design and Classic
identity.

**Resolution:** A — force stamp + rematerialize when effective viewer style changes so
editor lines match Hybrid C desired content on the same style flip.

**Next:** `/ds-step` — plan force rematerialize on style change (Settings path and/or
stamp path); clear Focus folds when leaving Focus already exists; ensure rebuild uses the
same effective style as the view.

## Resolved concerns

- **Prior Hybrid C / band / Classic-identity findings (review 01):** design, specs, step
  06, and implementation now match Hybrid C (open-region TextEdit, plain sections,
  open-region-only band). Not reopened.

- **Paint/band unit tests mostly pure helpers:** open-region scenario also checks desired
  editor lines; full iced Element wiring is hard to unit-test. Accepted for v1; not a
  blocking finding. Optional later hardening only.

- **Working-copy noise** (unrelated dirty tree: duckcore/ducktui, etc.): still not a
  product finding for this change.

## Outcome

Not ready to archive. Primary route is **`/ds-step`**: force rematerialize (and consistent
style stamp) on effective viewer-style change so Classic and Focus paint correct editor
content immediately.
