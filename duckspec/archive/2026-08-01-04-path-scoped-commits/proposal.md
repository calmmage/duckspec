# Path-scoped commits

When the user confirms a post-archive (or change-scoped) commit, only that change’s work
is committed — not the whole dirty working tree — and agents get standing VCS rules for
how to do it.

## Motivation

Archive handoff used to propose a commit message and wait for `` `commit` `` without
saying which paths belong to the change. Agents then ran whole-tree commits and swept
unrelated dirty work (other changes, WIP, noise). Multi-change dirty trees are normal;
that is actively unsafe.

A second gap: agents also need a stable global choice of VCS surface (plain git, jj, or
worktree guidance) so path-scoped commits use the right tool and never invent a whole-tree
default.

## Intent

- On user `` `commit` `` after a successful archive, the agent commits only paths that
  belong to that change

- Unrelated dirty paths stay uncommitted

- The user sees message + path set before the VCS command runs

- Standing VCS guidance (first-turn priming + Settings) states the same path-scoped rule
  for mid-lifecycle commits

- If nothing dirty belongs to the change, the agent says so and does not invent a commit

- Operator can pick git / jj / worktrees globally; that choice shapes standing
  instructions

## Non-goals

- A first-class `/ds-commit` stage or new workflow slash command
- Auto-commit without explicit user confirmation
- A native duckboard “run commit” button
- Multi-worktree isolation or per-change worktrees plumbing
- Push / remote publish workflow
- Replacing project-local commit-message conventions
- Full custom initial-message composition UI (section toggles / full override)
