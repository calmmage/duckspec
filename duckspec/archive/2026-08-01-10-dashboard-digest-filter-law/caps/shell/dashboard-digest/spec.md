# Dashboard digest

Filter-law shortlists, derived attention ranking, and a day-seeded 2+1 cut for the open
project’s Dashboard left column — audit panel and repo artifacts unchanged.

## Requirement: Dashboard section set

When a project is open, the Dashboard left column SHALL show only Changes, Explorations,
and Archived sections (when each has content as defined elsewhere); it SHALL NOT add Ideas
or Stuck sections. The audit panel is outside this capability.

> test: code

### Scenario: Left column sections are Changes Explorations and Archived only

- **GIVEN** an open project with active changes, live explorations, and archived work
- **WHEN** the Dashboard left column is shown
- **THEN** the section headings present are limited to Changes, Explorations, and Archived

> test: code

### Scenario: Ideas and Stuck sections are absent

- **GIVEN** an open project
- **WHEN** the Dashboard left column is shown
- **THEN** there is no Ideas section
- **AND** there is no Stuck section

> test: code

## Requirement: Filter-law shortlist

For Changes and Explorations, the resting view SHALL show at most three ordered rows, with
an “… N more” affordance when more exist, and no overflow chrome when the list length is
at most three. Expanding a section SHALL show the full ordered list; collapsing returns to
the shortlist. “New Exploration” SHALL remain visible under Explorations and SHALL NOT
count toward the three.

> test: code

### Scenario: More than three changes shows three rows and remainder count

- **GIVEN** five active changes
- **AND** the Changes section is at rest
- **WHEN** the Dashboard left column is shown
- **THEN** exactly three change rows are visible
- **AND** an “… 2 more” affordance is shown

> test: code

### Scenario: Three or fewer changes omits overflow chrome

- **GIVEN** two active changes
- **AND** the Changes section is at rest
- **WHEN** the Dashboard left column is shown
- **THEN** both change rows are visible
- **AND** no “… more” affordance is shown

> test: code

### Scenario: Expand and collapse restore full list and shortlist

- **GIVEN** five active changes
- **AND** the Changes section is at rest
- **WHEN** the user expands Changes
- **THEN** all five change rows are visible
- **AND** when the user collapses Changes the resting shortlist of three is restored

> test: code

### Scenario: New Exploration stays outside the shortlist slots

- **GIVEN** five live explorations
- **AND** the Explorations section is at rest
- **WHEN** the Dashboard left column is shown
- **THEN** the New Exploration control is visible
- **AND** exactly three exploration rows occupy the shortlist

> test: code

## Requirement: Archived density on Dashboard

The Dashboard Archived section SHALL start collapsed. When expanded, it SHALL apply the
same three-row shortlist and “… N more” / show-less rules to the newest-first archive
list, without day-seeded rotation.

> test: code

### Scenario: Dashboard Archived starts collapsed

- **GIVEN** a fresh Dashboard with no expand overrides
- **AND** at least one archived row
- **WHEN** the Dashboard is shown
- **THEN** the Archived section is collapsed

> test: code

### Scenario: Expanded Archived shortlists newest first

- **GIVEN** five archived rows in newest-first order
- **AND** the Archived section is expanded and at shortlist rest
- **WHEN** the Dashboard left column is shown
- **THEN** the three newest archived rows are visible
- **AND** an “… 2 more” affordance is shown

> test: code

### Scenario: Archived shortlist does not day-rotate

- **GIVEN** five archived rows

- **AND** the Archived section is expanded and at shortlist rest

- **WHEN** shortlist membership is computed on two different local calendar dates with the
  same archive order

- **THEN** the three visible rows are the three newest in both cases

> test: code

## Requirement: Change ranking

Active changes for the Dashboard SHALL be ordered by attention recency descending (max of
shallow change-tree mtime and latest scoped chat activity), then by needs-work descending
(partial steps and/or validation errors), then by name ascending. Missing mtime and chat
activity SHALL sort as coldest recency.

