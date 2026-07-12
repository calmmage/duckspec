# Scoped change commit

When the user confirms a post-archive commit, only that change’s work is committed — not
the whole dirty working tree.

## Motivation

Archive handoff proposes a commit message and waits for `` `commit` ``, but does not say
which paths belong to the change. Agents then run a whole-tree commit and pull in
unrelated dirty work (other changes, WIP, noise).

Why now: multi-change dirty trees are normal in this workflow; a “commit change” action
that commits everything is actively unsafe.

## Intent

- On user `` `commit` `` after a successful archive, the agent commits only paths that
  belong to that change

- Unrelated dirty paths stay uncommitted

- The user can see what will be included (message + path set) before the VCS command runs

- Standing VCS guidance matches this rule so mid-lifecycle commits are not free-for-all
  either

- If nothing in the dirty tree belongs to the change, the agent says so and does not
  invent a commit

## Non-goals

- A first-class `/ds-commit` stage or new workflow slash command
- Auto-commit without explicit user confirmation
- Implementing a native duckboard “run commit” button (still agent + VCS CLI)
- Multi-worktree isolation or per-change worktrees
- Push / remote publish workflow
- Replacing project-local commit-message conventions
