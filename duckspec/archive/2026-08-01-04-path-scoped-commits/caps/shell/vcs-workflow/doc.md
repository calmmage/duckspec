# VCS workflow

A global operator choice of how agents treat version control — plain git, jujutsu, or git
worktrees — with standing instructions injected on every new session's first turn.

## Workflows

```
| Workflow   | Agent tool surface              | Default |
| ---------- | ------------------------------- | ------- |
| Git        | plain `git` only                | yes     |
| Jj         | `jj` only (no bare `git`)       | no      |
| Worktrees  | `git` + `git worktree` guidance | no      |
```

Selection lives in Settings under Version control and applies to all projects. It is
stored in application config (`[vcs].workflow`), not per project.

## Standing instructions

Each workflow supplies a fixed instruction block the agent is expected to follow:

- use the matching VCS tool (and not the competing one where that applies)

- never auto-commit — show the message and wait for confirmation

- for duckspec change work (including post-archive `` `commit` ``): commit **only** paths
  that belong to that change; leave unrelated dirty paths out; if nothing owned is dirty,
  report it and do not invent a commit

- worktrees: prefer one worktree per parallel change (session plumbing may still be manual
  depending on product cut)

Product priming text may also warn about destructive VCS commands; that is not part of
this capability's contract.

Archive handoff path-scoping is specified under `archive/path-scoped-commit`; standing
text repeats the same change-only rule so mid-lifecycle commits match.

## First-turn priming

```
AGENTS.md (when present)
      +
scope orientation (when present)
      +
VCS standing instructions  ◄── this capability
      +
path-reference note
      │
      ▼
first-turn priming body → agent waits with "."
```

Only the first turn of a session is primed; later turns do not re-inject this body.
