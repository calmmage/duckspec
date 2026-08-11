# Project picker and settings

Implement usable project binding and shared-config editing as designed.

## Prerequisites

- [x] @step ducktui-shell

## Context

Review finding 4: project picker and settings screens exist for transitions, but path
entry, recents, and config writes are not implemented. Theme stays app-local.

## Tasks

- [x] 1. Project picker: path entry and recent projects from shared config; confirming a
         path binds the project and opens the work screen

- [x] 2. Promote bound path in shared recents (same config file as duckboard)

- [x] 3. Settings: edit harness/model/oneshot (and other designed shared prefs) and write
         the shared config file

- [x] 4. Theme remains terminal-local; leaving settings returns to bound work screen or
         unbound picker (existing shell scenarios)
