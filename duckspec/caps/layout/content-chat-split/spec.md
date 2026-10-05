# Content / chat column split

Default equal free-space width for the content and interaction columns until the user
customizes via grip drag; absolute width after that; full remaining width for the
interaction column when content is hidden.

Default equal free-space width for the content and interaction columns until the user
customizes via grip drag; absolute width after that; full remaining width for the
interaction column when content is hidden. The list column width is session-mutable via
its own grip and participates in free-space geometry.

## Requirement: Uncustomized equal width

While an interaction panel is uncustomized and the content column is shown, the
interaction column width SHALL equal half of free horizontal space: window width minus
left chrome (sidebar and its vertical divider, the **current** list column width, the list
resize grip, and the interaction handle). That half SHALL be floored at the minimum panel
width. Uncustomized half-width SHALL NOT be capped by a fixed maximum below free space (so
a true half is allowed on wide windows). Window resize, list-column width change, door
open, programmatic force-show of the panel, and other default layout refresh while
uncustomized SHALL recompute this equal width from the current free space. Constructing a
new interaction panel with a known window width SHALL initialize the uncustomized width to
half of free space for that window (not only a fixed default window size). Force-showing a
panel SHALL NOT mark it customized.

### Scenario: Default half of free space

- **GIVEN** an uncustomized interaction panel

- **AND** the content column is shown

- **AND** a window width whose free space is large enough for a half above the minimum
  panel width

- **WHEN** the interaction column width is resolved for layout

- **THEN** the width equals half of free space for that window

> test: code
> - crates/duckboard/src/area/interaction.rs:3157

### Scenario: Resize rebalances to half free space

- **GIVEN** an uncustomized interaction panel

- **AND** the content column is shown

- **WHEN** the window width changes to a new size whose free space still allows a half
  above the minimum panel width

- **THEN** the interaction column width equals half of free space for the new window

> test: code
> - crates/duckboard/src/area/interaction.rs:3176

### Scenario: Half floors at minimum panel width

- **GIVEN** an uncustomized interaction panel
- **AND** a window width whose free space is less than twice the minimum panel width
- **WHEN** the interaction column width is resolved for layout
- **THEN** the width equals the minimum panel width

> test: code
> - crates/duckboard/src/area/interaction.rs:3240

### Scenario: Half may exceed the old fixed max width

- **GIVEN** an uncustomized interaction panel
- **AND** a window width whose free space is more than twice 800 logical pixels
- **WHEN** the interaction column width is resolved for layout
- **THEN** the width equals half of free space
- **AND** the width is greater than 800 logical pixels

> test: code
> - crates/duckboard/src/area/interaction.rs:3258

### Scenario: Programmatic open rebalances to half free space

- **GIVEN** an uncustomized interaction panel whose width was set for a different window
  size

- **AND** the content column is shown

- **AND** a current window width whose free space is large enough for a half above the
  minimum panel width

- **WHEN** the panel is force-shown without a door open

- **THEN** the interaction column width equals half of free space for the current window

- **AND** the panel remains uncustomized

> test: code
> - crates/duckboard/src/area/interaction.rs:3444

### Scenario: Panel created for a known window starts at half free space

- **GIVEN** a window width whose free space is large enough for a half above the minimum
  panel width

- **AND** that width is not the fixed default window size

- **WHEN** a new uncustomized interaction panel is constructed for that window

- **THEN** the interaction column width equals half of free space for that window

> test: code
> - crates/duckboard/src/area/interaction.rs:3419

### Scenario: List width change rebalances uncustomized half free space

- **GIVEN** an uncustomized interaction panel

- **AND** the content column is shown

- **AND** a window width whose free space with the stock list width allows a half above
  the minimum panel width

- **WHEN** the list column width changes to a larger session value still leaving free
  space for a half above the minimum

- **THEN** the interaction column width equals half of free space computed with the new
  list width

> test: code
> - crates/duckboard/src/area/interaction.rs:3190

## Requirement: Grip customization

