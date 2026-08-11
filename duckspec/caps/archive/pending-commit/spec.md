# Archive pending commit

Duckboard keeps duckspec-archived packages visible in the Change list with uncommitted
chrome until residual dirt under that archive folder is gone.

## Requirement: Pending predicate

An archived package SHALL be pending when any working-tree dirty path lies under its
archive directory (the folder path itself or a descendant path), including uncommitted
deletes. An archived package with no dirty path under that directory SHALL NOT be pending.

> test: code

### Scenario: Dirty path under the archive folder is pending

- **GIVEN** an archived package whose folder name is a known archive id
- **AND** the working tree has a dirty path under `duckspec/archive/<id>/`
- **WHEN** pending is evaluated for that package
- **THEN** the package is pending

> test: code
> - crates/duckboard/src/area/change.rs:4126

### Scenario: Dirt only outside the archive folder is not pending

- **GIVEN** an archived package whose folder name is a known archive id
- **AND** the working tree is dirty only on paths outside `duckspec/archive/<id>/`
- **WHEN** pending is evaluated for that package
- **THEN** the package is not pending

> test: code
> - crates/duckboard/src/area/change.rs:4147

## Requirement: Change list placement

While an archived package is pending, the Change list SHALL include it as a row in the
Change section after active changes, ordered among other pending archives newest-first by
archive folder name. A pending package SHALL NOT be presented only as a finished Archived
row while it remains pending.

> test: code

### Scenario: Pending package appears in the Change section

- **GIVEN** an archived package that is pending
- **AND** at least one active change
- **WHEN** the Change list is built
- **THEN** the pending package appears as a row in the Change section
- **AND** it appears after the active changes

> test: code
> - crates/duckboard/src/area/change.rs:4330

### Scenario: Multiple pending archives order newest-first after actives

- **GIVEN** two pending archived packages with distinct archive folder prefixes
- **AND** at least one active change
- **WHEN** the Change list is built
- **THEN** both pending packages appear after the active changes
- **AND** among the pending packages the newer archive prefix appears first

> test: code
> - crates/duckboard/src/area/change.rs:4358

## Requirement: Uncommitted chrome and Commit send

A pending package row in the Change section SHALL show trailing uncommitted chrome. When
that chrome is activated, the UI SHALL select that change if it is not already selected
and SHALL send the text `Commit` into that change's chat. Activating the chrome SHALL NOT
itself perform a VCS commit.

> test: code

### Scenario: Pending row shows uncommitted chrome

- **GIVEN** a pending archived package shown in the Change section
- **WHEN** the row is rendered
- **THEN** trailing uncommitted chrome is present

> test: code
> - crates/duckboard/src/area/change.rs:4384

### Scenario: Activating chrome sends Commit without committing

- **GIVEN** a pending archived package shown in the Change section
- **AND** that change is not selected
- **WHEN** the uncommitted chrome is activated
- **THEN** that change becomes selected
- **AND** the text `Commit` is submitted into that change's chat
- **AND** no VCS commit has run as a result of the activation

> test: code
> - crates/duckboard/src/area/change.rs:4395
