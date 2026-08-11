# Review: List digit switch post-followup

Implementation, design, and `shell/list-digit-switch` contracts match after the fidelity
follow-up; audit is clean and the change is archive-ready.

## Scope

Reviewed proposal, design, `shell/list-digit-switch` spec/doc, all three steps,
`reviews/01-review-list-digit-switch-fidelity.md` remedies, `keybind_list_digit`,
`dispatch_list_digit`, Change `ordered_live_queue` / `painted_live_queue_ids`, Ideas
shared paint-order helpers and list walk, and the twelve linked `@spec` tests. `ds check`
and `ds audit list-digit-switch` are clean; focused unit tests pass.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
```

No accepted findings.

## Findings

_(none)_

## Resolved concerns

### Prior review remedies closed

Review 01 required idea-only Ideas indexing wording, pending-commit archives on the Change
live digit sequence, and a shared Ideas paint-order path for list and digits. Design,
spec, and doc now name those sequences; Ideas list and digits share
`sorted_direct_ideas_at`, `child_tag_names_at`, and expand helpers; Change digits continue
through `ordered_live_queue` with the list view.

### Ctrl+digit before modal KeyPress blocks

`KeyPress` resolves Ctrl+1/2/3 before modal routing and returns `Task::none()` when gated
by `navigation_keys_captured`. Spec only requires no selection under modal/rename; open
modals use Ctrl+n/p, not Ctrl+1/2/3. Behavior matches “existing handlers win” for the
chords modals actually own.

### Two thin Ideas walkers

`collect_section_rows` still paints tag chrome while `collect_section_idea_paths` collects
idea paths only. Sort, child-tag, and expand decisions are single-sourced in shared pure
helpers, which satisfies design “call it or derive from it” and closes review 01 finding 3
without forcing the list to consume path-only output.

## Outcome

Archive-ready. Primary next: **`/ds-archive`**.
