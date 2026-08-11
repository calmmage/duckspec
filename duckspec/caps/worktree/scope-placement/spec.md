# Worktree scope placement

Per-scope Main / Worktree / Auto placement for parallel ideas, gated by the operator’s VCS
workflow, with stable worktree names that match out-of-band VCS tools.

## Requirement: Workflow gate

When the operator’s VCS workflow is plain Git, every exploration and change scope SHALL
use the project’s main working tree only — placement controls SHALL NOT create sidecars.
When the workflow is Jujutsu or Git worktrees, placement modes from this capability SHALL
apply.

> test: code

### Scenario: Plain Git stays on Main

- **GIVEN** the VCS workflow is plain Git
- **AND** an exploration scope with placement Auto
- **AND** the main working tree is dirty under another scope
- **WHEN** that exploration becomes active
- **THEN** its work root is the project main working tree
- **AND** no sidecar worktree is created for it

> test: code
> - crates/duckboard/src/worktree.rs:1042

### Scenario: Jj or Worktrees may use placement

- **GIVEN** the VCS workflow is Jujutsu or Git worktrees
- **AND** an exploration scope with placement Worktree
- **WHEN** that scope becomes active
- **THEN** the scope is allowed to use a dedicated sidecar worktree under this capability

> test: code
> - crates/duckboard/src/worktree.rs:1069

## Requirement: Placement modes

Each exploration or change scope SHALL have a placement of Main, Worktree, or Auto.

- **Main:** the scope’s work root SHALL be the project main working tree; the system SHALL
  NOT create a sidecar for that scope.

- **Worktree:** the scope SHALL have a dedicated sidecar worktree (created if missing).

- **Auto:** the scope SHALL stay on Main while Main is not already in use and dirty by
  another scope; when Main is in use and dirty by another scope, the system SHALL place
  this scope on a sidecar instead.

A scope pinned to Main SHALL remain on Main even when Auto would fork. A scope pinned to
Worktree SHALL use a sidecar even when Main is clean.

> test: code

### Scenario: Main never creates a sidecar

- **GIVEN** the VCS workflow allows placement
- **AND** a scope with placement Main
- **AND** the main working tree is dirty under another scope
- **WHEN** the Main-pinned scope becomes active
- **THEN** its work root is the project main working tree
- **AND** no sidecar is created for it

> test: code
> - crates/duckboard/src/worktree.rs:1099

### Scenario: Worktree always has a sidecar

- **GIVEN** the VCS workflow allows placement
- **AND** a scope with placement Worktree and no sidecar yet
- **AND** the main working tree is clean
- **WHEN** that scope becomes active
- **THEN** a sidecar worktree exists for that scope
- **AND** the scope’s work root is that sidecar

> test: code
> - crates/duckboard/src/worktree.rs:1116

### Scenario: Auto stays on Main when Main is free

- **GIVEN** the VCS workflow allows placement
- **AND** a scope with placement Auto
- **AND** no other scope is using a dirty main working tree
- **WHEN** the Auto scope becomes active
- **THEN** its work root is the project main working tree
- **AND** no sidecar is created for it

> test: code
> - crates/duckboard/src/worktree.rs:1140

### Scenario: Auto forks when Main is in use and dirty

- **GIVEN** the VCS workflow allows placement
- **AND** scope A is bound to a dirty main working tree
- **AND** scope B has placement Auto and no sidecar yet
- **WHEN** scope B becomes active
- **THEN** a sidecar worktree exists for scope B
- **AND** scope B’s work root is that sidecar
- **AND** scope A remains on main

> test: code
> - crates/duckboard/src/worktree.rs:1154

### Scenario: Main pin wins over Auto fork pressure

- **GIVEN** the VCS workflow allows placement
- **AND** scope A is bound to a dirty main working tree
- **AND** scope B has placement Main
- **WHEN** scope B becomes active
- **THEN** scope B’s work root is the project main working tree
- **AND** no sidecar is created for scope B

> test: code
> - crates/duckboard/src/worktree.rs:1179

## Requirement: Stable naming

A sidecar worktree’s durable identity SHALL be `duck-<scope_key>`, where `scope_key` is
the exploration id or change folder name. The on-disk path, jj workspace name, and git
worktree branch SHALL derive from that identity so external `jj workspace list` /
`git worktree list` match duckboard. Changing an exploration’s display title SHALL NOT
change the worktree identity.

> test: code

### Scenario: Sidecar identity is duck-scope_key

- **GIVEN** the VCS workflow allows placement
- **AND** a scope whose key is `session-worktrees`
- **AND** placement Worktree
- **WHEN** its sidecar is created
- **THEN** the worktree identity is `duck-session-worktrees`
- **AND** the sidecar path is under `.duckboard/worktrees/duck-session-worktrees`

> test: code
> - crates/duckboard/src/worktree.rs:1197

### Scenario: Display rename does not rename the worktree id

- **GIVEN** an exploration with scope key `exp-123` and an existing sidecar `duck-exp-123`
- **WHEN** the exploration’s display name is renamed
- **THEN** the worktree identity remains `duck-exp-123`

> test: code
> - crates/duckboard/src/worktree.rs:1224

## Requirement: Placement persists

A scope’s placement SHALL be persisted outside the project working tree and reloaded with
the project. When an exploration is promoted into a change, the change SHALL inherit that
exploration’s placement.

> test: code

### Scenario: Placement survives project reload

- **GIVEN** a scope whose placement is Worktree
- **WHEN** the project is closed and reopened
- **THEN** that scope’s placement is still Worktree

> test: code
> - crates/duckboard/src/worktree.rs:1238

### Scenario: Promotion keeps placement on the change

- **GIVEN** an exploration with placement Worktree
- **WHEN** that exploration is promoted into a change
- **THEN** the change’s placement is Worktree

> test: code
> - crates/duckboard/src/worktree.rs:1256
