# Review: List digit switch fidelity

Implementation matches product intent and audit is clean; design and spec wording need to
match idea-only / live-queue paint, and Ideas should share one ordering walk with the
view.

## Scope

Reviewed proposal, design, `shell/list-digit-switch` spec/doc, both steps,
`keybinds::keybind_list_digit`, `dispatch_list_digit`, painted-row helpers in
`area/change.rs` and `area/ideas.rs`, and the linked unit tests. `ds check` and
`ds audit list-digit-switch` are clean.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Ideas “every painted row” vs idea-only digits | Keep idea-only; reword to selectable idea rows in paint order | /ds-design |
| 2 | Pending archives on Change digits | Keep indexing; document as part of painted live queue | /ds-design |
| 3 | Parallel Ideas ordering walk | Extract shared pure helper for view + digits | /ds-step |
```

## Findings

### 1. Ideas “every painted row” vs idea-only digits

**Where:** `design.md` visible-row table; `caps/shell/list-digit-switch/spec.md`
Painted-row index; `painted_idea_paths` in `crates/duckboard/src/area/ideas.rs`

**Evidence:** Design/spec say every painted Ideas row counts. The UI also paints
tag-folder chevron rows. Digits index only idea paths (and `ListDigitAction` only selects
ideas), so digit *n* can skip visual chrome and not match line-count positions under
nested tags.

**Impact:** Contract overclaims visual 1:1 indexing; dogfood with tags can feel “off by
one” relative to counting every line.

**Discussion:** Counting tag rows would need non-idea digit behavior (toggle/no-op).
Idea-only matches the action type and selection product. Prefer rewording over changing
selection semantics.

**Resolution:** Keep idea-only digits. Design and spec describe **selectable idea rows in
paint order** (expanded sections, nested idea paths included; tag chrome not indexed).

**Next:** `/ds-design` — update visible-row model wording; then `/ds-spec` — align
Painted-row index contract and doc.

### 2. Pending-commit archives on the Change digit index

**Where:** `design.md` Change sequence / “archive not indexed”; spec live-queue
definition; `ordered_live_queue` / `painted_live_queue_ids` in
`crates/duckboard/src/area/change.rs`

**Evidence:** The painted Change body appends pending-commit archives after sorted live
WIP. Digits use that sequence. Design text still says only explorations + active changes
and that archive is not indexed; the separate Archived section remains out of the digit
set.

**Impact:** Docs under-describe what `Ctrl+n` can hit when a pending package sits on the
live list; behavior already matches the view.

**Discussion:** Excluding pending archives from digits only would desync hotkeys from the
list. Removing them from paint is another product change. Keep index aligned with paint
and name the pending segment in design/spec.

**Resolution:** Keep pending packages digit-indexed. Design and spec state that the
painted live Change body includes pending-commit archives when shown after live WIP.

**Next:** `/ds-design` — correct Change sequence; then `/ds-spec` — same in the
contract/doc.

### 3. Parallel Ideas ordering walk

**Where:** `design.md` “same ordering builders”; `collect_section_rows` vs
`collect_section_idea_paths` in `crates/duckboard/src/area/ideas.rs`

**Evidence:** Change digits share `ordered_live_queue` with the view. Ideas digits re-walk
sort/expand in a parallel helper instead of a single pure builder both paths call.

**Impact:** Future list-order changes can leave digits wrong while tests still pass
against the duplicate walk.

**Discussion:** Accepting dual walks is smaller short-term. A shared pure “idea paths in
paint order” helper matches the settled design and removes the drift seam.

**Resolution:** Extract a shared pure Ideas paint-order path used by the list view and
digit resolution.

**Next:** `/ds-step` — plan (and later apply) the shared-helper refactor after design/spec
wording is fixed.

## Outcome

Not archive-ready until design and spec name the real paint model and Ideas shares one
ordering walk with the view. Primary next: **`/ds-design`**.
