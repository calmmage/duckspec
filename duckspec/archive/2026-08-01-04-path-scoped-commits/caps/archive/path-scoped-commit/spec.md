# Archive path-scoped commit

After a successful archive, the handoff proposes an optional commit of only this change's
dirty paths — message and include set first, `` `commit` `` only when nonempty, never the
whole dirty tree and never an invented empty commit.

## Requirement: Change-owned include set

After a successful archive, the handoff SHALL build a path-scoped include set from dirty
working-tree paths that belong to **this** change only: the archive directory just landed,
top-level `caps/` paths this archive applied, any still-dirty paths under the former
`changes/<name>/`, and code or other paths this change actually produced that are still
dirty. It SHALL exclude dirty paths from other changes or unknown WIP. When membership is
ambiguous, it SHALL ask before including a path and SHALL NOT default to the whole dirty
tree.

> test: code

### Scenario: Include set is dirty paths that belong to this change

- **GIVEN** a successful archive of a named change

- **AND** a dirty working tree that mixes paths owned by that change with other dirty
  paths

- **WHEN** the archive handoff builds the include set

- **THEN** the include set contains only dirty paths that belong to that change

- **AND** dirty paths from other work are excluded

### Scenario: Ambiguous membership never defaults to the whole dirty tree

- **GIVEN** a successful archive

- **AND** at least one dirty path whose change membership is ambiguous

- **WHEN** the archive handoff builds the include set

- **THEN** the ambiguous path is not auto-included by treating the whole dirty tree as the
  include set

- **AND** membership is resolved by asking before include

## Requirement: Pre-commit visibility

The handoff SHALL propose a commit message in ordinary markdown (using project conventions
when known) and SHALL show the include set with that message so the user sees what will be
committed before any VCS write. It MAY briefly note excluded dirty paths when useful.

> test: code

### Scenario: Message and include set are shown before any VCS write

- **GIVEN** a successful archive whose include set is nonempty
- **WHEN** the archive handoff is presented
- **THEN** a commit message is proposed in ordinary markdown
- **AND** the include set is shown with that message
- **AND** no VCS commit has run yet

## Requirement: Commit offer and empty set

When the include set is nonempty, the handoff SHALL emit a `next` meta card that includes
the `` `commit` `` send token. When the include set is empty, the handoff SHALL report
that nothing owned by the change is dirty, SHALL NOT invent a commit, and SHALL omit
`` `commit` `` from the `next` meta card (other actions MAY still be offered). The handoff
SHALL NOT auto-commit; it waits for the user to choose `` `commit` `` or another action.

> test: code

### Scenario: Nonempty include set offers commit

- **GIVEN** a successful archive whose include set is nonempty
- **WHEN** the archive handoff is presented
- **THEN** the trailing `next` meta card includes the `` `commit` `` send token

### Scenario: Empty include set reports no owned dirt and does not invent a commit

- **GIVEN** a successful archive whose include set is empty
- **WHEN** the archive handoff is presented
- **THEN** the handoff states that nothing owned by the change is dirty
- **AND** it does not invent a commit
- **AND** the `` `commit` `` send token is omitted from any `next` meta card

## Requirement: Path-scoped execution

On user `` `commit` ``, the agent SHALL run a path-scoped commit for the include set only
(path-limited add/commit under the project's VCS rules) and SHALL report which paths were
committed and what dirty remains. It SHALL NOT use whole-tree commit defaults that would
include unowned dirty paths. Handoff SHALL NEVER auto-commit without the user choosing
`` `commit` ``.

> test: code

### Scenario: On commit, only the include set is committed

- **GIVEN** a presented archive handoff with a nonempty include set and a proposed message
- **WHEN** the user sends `` `commit` ``
- **THEN** only paths in the include set are committed
- **AND** unrelated dirty paths remain uncommitted

### Scenario: Handoff never auto-commits without user commit

- **GIVEN** a successful archive whose include set is nonempty
- **WHEN** the archive handoff is presented and the user has not chosen `` `commit` ``
- **THEN** no VCS commit runs
