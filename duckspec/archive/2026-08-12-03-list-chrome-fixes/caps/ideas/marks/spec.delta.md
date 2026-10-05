# @ Idea marks

Exclusive boost marks on ideas — none, star, hot, or cool — with a pin time for stars,
inherited by linked CHANGE and exploration list rows, and shown as a single interactive
control on CHANGE and Ideas lists.

## + Requirement: Single interactive mark control

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

### Scenario: Unmarked hovered row exposes outline control glyph

- **GIVEN** a list row whose mark is none
- **AND** the row is hovered
- **WHEN** the mark control glyph is resolved for paint
- **THEN** the outline star glyph is produced

> test: code

### Scenario: Marked row always exposes the mark glyph

- **GIVEN** a list row whose mark is star, hot, or cool
- **WHEN** the mark control glyph is resolved for paint with or without hover
- **THEN** the corresponding mark glyph is produced

> test: code