The first grip-driven width change on an interaction panel SHALL mark that panel
customized and set its width to the chosen absolute value. While customized, window resize
SHALL keep that absolute width (no equal rebalance). Opening, closing, or
collapsing/restoring the content column SHALL NOT mark the panel customized by itself.

> test: code

### Scenario: First grip width change locks absolute width

- **GIVEN** an uncustomized interaction panel

- **WHEN** the grip sets the interaction column to a chosen absolute width

- **THEN** the panel is customized

- **AND** the interaction column width equals that absolute width while the content column
  is shown

> test: code
> - crates/duckboard/src/area/interaction.rs:3276

### Scenario: Resize after lock keeps absolute width

- **GIVEN** a customized interaction panel with a remembered absolute width
- **AND** the content column is shown
- **WHEN** the window width changes
- **THEN** the interaction column width remains that absolute width

> test: code
> - crates/duckboard/src/area/interaction.rs:3297

### Scenario: Open/close and content collapse do not lock

- **GIVEN** an uncustomized interaction panel

- **WHEN** the panel is closed and opened again

- **AND** the content column is collapsed and restored without a grip width change

- **THEN** the panel remains uncustomized

- **AND** the interaction column width still equals half of free space for the current
  window while the content column is shown

> test: code
> - crates/duckboard/src/area/interaction.rs:3366

## Requirement: Content-hidden fill

When the content column is not shown (no open tabs in a three-column area, or content
collapsed), a visible interaction column SHALL take the remaining horizontal space after
left chrome (sidebar and its divider, the current list column width, the list resize grip,
and the handle when present), not the equal-split absolute width.

In any three-column area, the content column SHALL NOT be shown when there are no open
tabs (neither a preview tab nor file tabs). When a list selection opens a tab, the content
column SHALL be shown again (interaction width follows equal or customized rules as when
content is visible).

### Scenario: Interaction column fills when content column is hidden

- **GIVEN** a visible interaction panel in a three-column area

- **AND** the content column is not shown

- **WHEN** the three-column area is laid out

- **THEN** the interaction column uses the remaining width after fixed left chrome rather
  than a fixed equal-split width

> test: code
> - crates/duckboard/src/area/interaction.rs:3317

### Scenario: No open tabs hides content column

- **GIVEN** a three-column area with a visible interaction panel
- **AND** no open tabs (no preview and no file tabs)
- **AND** content is not manually collapsed
- **WHEN** the three-column area is laid out
- **THEN** the content column is not shown
- **AND** the interaction column fills the remaining width after fixed left chrome

> test: code
> - crates/duckboard/src/area/interaction.rs:3334

### Scenario: Opening a tab restores content column

- **GIVEN** a three-column area where the content column is hidden because there are no
  open tabs

- **AND** a visible interaction panel

- **WHEN** a list selection opens a tab

- **THEN** the content column is shown

- **AND** the interaction column uses its equal or customized fixed width rather than fill

> test: code
> - crates/duckboard/src/area/interaction.rs:3350

## Requirement: List column width

In three-column areas the list column width SHALL be user-adjustable via a drag grip
between the list column and the content column. The width SHALL default to the stock list
column width and SHALL be remembered only for the process session (not durable
configuration). The width SHALL be clamped between a minimum list width and a maximum that
respects a soft upper bound and remaining window room for minimum interaction chrome.
Changing the list width SHALL update free space for the content ↔ interaction split.

> test: code

### Scenario: Free space uses live list width

- **GIVEN** a fixed window width
- **AND** free space computed at the stock list column width
- **WHEN** free space is computed again with a larger list column width
- **THEN** free space is strictly smaller than free space at the stock list width

> test: code
> - crates/duckboard/src/area/interaction.rs:3213

### Scenario: Grip sets list width within clamps

- **GIVEN** a three-column layout at the stock list width

- **WHEN** the list resize grip sets the list column to a chosen width inside the allowed
  range

- **THEN** the list column width equals that chosen width for the rest of the session
  until changed again

> test: code
> - crates/duckboard/src/area/interaction.rs:3225
