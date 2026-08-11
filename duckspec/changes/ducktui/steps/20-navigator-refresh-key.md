# Navigator refresh key

Add an explicit key that rebuilds the navigator tree from project state and document it in
help.

## Prerequisites

- [x] @step interactive-navigator-keys
- [x] @step ducktui-shell

## Context

Review finding 4: the tree is built on project bind (and quick-idea save) only; the
watcher reloads the active session but does not rebuild navigator rows or badges. Full
live auto-refresh was deferred; a manual refresh key is the accepted fix.

## Tasks

- [x] 1. Add a refresh helper that rebuilds `Navigator` from the bound project root and
         preserves selection when the selected id is still valid

- [x] 2. Bind a work-screen key (e.g. `r` or `Ctrl+r` when navigator-focused / documented)
         and add a matching line to the help overlay

- [x] 3. Cover with a unit test: after a tree-affecting on-disk change, refresh updates
         visible rows or session badges without requiring project rebind
