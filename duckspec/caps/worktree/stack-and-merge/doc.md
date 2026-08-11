# Worktree stack and merge

How ideas that depend on each other share trees, and how sidecar work returns to the
project main working tree.

## Stack base

```
  Scope A (feature 1)          work root R_A
       │
       │  base = A
       ▼
  Scope B (feature 2)          sidecar seeded from R_A
```

Default: B may explore and run agents on A’s line of work without waiting for A to merge.
Optional **require base merged**: first agent turn on B is blocked until A is on Main.

Cycles (A→B→A) are rejected. Clearing base returns new sidecar creates to ordinary trunk
policy.

## Integrate to Main

One integrate path, several triggers:

```
| Trigger | Behavior |
| --- | --- |
| Archive success | Always attempt integrate if the change still has a sidecar / unmerged stack work |
| Explicit “merge to main” | Same integrate path |
| Clean result | Forget sidecar, clear binding |
| Conflict / unsafe | Stop; show paths; keep binding; never silent success |
```

Stacked order: when B bases on A, finishing A first (or rebasing B after A lands) keeps
the stack coherent. After A merges cleanly, B may auto-update onto Main when that update
is clean; otherwise the same conflict surfacing applies.

## Relation to placement

Placement (Main / Worktree / Auto) decides *whether* a scope has a sidecar. This
capability decides *how sidecars relate* (stack) and *how they finish* (integrate). Scopes
pinned to Main never take the integrate-sidecar path.
