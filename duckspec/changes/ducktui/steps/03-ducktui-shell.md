# Ducktui shell

Scaffold `crates/ducktui` (ratatui + crossterm + tokio): three screens, two-pane work
layout, focus, overlays, global keys, and status bar.

## Prerequisites

- [x] @step extract-duckcore
- [x] @step session-sharing

## Tasks

- [x] 1. Add `crates/ducktui` binary crate on duckcore, duckpond, and duckchat; workspace
         member and main event loop (`select!` over crossterm, agent, file, tick)

- [x] 2. Project picker, work screen, and settings transitions from bound and unbound
         project state

- [x] 3. Work screen with navigator and chat panes only; Tab toggles pane focus

- [x] 4. Overlay stack (stubs allowed) with input capture; restore pane dispatch on close

- [x] 5. Global key layer (quit, help, screen switches) before pane or overlay content
         handling

- [x] 6. Status bar fields: scope when bound, model, context fill, and turn state

- [x] 7. @spec tui/shell Three screens: No bound project opens the project picker

- [x] 8. @spec tui/shell Three screens: Binding a project opens the work screen

- [x] 9. @spec tui/shell Three screens: Settings returns to the bound or unbound surface

- [x] 10. @spec tui/shell Work screen two panes: Work screen has navigator and chat only

- [x] 11. @spec tui/shell Work screen two panes: Tab toggles pane focus

- [x] 12. @spec tui/shell Overlay input capture: Open overlay consumes pane-destined keys

- [x] 13. @spec tui/shell Overlay input capture: Closing the overlay restores pane dispatch

- [x] 14. @spec tui/shell Global keys before pane dispatch: Help opens from either focused pane

- [x] 15. @spec tui/shell Global keys before pane dispatch: Quit is available without a pane handler

- [x] 16. @spec tui/shell Status bar: Work screen status reports scope, model, context, and turn

- [x] 17. @spec tui/shell Status bar: Project picker status has no scope
