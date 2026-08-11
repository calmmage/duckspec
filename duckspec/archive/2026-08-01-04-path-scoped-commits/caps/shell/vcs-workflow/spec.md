# VCS workflow

A global operator choice of how agents treat version control — plain git, jujutsu, or git
worktrees — with standing instructions (including path-scoped change commits) injected
into every new session's first-turn priming.

## Requirement: Global workflow choice

Duckboard SHALL offer three global VCS workflows: plain git, jujutsu (jj), and git
worktrees. The default workflow SHALL be plain git. The operator's selection SHALL persist
in application config (all projects) and drive standing instructions for new sessions.
Settings SHALL expose the three workflows for selection.

> test: code

### Scenario: Default workflow is plain git

- **GIVEN** a fresh configuration with no VCS workflow set
- **WHEN** the VCS workflow is read
- **THEN** the workflow is plain git

### Scenario: Selected workflow persists in config

- **GIVEN** an operator-selected workflow of jujutsu or git worktrees
- **WHEN** configuration is saved and reloaded
- **THEN** the same workflow is restored

### Scenario: Settings exposes the three workflows for selection

- **GIVEN** the global Settings surface
- **WHEN** the version-control workflow choices are presented
- **THEN** plain git, jujutsu, and git worktrees are all offered for selection

> test: code

## Requirement: Standing instructions

Each workflow SHALL provide standing instructions that name the VCS tool the agent must
use (and which competing tool to avoid where applicable), require confirmation before any
commit, require path-scoped commits for duckspec change work (including post-archive
`` `commit` ``) so the entire dirty tree is never the default, and forbid inventing a
commit when nothing dirty belongs to the change. The worktrees workflow SHALL additionally
prefer a dedicated git worktree per parallel change.

> test: code

### Scenario: Each workflow names its VCS tool

- **GIVEN** each of the three workflows
- **WHEN** standing instructions for that workflow are produced
- **THEN** plain git names `git` and not `jj` as the tool to use
- **AND** jujutsu names `jj` and forbids bare `git`
- **AND** worktrees names git worktrees and not `jj`

### Scenario: Each workflow requires path-scoped change-only commits

- **GIVEN** each of the three workflows
- **WHEN** standing instructions for that workflow are produced
- **THEN** the text requires committing only paths that belong to the change
- **AND** it forbids defaulting to the entire dirty tree

### Scenario: Each workflow forbids auto-commit and inventing an empty commit

- **GIVEN** each of the three workflows

- **WHEN** standing instructions for that workflow are produced

- **THEN** the text forbids committing without explicit user confirmation

- **AND** it requires reporting and not inventing a commit when nothing dirty belongs to
  the change

### Scenario: Worktrees workflow prefers a dedicated worktree per parallel change

- **GIVEN** the git worktrees workflow
- **WHEN** standing instructions for that workflow are produced
- **THEN** the text prefers a dedicated worktree for each change when working in parallel

## Requirement: First-turn priming injection

The first-turn priming body for a new session SHALL include the standing instructions for
the currently selected global VCS workflow. Those instructions SHALL ride the same priming
delivery as project conventions and scope orientation when those are present.

> test: code

### Scenario: First-turn priming body includes the selected workflow's standing instructions

- **GIVEN** a selected VCS workflow with non-empty standing instructions
- **WHEN** the first-turn priming body is assembled
- **THEN** the body contains that workflow's standing instructions
