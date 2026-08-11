# Worktree manager and placement policy

CLI manager, app-data bindings, stable naming, and Main/Worktree/Auto resolve under the
VCS workflow gate — without placement UI chrome.

## Tasks

- [x] 1. Add `crates/duckboard/src/worktree.rs`: ensure/forget sidecar (jj workspace / git
         worktree by workflow), dirty check via `vcs::changed_files`, ignore
         `.duckboard/worktrees/`

- [x] 2. Persist scope bindings + placement by project hash in app data; load on project
         open; reconcile orphan worktrees

- [x] 3. Resolve placement: plain Git → always Main; Jj/Worktrees → Main / Worktree / Auto
         policy (Main pin wins; Worktree always sidecars; Auto forks when main in use and
         dirty)

- [x] 4. @spec worktree/scope-placement Workflow gate: Plain Git stays on Main

- [x] 5. @spec worktree/scope-placement Workflow gate: Jj or Worktrees may use placement

- [x] 6. @spec worktree/scope-placement Placement modes: Main never creates a sidecar

- [x] 7. @spec worktree/scope-placement Placement modes: Worktree always has a sidecar

- [x] 8. @spec worktree/scope-placement Placement modes: Auto stays on Main when Main is free

- [x] 9. @spec worktree/scope-placement Placement modes: Auto forks when Main is in use and dirty

- [x] 10. @spec worktree/scope-placement Placement modes: Main pin wins over Auto fork pressure

- [x] 11. @spec worktree/scope-placement Stable naming: Sidecar identity is duck-scope_key

- [x] 12. @spec worktree/scope-placement Stable naming: Display rename does not rename the worktree id

- [x] 13. @spec worktree/scope-placement Placement persists: Placement survives project reload
