# Mark inheritance

Resolve idea links for change and exploration rows and project mark (including unlinked
rows until mark-cycle mint).

## Prerequisites

- [x] @step idea-mark-model

## Tasks

- [x] 1. Add `idea_for_exploration` / `idea_for_change` (or equivalent link index) over
         loaded ideas

- [x] 2. Project a row mark from the linked idea (or none when unlinked) without requiring
         full list UI

- [x] 3. Implement mark-cycle action that saves when a linked idea exists (mint when
         unlinked is wired via `cycle_mark_for_target` in the mark-mint path)

- [x] 4. @spec ideas/marks Linked rows inherit the idea mark: Change-linked row exposes the idea's mark

- [x] 5. @spec ideas/marks Linked rows inherit the idea mark: Exploration-linked row exposes the idea's mark

- [x] 6. @spec ideas/marks Linked rows inherit the idea mark: Unlinked row has no mark until mark cycle mints