> test: code

### Scenario: Newer attention ranks above older

- **GIVEN** two active changes whose attention timestamps differ
- **WHEN** the Dashboard change order is computed
- **THEN** the change with the newer attention timestamp appears before the other

> test: code

### Scenario: Needs-work breaks recency ties

- **GIVEN** two active changes with equal attention recency
- **AND** only one has partial steps or validation errors
- **WHEN** the Dashboard change order is computed
- **THEN** the needs-work change appears before the other

> test: code

### Scenario: Missing activity sorts colder than known activity

- **GIVEN** one active change with known attention activity
- **AND** one active change with no mtime and no chat activity
- **WHEN** the Dashboard change order is computed
- **THEN** the change with known activity appears before the change without

> test: code

## Requirement: Exploration ranking

Live explorations on the Dashboard SHALL be ordered by latest scoped chat activity
descending, falling back to the exploration id’s embedded timestamp when activity is
missing. Archived explorations SHALL NOT appear in the Explorations section.

> test: code

### Scenario: Newer chat activity ranks explorations

- **GIVEN** two live explorations with different latest chat activity times
- **WHEN** the Dashboard exploration order is computed
- **THEN** the exploration with newer activity appears before the other

> test: code

### Scenario: Missing chat falls back to id timestamp

- **GIVEN** two live explorations with no chat activity
- **AND** different id-embedded timestamps
- **WHEN** the Dashboard exploration order is computed
- **THEN** they appear in descending id-timestamp order

> test: code

### Scenario: Archived exploration omitted from Explorations

- **GIVEN** an archived exploration
- **WHEN** the Dashboard Explorations section is built
- **THEN** that exploration does not appear as a row

> test: code

## Requirement: Day-seeded shortlist cut

When a Changes or Explorations section is at rest and has more than three ranked items,
the shortlist SHALL be the top two by rank plus one item chosen deterministically from the
remainder using the local calendar date as seed. The same day and same ranked inputs SHALL
yield the same shortlist. Expand SHALL show the full ranked list, not only the three
shortlist members.

> test: code

### Scenario: Resting shortlist is two heat plus one spun tail

- **GIVEN** five ranked active changes
- **AND** the Changes section is at rest
- **WHEN** the shortlist is computed
- **THEN** the first two shortlist members are the top two ranked changes
- **AND** the third member is one of the remaining three

> test: code

### Scenario: Same day and inputs yield the same shortlist

- **GIVEN** a fixed ranked list of more than three changes
- **AND** a fixed local calendar date
- **WHEN** the resting shortlist is computed twice
- **THEN** both results are identical

> test: code

### Scenario: Different days may spin a different tail member

- **GIVEN** a fixed ranked list of more than three changes
- **WHEN** the resting shortlist is computed for two different local calendar dates
- **THEN** each shortlist still starts with the same top two ranked changes
- **AND** the third member may differ between the dates

> test: code

### Scenario: Expand shows full ranked list

- **GIVEN** five ranked active changes
- **AND** the resting shortlist omits at least one change
- **WHEN** the user expands Changes
- **THEN** all five changes appear in full ranked order

> test: code

## Requirement: Expand state lifecycle

Per-section expand flags for Changes, Explorations, and Archived are UI-only. Opening a
different project SHALL reset all three to the collapsed/resting state.

> test: code

### Scenario: Project switch resets expand flags

- **GIVEN** Changes is expanded on the current project
- **WHEN** a different project is opened
- **THEN** the Changes section is at rest (not expanded)

> test: code

## Requirement: Derived-only scores

Ranking and shortlist selection SHALL NOT write heat, scores, or shortlist membership into
`duckspec/` artifacts or idea frontmatter.

> test: code

### Scenario: Ranking leaves duckspec free of heat fields

- **GIVEN** an open project with a `duckspec/` tree
- **WHEN** Dashboard ranking and shortlisting run
- **THEN** no heat or score fields are written under that `duckspec/` tree

> test: code
