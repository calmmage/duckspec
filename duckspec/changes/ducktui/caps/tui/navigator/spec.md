# Tui navigator

The left work-screen pane: a single tree from existing project state (changes,
explorations, archived, idea list for Ideas navigation, plus Codex and Settings entries)
that binds the chat pane to shared scopes when a row is selected. No new on-disk navigator
state; no fixed Ideas session scope. Scope orientation and exploration promotion stay
owned by their existing capabilities.

## Requirement: Tree from existing state

The navigator SHALL build one tree from existing project state only (duckpond changes and
explorations, the project's idea list for Ideas navigation, plus session counts from the
shared session store) and SHALL NOT invent a separate on-disk navigator store. The tree
SHALL include: a CHANGES section with a phase indicator for each active change, an
EXPLORATIONS section, an ARCHIVED section collapsed by default, and bottom entries for
Ideas, Codex, and Settings.

> test: code

### Scenario: Active changes appear under CHANGES with phase indicators

- **GIVEN** a project with at least one active change
- **WHEN** the navigator tree is built
- **THEN** each active change appears under the CHANGES section
- **AND** each active change row shows a phase indicator

### Scenario: Explorations appear under EXPLORATIONS

- **GIVEN** a project with at least one non-archived exploration
- **WHEN** the navigator tree is built
- **THEN** each such exploration appears under the EXPLORATIONS section

### Scenario: ARCHIVED is collapsed by default

- **GIVEN** a project with at least one archived change or exploration
- **WHEN** the navigator tree is first shown
- **THEN** the ARCHIVED section is present and collapsed
- **AND** archived rows are not visible until the section is expanded

### Scenario: Ideas, Codex, and Settings appear as bottom entries

- **GIVEN** a bound project

- **WHEN** the navigator tree is built

- **THEN** Ideas, Codex, and Settings appear as bottom entries outside the change and
  exploration sections

## Requirement: Selection binds chat scope

Selecting a change or exploration row SHALL bind the chat pane to that row's shared
`Scope` with the same binding semantics duckboard uses (including exploration promotion
paths owned by `exploration/promotion`). Selecting Codex SHALL open the codex scope's chat
only and SHALL NOT open a content browser. Selecting Settings SHALL open the settings
screen, not a chat scope.

Selecting Ideas SHALL open ideas navigation in the navigator only (no content browser) and
SHALL NOT bind a fixed ideas session key or invent a global ideas chat scope. Selecting an
idea that is linked to a change or an exploration SHALL bind the chat pane to that idea's
shared `Scope` (change name or exploration id, matching duckboard's idea-to-scope rule).
Selecting an inbox-only idea (no change or exploration link) SHALL NOT bind a chat scope.

> test: code

### Scenario: Selecting a change binds chat to that change scope

- **GIVEN** the work screen with an active change in the navigator
- **WHEN** the user selects that change row
- **THEN** the chat pane is bound to that change scope

### Scenario: Selecting an exploration binds chat to that exploration scope

- **GIVEN** the work screen with an exploration in the navigator
- **WHEN** the user selects that exploration row
- **THEN** the chat pane is bound to that exploration scope

### Scenario: Selecting Codex binds codex chat without a content browser

- **GIVEN** the work screen
- **WHEN** the user selects the Codex entry
- **THEN** the chat pane is bound to the codex scope
- **AND** no content browser pane is shown

### Scenario: Selecting Settings opens the settings screen

- **GIVEN** the work screen
- **WHEN** the user selects the Settings entry
- **THEN** the settings screen is shown
- **AND** chat scope is not rebound as a settings scope

### Scenario: Selecting Ideas opens navigator ideas list without a fixed ideas chat scope

- **GIVEN** the work screen
- **WHEN** the user selects the Ideas entry
- **THEN** ideas navigation is available in the navigator
- **AND** no content browser pane is shown
- **AND** the chat pane is not bound to a fixed ideas session key

### Scenario: Selecting a change-linked idea binds that change scope

- **GIVEN** the work screen with Ideas navigation showing an idea linked to a change
- **WHEN** the user selects that idea
- **THEN** the chat pane is bound to that change scope

### Scenario: Selecting an inbox-only idea does not bind a chat scope

- **GIVEN** the work screen with Ideas navigation showing an idea with no change or
  exploration link

- **WHEN** the user selects that idea

- **THEN** the chat pane is not bound to a chat scope

## Requirement: Session count badges

When a scope has one or more persisted sessions, its navigator row SHALL show a
session-count badge derived from the shared session store. A scope with zero sessions
SHALL show no session badge.

> test: code

### Scenario: Scope with sessions shows the count badge

- **GIVEN** a change scope with N persisted sessions where N is greater than zero
- **WHEN** the navigator tree is built
- **THEN** that change's row shows a session-count badge of N

### Scenario: Scope with zero sessions shows no badge

- **GIVEN** a change scope with no persisted sessions
- **WHEN** the navigator tree is built
- **THEN** that change's row shows no session-count badge
