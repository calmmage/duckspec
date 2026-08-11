# Settings exposes three VCS workflows

Pin that Settings offers Git / Jj / Worktrees for selection (cheap pure check, not a full
GUI test).

## Context

Review finding 2: Global workflow choice requires Settings to expose the three workflows,
but only config default/persist were tested. Prefer asserting the picker source is
`VcsWorkflow::ALL` (or equivalent pure surface) rather than iced UI automation.

## Tasks

- [x] 1. Assert Settings version-control choices are exactly `VcsWorkflow::ALL` (plain
         git, jujutsu, git worktrees) — e.g. unit test on the picker source or a small
         pure helper Settings uses

- [x] 2. @spec shell/vcs-workflow Global workflow choice: Settings exposes the three workflows for selection
