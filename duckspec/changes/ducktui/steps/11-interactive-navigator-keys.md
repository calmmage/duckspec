# Interactive navigator keys

Wire work-screen navigator focus so the user can move the highlight and activate a row.

## Prerequisites

- [x] @step navigator
- [x] @step ducktui-shell

## Context

Review finding 1 (`02-review-ducktui-interactive-and-rendering-gaps`): selection APIs and
unit tests exist, but the live event loop never moves or activates navigator rows.

## Tasks

- [x] 1. Add move-selection among `visible_rows` (j/k or ↑↓) while the navigator is
         focused

- [x] 2. Enter activates the current row via `select` + `apply_nav_select` (change,
         exploration, Ideas, Codex, Settings, archived)

- [x] 3. Optionally seed a default selection when the tree first appears

- [x] 4. Dispatch navigator keys from the live event loop (not only unit-tested `select`)
