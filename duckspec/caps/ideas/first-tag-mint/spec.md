# First-tag idea mint

On the first tag for a free exploration or an unlinked change, mint and link an idea so
annotations have a home — without minting from chat messages or mark cycles alone.

## Requirement: First tag on free exploration mints a linked idea

When a tag is successfully applied to a free exploration that has no linked idea, the
system SHALL create an exploration-state idea whose title is that exploration's display
name, SHALL record the exploration's identity on the idea, SHALL include that first tag on
the idea, and SHALL record the idea as the exploration's linked idea. After that link, the
exploration SHALL still appear as a row on the CHANGE list.

> test: code

### Scenario: First tag creates exploration-state idea with display name title

- **GIVEN** a free exploration with display name `Cloud agent options` and no linked idea
- **WHEN** the first tag `ui` is applied to that exploration
- **THEN** an exploration-state idea exists whose title is `Cloud agent options`
- **AND** the idea's tags include `ui`

> test: code
> - crates/duckboard/src/idea_store.rs:1224

### Scenario: Exploration record points at the new idea

- **GIVEN** a free exploration with no linked idea
- **WHEN** the first tag is applied to that exploration
- **THEN** the exploration is linked to the newly created idea

> test: code
> - crates/duckboard/src/idea_store.rs:1225

### Scenario: Linked exploration remains on the CHANGE list

- **GIVEN** a free exploration that is visible on the CHANGE list
- **WHEN** the first tag is applied and the exploration becomes idea-linked
- **THEN** that exploration still appears as a row on the CHANGE list

> test: code
> - crates/duckboard/src/idea_store.rs:1226

## Requirement: First tag on unlinked change mints a linked idea

When a tag is successfully applied to an active change that has no linked idea, the system
SHALL create a change-state idea whose title is a prettified form of the change folder
name (kebab-case segments as spaced words with light title-case), SHALL record the change
name on the idea, and SHALL include that first tag on the idea. A later tag on a change
that already has a linked idea SHALL NOT create another idea for that change.

> test: code

### Scenario: First tag creates change-state idea with prettified-slug title

- **GIVEN** an active change named `list-marks-tags-sort` with no linked idea

- **WHEN** the first tag `queue` is applied to that change

- **THEN** a change-state idea exists whose title is the prettified form of
  `list-marks-tags-sort`

- **AND** the idea's tags include `queue`

> test: code
> - crates/duckboard/src/idea_store.rs:1257

### Scenario: Idea links to the change name

- **GIVEN** an active change named `list-marks-tags-sort` with no linked idea
- **WHEN** the first tag is applied to that change
- **THEN** the new idea is linked to the change name `list-marks-tags-sort`

> test: code
> - crates/duckboard/src/idea_store.rs:1258

### Scenario: Second tag on already-linked change does not mint another idea

- **GIVEN** an active change already linked to one idea
- **WHEN** another tag is applied to that change
- **THEN** no additional idea is created for that change

> test: code
> - crates/duckboard/src/idea_store.rs:1285

## Requirement: Chat message does not create an idea

A first non-priming chat message under an exploration or change scope SHALL NOT by itself
create an idea. Creating an idea for a free exploration or unlinked change is reserved for
annotation gestures (mark cycle or first tag), not chat traffic.

> test: code

### Scenario: First chat message alone does not mint an idea

- **GIVEN** a free exploration with no linked idea and no prior user chat
- **WHEN** the first non-priming user message is sent in that exploration's chat
- **THEN** no idea is created for that exploration

> test: code
> - crates/duckboard/src/idea_store.rs:1311

## Requirement: First mark cycle mints when unlinked

A mark-cycle action on a free exploration or unlinked change that has no linked idea SHALL
create and link an idea (title from the exploration display name or prettified change
slug), then apply the cycled mark to that idea. The exploration SHALL remain on the CHANGE
list after the link.

> test: code

### Scenario: Mark cycle on free exploration creates a linked idea

- **GIVEN** a free exploration with display name `Cloud agent options` and no linked idea
- **WHEN** a mark-cycle action is applied to its CHANGE list row
- **THEN** an exploration-state idea exists whose title is `Cloud agent options`
- **AND** the idea's mark is star
- **AND** the exploration is linked to that idea

> test: code
> - crates/duckboard/src/idea_store.rs:1322
