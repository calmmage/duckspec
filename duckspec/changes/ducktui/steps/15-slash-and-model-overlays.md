# Slash and model overlays

Replace slash and model picker stubs with real catalog-backed UIs.

## Prerequisites

- [x] @step ducktui-shell
- [x] @step chat-pane

## Context

Review finding 5 (partial): slash opens from `/` but the overlay is a stub; model change
is settings-only. Keep `tui/shell` overlay input capture.

## Tasks

- [x] 1. Slash palette: list the shared slash catalog, filter from the composer prefix,
         select inserts or runs per existing slash semantics

- [x] 2. Model picker: list the process catalog grouped by harness; selecting sets the
         active model (and persists default when appropriate)

- [x] 3. Keep overlay input capture and Esc restore (`tui/shell`)

- [x] 4. Open paths already designed (`/` at line start; model overlay entry key or
         help-documented)
