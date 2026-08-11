# First-tag mint

Mint and link an idea on first tag (or first mark cycle) for free explorations and
unlinked changes; keep linked explorations on the CHANGE list.

## Prerequisites

- [x] @step mark-inheritance

## Tasks

- [x] 1. Implement `mint_linked_idea` for exploration and change targets (title, state,
         link fields, first tag)

- [x] 2. Prettify change folder slug for mint titles (kebab → spaces, light title-case)

- [x] 3. Wire first successful tag on an unlinked CHANGE-row target through mint + tag
         apply; skip mint when already linked

- [x] 4. Stop filtering idea-linked explorations out of the CHANGE list
         (`idea_path.is_none()` filter)

- [x] 5. Wire mark-cycle mint for unlinked rows; chat message still does not mint

- [x] 6. @spec ideas/first-tag-mint First tag on free exploration mints a linked idea: First tag creates exploration-state idea with display name title

- [x] 7. @spec ideas/first-tag-mint First tag on free exploration mints a linked idea: Exploration record points at the new idea

- [x] 8. @spec ideas/first-tag-mint First tag on free exploration mints a linked idea: Linked exploration remains on the CHANGE list

- [x] 9. @spec ideas/first-tag-mint First tag on unlinked change mints a linked idea: First tag creates change-state idea with prettified-slug title

- [x] 10. @spec ideas/first-tag-mint First tag on unlinked change mints a linked idea: Idea links to the change name

- [x] 11. @spec ideas/first-tag-mint First tag on unlinked change mints a linked idea: Second tag on already-linked change does not mint another idea

- [x] 12. @spec ideas/first-tag-mint Chat message does not create an idea: First chat message alone does not mint an idea

- [x] 13. @spec ideas/first-tag-mint First mark cycle mints when unlinked: Mark cycle on free exploration creates a linked idea

## Outcomes

- Mark click on CHANGE creates the idea when missing (`cycle_mark_for_target` in main).
- First-tag mint API remains for when CHANGE tag UI is added (option A).
