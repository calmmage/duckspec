# Navigator

Build the scope tree from duckpond and session counts; bind chat scope on selection; show
session badges; route Settings through the shell.

## Prerequisites

- [x] @step ducktui-shell

## Tasks

- [x] 1. Build CHANGES, EXPLORATIONS, collapsed ARCHIVED, and bottom Ideas/Codex/Settings
         entries from existing project state only

- [x] 2. Wire selection to shared `Scope` for change and exploration; Settings opens the
         settings screen (Ideas/Codex selection retargeted in step
         `ideas-and-codex-selection`)

- [x] 3. Session-count badges from the shared session store; omit the badge when the count
         is zero

- [x] 4. @spec tui/navigator Tree from existing state: Active changes appear under CHANGES with phase indicators

- [x] 5. @spec tui/navigator Tree from existing state: Explorations appear under EXPLORATIONS

- [x] 6. @spec tui/navigator Tree from existing state: ARCHIVED is collapsed by default

- [x] 7. @spec tui/navigator Tree from existing state: Ideas, Codex, and Settings appear as bottom entries

- [x] 8. @spec tui/navigator Selection binds chat scope: Selecting a change binds chat to that change scope

- [x] 9. @spec tui/navigator Selection binds chat scope: Selecting an exploration binds chat to that exploration scope

- [x] 10. @spec tui/navigator Selection binds chat scope: Selecting Settings opens the settings screen

- [x] 11. @spec tui/navigator Session count badges: Scope with sessions shows the count badge

- [x] 12. @spec tui/navigator Session count badges: Scope with zero sessions shows no badge
