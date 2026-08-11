# @ Archive browse

Duckboard presents finished archived work newest-first, interleaves archived explorations
with finished archived changes on Change and Dashboard archived lists, and keeps archived
sections closed by default.

## @ Requirement: Interleaved archived rows

The Change list and Dashboard Archived lists SHALL include non–idea-owned archived
explorations together with finished archived changes (packages that are not pending under
the archive pending-commit rule), ordered by archive date descending. Pending archived
packages SHALL NOT appear on those Archived lists. Idea-owned archived explorations SHALL
NOT appear on those lists.

### ~ Scenario: Archived non–idea-owned explorations appear with archived changes

- **GIVEN** at least one finished archived change
- **AND** a non–idea-owned archived exploration
- **WHEN** the Change or Dashboard archived list is built
- **THEN** both the change and the exploration appear as rows

> test: code

### + Scenario: Pending archived package is omitted from Archived lists

- **GIVEN** an archived package that is pending
- **WHEN** the Change or Dashboard archived list is built
- **THEN** that package does not appear as a row

> test: code

## @ Requirement: Archived section visibility

The Change list Archived section SHALL be absent only when there are no finished archived
changes and no listable archived explorations. The Ideas Archive section and the Change
Archived section SHALL start collapsed until the user expands them.

### ~ Scenario: Archived section is empty only when both kinds are empty

- **GIVEN** no finished archived changes
- **AND** one non–idea-owned archived exploration
- **WHEN** the Change list is built
- **THEN** the Archived section is present
- **AND** it contains that exploration

> test: code
