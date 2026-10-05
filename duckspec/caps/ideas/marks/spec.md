# Idea marks

Exclusive boost marks on ideas — none, star, hot, or cool — with a pin time for stars,
inherited by linked CHANGE and exploration list rows.

Exclusive boost marks on ideas — none, star, hot, or cool — with a pin time for stars,
inherited by linked CHANGE and exploration list rows, and shown as a single interactive
control on CHANGE and Ideas lists.

## Requirement: Exclusive mark on the idea

An idea SHALL carry exactly one boost mark among *none*, *star*, *hot*, and *cool*.
Cycling the mark SHALL advance in that order and wrap from *cool* back to *none*. The mark
SHALL be durable on the idea so a later load of the same idea restores it. A stored value
that is not one of those four marks SHALL load as *none*.

> test: code

### Scenario: Mark cycles through none, star, hot, cool

- **GIVEN** an idea whose mark is none
- **WHEN** the mark is cycled four times in succession
- **THEN** the marks after each cycle are star, hot, cool, and none in that order

> test: code
> - crates/duckboard/src/idea_store.rs:1059

### Scenario: Mark persists across idea reload

- **GIVEN** an idea whose mark is hot
- **AND** the idea has been saved
- **WHEN** the idea is loaded again
- **THEN** the idea's mark is hot

> test: code
> - crates/duckboard/src/idea_store.rs:1073

### Scenario: Unknown stored mark loads as none

- **GIVEN** idea storage whose mark field is not none, star, hot, or cool
- **WHEN** the idea is loaded
- **THEN** the idea's mark is none

> test: code
> - crates/duckboard/src/idea_store.rs:1087

## Requirement: Star pin time

When an idea's mark becomes *star*, the system SHALL record a pin time for that idea. When
the mark leaves *star*, the pin time SHALL be cleared. When the mark becomes *star* again
after having left it, the pin time SHALL be a new time at that transition, not the
previous star's pin time.

> test: code

### Scenario: Entering star records a pin time

- **GIVEN** an idea whose mark is none and which has no pin time
- **WHEN** the mark becomes star
- **THEN** the idea has a pin time

> test: code
> - crates/duckboard/src/idea_store.rs:1095

### Scenario: Leaving star clears pin time

- **GIVEN** an idea whose mark is star and which has a pin time
- **WHEN** the mark becomes hot
- **THEN** the idea has no pin time

> test: code
> - crates/duckboard/src/idea_store.rs:1110

### Scenario: Re-entering star refreshes pin time

- **GIVEN** an idea that was star with an earlier pin time
- **AND** its mark is no longer star
- **WHEN** the mark becomes star again
- **THEN** the idea's pin time is later than the earlier pin time

> test: code
> - crates/duckboard/src/idea_store.rs:1126

## Requirement: Linked rows inherit the idea mark

A CHANGE list row for a change or exploration that is linked to an idea SHALL expose that
idea's current mark. A CHANGE list row with no linked idea SHALL expose no mark until a
mark-cycle (or other mint) creates a linked idea.

> test: code

### Scenario: Change-linked row exposes the idea's mark

- **GIVEN** an idea linked to a change whose mark is cool
- **WHEN** the CHANGE list row for that change is projected
- **THEN** the row's mark is cool

> test: code
> - crates/duckboard/src/idea_store.rs:1167

### Scenario: Exploration-linked row exposes the idea's mark

- **GIVEN** an idea linked to an exploration whose mark is star
- **WHEN** the CHANGE list row for that exploration is projected
- **THEN** the row's mark is star

> test: code
> - crates/duckboard/src/idea_store.rs:1185

### Scenario: Unlinked row has no mark until mark cycle mints

- **GIVEN** an exploration with no linked idea
- **WHEN** the CHANGE list row for that exploration is projected without a mint
- **THEN** the row has no mark

> test: code
> - crates/duckboard/src/idea_store.rs:1195

## Requirement: Single interactive mark control

On CHANGE and Ideas list rows, the mark SHALL appear as at most one interactive control
and SHALL NOT also appear as a prefix of the row title. When the mark is *none*, the
outline control SHALL appear only while the row is hovered; when the mark is not *none*,
the control SHALL be shown regardless of hover. The row title text SHALL be the plain
display name without an embedded mark glyph. Activating the control SHALL cycle the mark
(including minting a linked idea when the row is unlinked).

> test: code

### Scenario: Unmarked non-hovered row has no mark glyph

- **GIVEN** a list row whose mark is none
- **AND** the row is not hovered
- **WHEN** the mark control glyph is resolved for paint
- **THEN** no mark glyph is produced

> test: code
> - crates/duckboard/src/queue_list.rs:696

### Scenario: Unmarked hovered row exposes outline control glyph

- **GIVEN** a list row whose mark is none
- **AND** the row is hovered
- **WHEN** the mark control glyph is resolved for paint
- **THEN** the outline star glyph is produced

> test: code
> - crates/duckboard/src/queue_list.rs:705

### Scenario: Marked row always exposes the mark glyph

- **GIVEN** a list row whose mark is star, hot, or cool
- **WHEN** the mark control glyph is resolved for paint with or without hover
- **THEN** the corresponding mark glyph is produced

> test: code
> - crates/duckboard/src/queue_list.rs:714
