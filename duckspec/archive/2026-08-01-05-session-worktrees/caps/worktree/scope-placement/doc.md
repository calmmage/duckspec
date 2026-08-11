# Worktree scope placement

How duckboard chooses whether a change or exploration works on the project main tree or on
a dedicated sidecar, and how those sidecars are named for tools outside the app.

## Modes

```
| Placement | Work root | When to use |
| --- | --- | --- |
| **Main** | Project root only | Small fixes; accept sharing the main dirty tree |
| **Worktree** | Always a sidecar | Parallel ideas that must not interleave |
| **Auto** | Main until Main is busy+dirty for another scope, then sidecar | Default under Jj / Git worktrees |
```

Placement is a per-scope setting (CHANGE row / change header). Pinning Main or Worktree
overrides Auto’s fork policy.

## Workflow gate

```
| Settings workflow | Placement active? |
| --- | --- |
| Plain Git | No — everything stays on Main |
| Jujutsu (jj) | Yes — sidecars via jj workspaces |
| Git worktrees | Yes — sidecars via git worktrees |
```

## Naming

Stable identity (not the display title):

```
duck-<scope_key>
```

- Path: `<project>/.duckboard/worktrees/duck-<scope_key>/`
- jj workspace name: `duck-<scope_key>`
- git branch: `duck/<scope_key>`

`scope_key` is the exploration id or change folder name. Renaming a list label does not
rename the worktree.

## Persistence and promotion

Bindings and placement live in duckboard app data (not in the repo). Exploration → change
promotion carries placement with the chat remount so the idea keeps the same Main vs
Worktree choice.
