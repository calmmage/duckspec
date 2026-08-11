# Worktree stack and merge

Optional stack-on parent scopes, an opt-in gate that requires the base merged before work,
and integrate-to-main that runs automatically when safe (including after archive) and
never claims success on conflict.

## Requirement: Stack base

An exploration or change scope MAY declare another active scope as its stack base. When a
sidecar is created for a scope that has a base, the system SHALL seed that sidecar from
the base scope’s current work-tree revision, not from trunk alone. A base that would form
a cycle SHALL be rejected. Clearing the base SHALL make subsequent sidecar creates follow
ordinary trunk-based placement policy.

> test: code

### Scenario: Stacked sidecar is created from the base scope’s work revision

- **GIVEN** scope A has a work root at a revision that is not trunk-only
- **AND** scope B declares base A and needs a new sidecar
- **WHEN** B’s sidecar is created
- **THEN** B’s sidecar is seeded from A’s work-tree revision
- **AND** not solely from the project trunk tip independent of A

> test: code

### Scenario: Cyclic base is rejected

- **GIVEN** scope A already bases on scope B
- **WHEN** the user sets B’s base to A
- **THEN** the cycle is rejected
- **AND** B’s base is unchanged

> test: code

### Scenario: Clearing base returns create policy to trunk

- **GIVEN** scope B had base A
- **AND** the base is cleared
- **WHEN** a new sidecar is created for B
- **THEN** the sidecar is seeded under ordinary trunk-based policy
- **AND** not from A’s work-tree revision

> test: code

## Requirement: Require base merged

A scope with a stack base MAY enable “require base merged.” While that flag is set and the
base has not been integrated to the project main working tree, the system SHALL block
starting an agent turn for that scope and SHALL indicate the base scope. While the flag is
unset, agent turns SHALL be allowed even if the base is still unmerged. After the base is
integrated to Main, agent turns SHALL be allowed with the flag still set.

> test: code

### Scenario: Blocked send while base unmerged and flag set

- **GIVEN** scope B bases on A with require-base-merged enabled
- **AND** A is not integrated to Main
- **WHEN** the user attempts to start an agent turn on B
- **THEN** the turn does not start
- **AND** the UI indicates that A must be merged first

> test: code

### Scenario: Send allowed when flag unset even if base unmerged

- **GIVEN** scope B bases on A with require-base-merged disabled
- **AND** A is not integrated to Main
- **WHEN** the user starts an agent turn on B
- **THEN** the turn is allowed

> test: code

### Scenario: Send allowed after base integrated to Main

- **GIVEN** scope B bases on A with require-base-merged enabled
- **AND** A has been integrated to Main
- **WHEN** the user starts an agent turn on B
- **THEN** the turn is allowed

> test: code

## Requirement: Integrate to Main

Integrating a scope’s sidecar into the project main working tree SHALL be the same path
for archive success, explicit “merge to main,” and other finish actions that request
integrate. On archive success for a change that still has a sidecar (or unmerged stack
work), the system SHALL automatically attempt integrate. When integrate completes cleanly
(including empty or fast-forward-equivalent cases), the system SHALL forget the sidecar
and clear the scope’s sidecar binding. When integrate conflicts or cannot complete safely,
the system SHALL stop, surface the conflicting paths (or equivalent failure), and SHALL
NOT report success or drop the sidecar as if merged.

> test: code

### Scenario: Archive success auto-attempts integrate for a sidecar scope

- **GIVEN** a change whose work root is a sidecar with unmerged work

- **WHEN** archive of that change succeeds

- **THEN** an integrate-to-Main attempt runs for that scope without a separate user merge
  command

> test: code

### Scenario: Clean integrate forgets the sidecar

- **GIVEN** a scope with a sidecar that integrates cleanly onto Main
- **WHEN** integrate completes successfully
- **THEN** the sidecar is forgotten
- **AND** the scope no longer binds to that sidecar path

> test: code

### Scenario: Conflict stops without claiming success

- **GIVEN** a scope whose sidecar conflicts with Main on integrate
- **WHEN** integrate is attempted
- **THEN** the operation does not report success
- **AND** conflicting paths (or equivalent failure detail) are surfaced
- **AND** the sidecar binding remains

> test: code

### Scenario: Explicit merge to main uses the same integrate path

- **GIVEN** a scope with a sidecar
- **WHEN** the user requests merge to main
- **THEN** the same integrate-to-Main behavior applies as on archive success

> test: code

## Requirement: Stack after parent merge

When a base scope integrates cleanly to Main, a dependent scope that still has a sidecar
MAY be rebased (or equivalently updated) onto Main automatically when that update is
clean. When the update conflicts, the system SHALL stop and surface the failure without
claiming the dependent is on Main.

> test: code

### Scenario: After base merges cleanly, dependent may rebase onto Main when clean

- **GIVEN** scope B bases on A
- **AND** A integrates cleanly to Main
- **AND** updating B onto Main is clean
- **WHEN** the post-base-merge update runs
- **THEN** B’s work root reflects Main plus B’s own work
- **AND** the update is not left claiming success if it would conflict

> test: code
