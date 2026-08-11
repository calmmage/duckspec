# Worktree active root

What “active” means once a scope has a work root: one focused tree for the human UI and
for agents under that scope.

## Authority

The focused exploration or change owns:

```
| Surface | Bound to focused work root |
| --- | --- |
| Changed files | Yes — that root only |
| Diffs | Yes |
| Project-relative file open | Yes |
| Agent cwd (sessions under the scope) | Yes |
| Caps / codex sessions | Always project main |
```

There is still a single Changed files list in the UI; it is not a merge of every scope’s
dirty set. Switching the CHANGE selection refreshes that list from the new root.

## Switching and streaming

```
focus scope B
    │
    ├─► refresh Changed files from B.root
    ├─► resolve file opens under B.root
    └─► cold sessions: next turn uses B.root
            │
            streaming turn on old root?
            └─► leave in-flight cwd alone until turn ends
```

Mid-turn root swaps are unsafe (resume ids, tools mid-edit). Cold rebind is enough.

## Background agents

Other scopes may keep running on their own roots. Watchers cover main plus live sidecars
so background dirty state can update; the human-facing Changed files strip still shows
only the focused root.
