# Idea queue list

Shared presentation for CHANGE and Ideas queues: pin up to three newest stars, sort the
rest by a chosen key, and show type and phase pillows with density and visibility prefs.

## Requirement: Star pin before ordinary sort

When ordering a queue list body, the system SHALL place up to three starred rows with the
newest pin times ahead of all non-pinned rows, ordered among themselves by pin time
newest-first. Any additional starred row beyond those three SHALL participate in ordinary
sort with non-pinned rows and SHALL NOT occupy a pin slot.

> test: code

### Scenario: Up to three newest stars pin above non-pinned rows

- **GIVEN** a queue body with three starred rows and several non-starred rows
- **WHEN** the queue is ordered
- **THEN** the three starred rows appear before every non-starred row

> test: code
> - crates/duckboard/src/queue_list.rs:365

### Scenario: A fourth star follows ordinary sort among non-pinned rows

- **GIVEN** a queue body with four starred rows whose pin times differ

- **WHEN** the queue is ordered

- **THEN** exactly three starred rows occupy the pin prefix

- **AND** the remaining starred row appears among the non-pinned segment under the
  ordinary sort key

> test: code
> - crates/duckboard/src/queue_list.rs:383

### Scenario: Pin ranking uses pin time newest-first

- **GIVEN** three starred rows with distinct pin times
- **WHEN** the queue is ordered
- **THEN** those three appear in the pin prefix ordered by pin time newest-first

> test: code
> - crates/duckboard/src/queue_list.rs:401

## Requirement: Ordinary sort keys

After the pin prefix, the system SHALL order remaining rows by the active sort key. The
default key SHALL be last non-priming chat message time, newest-first. Under that key, a
row with no known activity time SHALL sort after every row that has one. A phase key SHALL
order by the row's derived duckspec phase along a stable lifecycle ladder, with title as
tie-break; rows without a phase SHALL sort after phased rows. A created key SHALL order by
the row's creation time.

> test: code

### Scenario: Default key orders by last non-priming message time newest-first

- **GIVEN** two non-pinned rows whose latest non-priming message times differ
- **AND** the active sort key is last-message (default)
- **WHEN** the queue is ordered
- **THEN** the row with the newer last message appears before the other

> test: code
> - crates/duckboard/src/queue_list.rs:414

### Scenario: Missing activity sorts after rows with activity under last-message key

- **GIVEN** a non-pinned row with a known last-message time
- **AND** a non-pinned row with no known last-message time
- **AND** the active sort key is last-message
- **WHEN** the queue is ordered
- **THEN** the row with known activity appears before the row without

> test: code
> - crates/duckboard/src/queue_list.rs:426

### Scenario: Phase key orders by lifecycle phase then title

- **GIVEN** non-pinned change-linked rows at different derived phases
- **AND** the active sort key is phase
- **WHEN** the queue is ordered
- **THEN** the rows appear in lifecycle phase order
- **AND** rows that share a phase are ordered by title

> test: code
> - crates/duckboard/src/queue_list.rs:438

### Scenario: Created key orders by creation time

- **GIVEN** two non-pinned rows with different creation times
- **AND** the active sort key is created
- **WHEN** the queue is ordered
- **THEN** they appear ordered by creation time

> test: code
> - crates/duckboard/src/queue_list.rs:476

## Requirement: List preferences and sort menu

The system SHALL persist a shared list preference set that includes the active sort key
and independent show/hide flags for type pillows and phase pillows. Those preferences
SHALL apply to both the Change queue and the Ideas queue. Type and phase pillow flags
SHALL default to shown. The Change section header and Ideas section headers SHALL each
offer a sort menu that can change those preferences.

> test: code

### Scenario: Sort key preference is shared by Change and Ideas lists

- **GIVEN** the sort key preference is set to phase from the Change sort menu
- **WHEN** the Ideas queue is ordered
- **THEN** it uses the phase sort key

> test: code
> - crates/duckboard/src/queue_list.rs:488

### Scenario: Type and phase pillow visibility prefs default on and are toggled from the menu

- **GIVEN** default list preferences under which type and phase pillows are enabled
- **WHEN** the sort menu turns type pillows off
- **THEN** type pillows are disabled
- **AND** phase pillows remain enabled

> test: code
> - crates/duckboard/src/queue_list.rs:501

### Scenario: Sort menu is available on Change and Ideas section headers

- **GIVEN** the Change area list and the Ideas area list are shown
- **WHEN** each section header that owns a queue body is inspected
- **THEN** a sort menu control is present on the Change header
- **AND** a sort menu control is present on each Ideas section header

> manual: iced header chrome placement

## Requirement: Type and phase pillows with density

When type pillows are enabled, a queue row SHALL present its secondary tags as type
pillows and SHALL NOT present the primary tag as a type pillow. When phase pillows are
enabled and the row is change-linked with a derived duckspec phase, the row SHALL present
that phase as a phase pillow. When the measured title plus enabled pillows would exceed
the row's content width, the system SHALL omit pillows from the steady row and SHALL show
those pillows only while the row is hovered. When a pillow kind's preference is off, that
kind SHALL not appear even if width remains.

> test: code

### Scenario: Secondary tags render as type pillows; primary does not

- **GIVEN** an idea whose tags are primary `parser` and secondary `bugfix`
- **AND** type pillows are enabled
- **WHEN** its queue row is projected with sufficient width
- **THEN** a type pillow for `bugfix` is present
- **AND** no type pillow for `parser` is present

> test: code
> - crates/duckboard/src/queue_list.rs:531

### Scenario: Change-linked row can show derived phase pillow when pref is on

- **GIVEN** a change-linked row whose derived phase is non-empty
- **AND** phase pillows are enabled
- **WHEN** the row is projected with sufficient width
- **THEN** a phase pillow showing that phase is present

> test: code
> - crates/duckboard/src/queue_list.rs:541

### Scenario: When title plus pillows overflow row width, pillows hide until row hover

- **GIVEN** a row whose title plus enabled pillows exceed the row content width
- **AND** the row is not hovered
- **WHEN** the row is projected
- **THEN** no type or phase pillows are shown on the steady row
- **AND** the same row under hover shows the enabled pillows

> test: code
> - crates/duckboard/src/queue_list.rs:565

### Scenario: Hidden pillow prefs suppress those pillows even when space remains

- **GIVEN** a row with secondary tags and sufficient width for pillows
- **AND** type pillows are disabled
- **WHEN** the row is projected
- **THEN** no type pillows are shown

> test: code
> - crates/duckboard/src/queue_list.rs:578
