# List digit switch

Shell `Ctrl+1/2/3` selects the nth digit-indexed row of the focused Change or Ideas queue,
using the list’s paint-order sequences for those rows, with no digit chrome on rows.

## Requirement: Chord and eligibility

With a project open and the active area Change or Ideas, `Ctrl+1`, `Ctrl+2`, and `Ctrl+3`
SHALL resolve list-digit selection for that area. The chord SHALL remain eligible when
chat or content focus is inside that area. The chord SHALL NOT resolve when a modal or
exploration rename owns navigation keys, when the active area is not Change or Ideas, or
when no project is open.

> test: code

### Scenario: Change area with project resolves digit

- **GIVEN** a project is open
- **AND** the active area is Change
- **AND** the live queue has at least one painted row
- **WHEN** `Ctrl+1` is pressed
- **THEN** list-digit selection resolves for index 1 of the Change live queue

### Scenario: Ideas area with project resolves digit

- **GIVEN** a project is open
- **AND** the active area is Ideas
- **AND** the Ideas list has at least one digit-indexed idea row
- **WHEN** `Ctrl+1` is pressed
- **THEN** list-digit selection resolves for index 1 of the Ideas idea sequence

### Scenario: Other area does not resolve

- **GIVEN** a project is open
- **AND** the active area is neither Change nor Ideas
- **WHEN** `Ctrl+1` is pressed
- **THEN** list-digit selection does not resolve

### Scenario: Modal or rename capture does not resolve

- **GIVEN** a project is open
- **AND** the active area is Change or Ideas
- **AND** a modal or exploration rename owns navigation keys
- **WHEN** `Ctrl+1` is pressed
- **THEN** list-digit selection does not resolve

### Scenario: Eligible with chat focused in Change or Ideas

- **GIVEN** a project is open
- **AND** the active area is Change or Ideas
- **AND** chat input is focused
- **AND** the digit-indexed list has at least one row
- **WHEN** `Ctrl+1` is pressed
- **THEN** list-digit selection resolves for index 1

## Requirement: Painted-row index

Digit `n` (1 through 3) SHALL address the nth row of the digit-indexed paint sequence for
the active area, matching how that list orders those rows. For Change, that sequence SHALL
be the painted live body: explorations on the live list and active changes after shared
queue sort and star pins, then pending-commit archives when shown after live WIP. The
Archived section and the files explorer SHALL NOT be digit-indexed. For Ideas, that
sequence SHALL be selectable idea rows only, in paint order across expanded sections
(nested idea paths under expanded tags included). Tag-folder chrome and other non-idea
rows SHALL NOT be digit-indexed. Collapsed sections SHALL contribute no rows. A digit with
no corresponding row SHALL NOT select a row.

> test: code

### Scenario: Change nth row matches live-queue order

- **GIVEN** the active area is Change
- **AND** the live queue paints rows in a known order
- **WHEN** `Ctrl+2` is pressed
- **THEN** the second live-queue row is the digit target

### Scenario: Ideas skips collapsed sections

- **GIVEN** the active area is Ideas
- **AND** a section that contains idea rows is collapsed
- **AND** an earlier expanded section has fewer than three digit-indexed idea rows
- **WHEN** list-digit indexes are resolved
- **THEN** idea rows from the collapsed section are not indexed

### Scenario: Ideas includes nested painted rows in order

- **GIVEN** the active area is Ideas
- **AND** an expanded section paints a parent idea and a nested idea path after it
- **WHEN** list-digit indexes are resolved
- **THEN** the parent and nested ideas occupy consecutive digit indexes in paint order

### Scenario: Out-of-range digit selects nothing

- **GIVEN** the active area is Change or Ideas
- **AND** the digit-indexed sequence has fewer than three rows
- **WHEN** `Ctrl+3` is pressed
- **THEN** no row is selected by list-digit

## Requirement: Selection without rename side effects

Resolving a list digit SHALL select the target change, exploration, or idea and SHALL
leave the active area unchanged. When the target is already the current selection,
resolution SHALL be a no-op and SHALL NOT start exploration rename.

> test: code

### Scenario: Digit selects a different live-queue row

- **GIVEN** the active area is Change
- **AND** a live-queue row other than the current selection is at index 1
- **WHEN** `Ctrl+1` is pressed
- **THEN** that row becomes the selected change or exploration
- **AND** the active area remains Change

### Scenario: Digit selects a different idea row

- **GIVEN** the active area is Ideas
- **AND** a digit-indexed idea other than the current selection is at index 1
- **WHEN** `Ctrl+1` is pressed
- **THEN** that idea becomes selected
- **AND** the active area remains Ideas

### Scenario: Already-selected exploration is no-op without rename

- **GIVEN** the active area is Change
- **AND** the selected row is an exploration at live-queue index 1
- **WHEN** `Ctrl+1` is pressed
- **THEN** selection is unchanged
- **AND** exploration rename is not started
