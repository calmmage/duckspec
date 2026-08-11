# Attention signal gathering

Gather shallow change mtimes on project load/refresh and latest chat activity per scope
for ranking inputs.

## Prerequisites

- [x] @step pure-rank-and-shortlist-helpers

## Tasks

- [x] 1. On `ProjectData` load/watcher refresh, record shallow mtime per active change
         (dir + immediate children)

- [x] 2. Compute latest chat activity per scope (max created / last message activity) for
         change names and exploration ids

- [x] 3. Wire signal maps into the rank helper call site used by Dashboard (may stay
         stubbed until the next step)
