# @ Content / chat column split

Default equal free-space width for the content and interaction columns until the user
customizes via grip drag; absolute width after that; full remaining width for the
interaction column when content is hidden. The list column width is session-mutable via
its own grip and participates in free-space geometry.

## @ Requirement: Uncustomized equal width

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

### + Scenario: List width change rebalances uncustomized half free space

- **GIVEN** an uncustomized interaction panel

- **AND** the content column is shown

- **AND** a window width whose free space with the stock list width allows a half above
  the minimum panel width

- **WHEN** the list column width changes to a larger session value still leaving free
  space for a half above the minimum

- **THEN** the interaction column width equals half of free space computed with the new
  list width

> test: code

## @ Requirement: Content-hidden fill

When the content column is not shown (no open tabs in a three-column area, or content
collapsed), a visible interaction column SHALL take the remaining horizontal space after
left chrome (sidebar and its divider, the current list column width, the list resize grip,
and the handle when present), not the equal-split absolute width.

In any three-column area, the content column SHALL NOT be shown when there are no open
tabs (neither a preview tab nor file tabs). When a list selection opens a tab, the content
column SHALL be shown again (interaction width follows equal or customized rules as when
content is visible).

## + Requirement: List column width

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

### Scenario: Grip sets list width within clamps

- **GIVEN** a three-column layout at the stock list width

- **WHEN** the list resize grip sets the list column to a chosen width inside the allowed
  range

- **THEN** the list column width equals that chosen width for the rest of the session
  until changed again

> test: code
