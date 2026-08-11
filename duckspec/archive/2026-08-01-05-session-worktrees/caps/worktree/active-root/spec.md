# Worktree active root

The focused scope’s work root is the single authority for Changed files, diffs, file
opens, and agent working directory while that scope is active.

## Requirement: Focused root authority

While an exploration or change scope is focused, the system SHALL report Changed files and
diffs from that scope’s work root only (not a union of all scopes). Agent turns for
sessions under that scope SHALL use that work root as the working directory. Opening a
project-relative file path SHALL resolve against that work root.

> test: code

### Scenario: Changed files reflect the focused scope’s root only

- **GIVEN** scope A’s work root is dirty with file `a.rs`
- **AND** scope B’s work root is dirty with file `b.rs`
- **AND** scope A is focused
- **WHEN** Changed files is shown
- **THEN** it includes `a.rs`
- **AND** it does not include `b.rs` solely because B is dirty on another root

> test: code

### Scenario: Agent working directory matches the focused scope’s root

- **GIVEN** a focused scope whose work root is a sidecar path
- **WHEN** an agent turn runs for a session under that scope
- **THEN** the agent’s working directory is that sidecar path

> test: code

### Scenario: File open resolves under the focused root

- **GIVEN** a focused scope whose work root contains `src/lib.rs`
- **WHEN** the user opens project-relative path `src/lib.rs`
- **THEN** the opened content is read from that scope’s work root

> test: code

## Requirement: Focus switch

When focus moves from one exploration or change scope to another, Changed files SHALL
refresh from the newly focused scope’s work root. A non-streaming agent session under the
newly focused scope SHALL use that scope’s work root on its next turn (rebinding the
runtime if the previous root differed).

> test: code

### Scenario: Switching scope rebinds Changed files to the new root

- **GIVEN** scope A is focused and Changed files shows A’s dirty set
- **AND** scope B has a different work root with a different dirty set
- **WHEN** focus switches to scope B
- **THEN** Changed files shows B’s dirty set

> test: code

### Scenario: Cold agent rebinds to the new root on next use

- **GIVEN** a session whose last runtime used work root R1
- **AND** its scope’s work root is now R2
- **AND** the session is not streaming
- **WHEN** the next agent turn starts for that session
- **THEN** the agent’s working directory is R2

> test: code

## Requirement: No mid-stream rebind

While a session’s agent turn is streaming, the system SHALL NOT change that session’s
agent working directory for the in-flight turn. Root changes for that session SHALL apply
only after the turn ends (or is cancelled) and a later cold turn starts.

> test: code

### Scenario: Streaming session keeps its root until the turn ends

- **GIVEN** a session streaming an agent turn with working directory R1
- **AND** its scope’s work root becomes R2 during the turn
- **WHEN** the turn is still in flight
- **THEN** the in-flight agent continues with working directory R1

> test: code

## Requirement: Caps and codex on Main

Sessions scoped to the capability tree (caps) or codex SHALL always use the project main
working tree as their work root, regardless of any exploration or change placement.

> test: code

### Scenario: Caps and codex scopes always use the project main root

- **GIVEN** an exploration with a sidecar work root
- **WHEN** a caps or codex session runs an agent turn
- **THEN** that session’s working directory is the project main working tree

> test: code
