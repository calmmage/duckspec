# Archive browse

Duckboard presents archived work newest-first, interleaves archived explorations with
archived changes on Change and Dashboard archived lists, and keeps archived sections
closed by default.

Duckboard presents finished archived work newest-first, interleaves archived explorations
with finished archived changes on Change and Dashboard archived lists, and keeps archived
sections closed by default.

## Requirement: Archived change order

Archived changes SHALL be listed most recent first, using each archive folder's
date-and-counter prefix as the order key.

> test: code

### Scenario: Archived changes list most recent first

- **GIVEN** more than one archived change with distinct archive prefixes
- **WHEN** the archived change list is built
- **THEN** entries appear in descending archive-prefix order

> test: code
> - crates/duckboard/src/data.rs:585

## Requirement: Interleaved archived rows

The Change list and Dashboard Archived lists SHALL include non–idea-owned archived
explorations together with finished archived changes (packages that are not pending under
the archive pending-commit rule), ordered by archive date descending. Pending archived
packages SHALL NOT appear on those Archived lists. Idea-owned archived explorations SHALL
NOT appear on those lists.

### Scenario: Archived non–idea-owned explorations appear with archived changes

- **GIVEN** at least one finished archived change
- **AND** a non–idea-owned archived exploration
- **WHEN** the Change or Dashboard archived list is built
- **THEN** both the change and the exploration appear as rows

> test: code
> - crates/duckboard/src/area/change.rs:4182

### Scenario: Mixed archive rows order by archive date descending

- **GIVEN** archived changes and non–idea-owned archived explorations with distinct
  archive dates

- **WHEN** the archived list is built

- **THEN** all rows appear in descending archive-date order

> test: code
> - crates/duckboard/src/area/change.rs:4196

### Scenario: Idea-owned archived explorations stay off Change and Dashboard archived lists

- **GIVEN** an idea-owned exploration that is archived
- **WHEN** the Change or Dashboard archived list is built
- **THEN** that exploration does not appear as a row

> test: code
> - crates/duckboard/src/area/change.rs:4220

### Scenario: Pending archived package is omitted from Archived lists

- **GIVEN** an archived package that is pending
- **WHEN** the Change or Dashboard archived list is built
- **THEN** that package does not appear as a row

> test: code
> - crates/duckboard/src/area/change.rs:4238

## Requirement: Archived section visibility

The Change list Archived section SHALL be absent only when there are no finished archived
changes and no listable archived explorations. The Ideas Archive section and the Change
Archived section SHALL start collapsed until the user expands them.

### Scenario: Archived section is empty only when both kinds are empty

- **GIVEN** no finished archived changes
- **AND** one non–idea-owned archived exploration
- **WHEN** the Change list is built
- **THEN** the Archived section is present
- **AND** it contains that exploration

> test: code
> - crates/duckboard/src/area/change.rs:4254

### Scenario: Ideas Archive section starts collapsed

- **GIVEN** a fresh Ideas list with no user expand overrides
- **WHEN** the Ideas list is shown
- **THEN** the Archive section is collapsed

> test: code
> - crates/duckboard/src/area/ideas.rs:1518

### Scenario: Change Archived section starts collapsed

- **GIVEN** a fresh Change list with no user expand overrides
- **AND** at least one archived row to show
- **WHEN** the Change list is shown
- **THEN** the Archived section is collapsed

> test: code
> - crates/duckboard/src/area/change.rs:4272
