# Focus presentation and folds

Apply Focus layout for settled Answers, fold state, chrome, and live Classic rule.

## Prerequisites

- [x] @step classic-answer-presentation-wire
- [x] @step focus-geometry

## Tasks

- [x] 1. Wire `focus_layout` into Answer presentation when effective focus + not live +
         open region

- [x] 2. Ephemeral per-Answer section fold state (`user_set`, clear on leave Focus)

- [x] 3. Focus chrome: collapsible headers + classic paint for expanded bodies / open
         region

- [x] 4. @spec chat/focus-answer When Focus applies: Live Answer under focus style uses classic presentation

- [x] 5. @spec chat/focus-answer When Focus applies: Settled Answer without trailing next uses classic passthrough

- [x] 6. @spec chat/focus-answer When Focus applies: Settled Answer with trailing next uses Focus layout

- [x] 7. @spec chat/focus-answer Fold defaults and toggles: First sight collapses foldable sections and shows open region

- [x] 8. @spec chat/focus-answer Fold defaults and toggles: User expand survives rematerialize for the same section key

- [x] 9. @spec chat/focus-answer Fold defaults and toggles: Leaving Focus clears section fold state

- [x] 10. @spec chat/focus-answer Focus slice presentation (Hybrid C): Collapsed preamble label uses line count form

- [x] 11. @spec chat/viewer-style Effective viewer style: Stored focus yields effective focus when Focus is implemented

## Outcomes

- `FOCUS_ANSWER_IMPLEMENTED` is `true`; Settings list still only gains Focus in step 05.

- Step 04 shipped plain content-font Focus slices; Hybrid C open-region TextEdit and
  open-region-only band land in step 06.
