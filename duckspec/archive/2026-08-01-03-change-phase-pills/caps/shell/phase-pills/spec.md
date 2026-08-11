# Phase pills

Derived lifecycle (and late-stage VCS dirty) short pills on the change list and above
chat, with per-surface settings and click-to-send for the next stage command — never a
freeform status field.

## Requirement: Derived stage display

For an active change, the phase-pill display SHALL derive a short stage label and an
optional lifecycle send string from the same disk artifact and step state used for
lifecycle recognition (proposal, design, caps, steps, reviews). The short label SHALL be
one of: `empty`, `proposal`, `design`, `specs`, `steps`, `review`, `ready`. The lifecycle
send, when present, SHALL be the empty-send form of that change's first next-stage option
(`/ds-…`). For an archived change the short label SHALL be `archived` and there SHALL be
no lifecycle send. The long lifecycle hover for an active change SHALL be the recognized
long phase description for that change.

> test: code

### Scenario: Empty change yields empty short and propose send

- **GIVEN** an active change with no proposal, design, caps, steps, or reviews
- **WHEN** the phase-pill display is derived
- **THEN** the short stage is `empty`
- **AND** the lifecycle send is `/ds-propose`

> test: code

### Scenario: Proposal-only yields proposal short and design send

- **GIVEN** an active change that has a proposal and no design, caps, steps, or reviews
- **WHEN** the phase-pill display is derived
- **THEN** the short stage is `proposal`
- **AND** the lifecycle send is `/ds-design`

> test: code

### Scenario: Open steps yield steps short and apply send

- **GIVEN** an active change that has at least one incomplete step
- **WHEN** the phase-pill display is derived
- **THEN** the short stage is `steps`
- **AND** the lifecycle send is `/ds-apply`

> test: code

### Scenario: Complete steps without review yield ready short and archive send

- **GIVEN** an active change whose steps are all complete
- **AND** the change has no reviews
- **WHEN** the phase-pill display is derived
- **THEN** the short stage is `ready`
- **AND** the lifecycle send is `/ds-archive`

> test: code

### Scenario: No open steps with review yield review short and step send

- **GIVEN** an active change with no incomplete steps
- **AND** the change has at least one review
- **WHEN** the phase-pill display is derived
- **THEN** the short stage is `review`
- **AND** the lifecycle send is `/ds-step`

> test: code

### Scenario: Archived yields archived short without lifecycle send

- **GIVEN** an archived change
- **WHEN** the phase-pill display is derived
- **THEN** the short stage is `archived`
- **AND** there is no lifecycle send

> test: code

### Scenario: Active-change long hover is the recognized phase description

- **GIVEN** an active change whose recognized long phase description is known
- **WHEN** the phase-pill display is derived
- **THEN** the lifecycle hover text is that long phase description

> test: code

## Requirement: Late-stage VCS pill

When the short stage is `ready` or `archived`, the phase-pill display SHALL include a
second pill reflecting the working tree versus HEAD: `uncommitted` when the repository has
any pending changes (repo-wide), otherwise `committed`. The uncommitted pill SHALL offer
send text `Commit` and a hover that states the working tree is dirty repo-wide. The
committed pill SHALL offer no send. When the short stage is neither `ready` nor
`archived`, the display SHALL omit the VCS pill.

> test: code

### Scenario: Ready with dirty tree is uncommitted with Commit send

- **GIVEN** a phase-pill display whose short stage is `ready`
- **AND** the working tree has pending changes versus HEAD
- **WHEN** the VCS pill is derived
- **THEN** the VCS short is `uncommitted`
- **AND** the VCS send is `Commit`
- **AND** the VCS hover states that the working tree is dirty repo-wide

> test: code

### Scenario: Ready with clean tree is committed without send

- **GIVEN** a phase-pill display whose short stage is `ready`
- **AND** the working tree is clean versus HEAD
- **WHEN** the VCS pill is derived
- **THEN** the VCS short is `committed`
- **AND** there is no VCS send

> test: code

### Scenario: Pre-ready stage omits VCS pill

- **GIVEN** a phase-pill display whose short stage is not `ready` and not `archived`
- **WHEN** the display is derived
- **THEN** there is no VCS pill

> test: code

### Scenario: Archived includes VCS pill from tree dirty state

- **GIVEN** an archived change
- **AND** the working tree has pending changes versus HEAD
- **WHEN** the phase-pill display is derived
- **THEN** a VCS pill is present
- **AND** the VCS short is `uncommitted`
- **AND** the VCS send is `Commit`

> test: code

## Requirement: Exploration display

For an exploration, the phase-pill display SHALL use short stage `explore`. When that
exploration's chat session is empty (or no session exists yet), the lifecycle send SHALL
be `/ds-explore`. When the session is non-empty, there SHALL be no lifecycle send.

> test: code

### Scenario: Empty exploration offers explore short with explore send

- **GIVEN** an exploration whose chat session is empty
- **WHEN** the phase-pill display is derived
- **THEN** the short stage is `explore`
- **AND** the lifecycle send is `/ds-explore`

> test: code

### Scenario: Non-empty exploration is explore short without send

- **GIVEN** an exploration whose chat session is non-empty
- **WHEN** the phase-pill display is derived
- **THEN** the short stage is `explore`
- **AND** there is no lifecycle send

> test: code

## Requirement: Surface settings

Duckboard settings SHALL expose independent toggles for phase pills on the change list and
above the chat composer. Both toggles SHALL default to enabled. When the list toggle is
disabled, change-list rows SHALL NOT show phase pills. When the chat toggle is disabled,
the chat composer strip SHALL NOT show phase pills.

When the list phase-pill setting is enabled, short clickable phase pills SHALL own list
phase chrome: the Change and Ideas Sort menus SHALL NOT offer a Phase toggle for long
phase text pillows, and toggling long-phase pillow visibility SHALL be ignored. Long phase
text pillows SHALL NOT appear on change or idea list rows while list phase pills are
enabled. When the list phase-pill setting is disabled, Sort Phase and long-phase pillows
MAY apply as before.

> test: code

### Scenario: List and chat phase-pill settings default enabled

- **GIVEN** application config defaults
- **WHEN** the phase-pill list and chat settings are read
- **THEN** both are enabled

> test: code

### Scenario: Disabled list setting hides list pills

- **GIVEN** the phase-pill list setting is disabled
- **AND** a change that would otherwise show a phase pill
- **WHEN** the change list is shown
- **THEN** that row has no phase pill

> manual: change list chrome

### Scenario: Disabled chat setting hides composer pills

- **GIVEN** the phase-pill chat setting is disabled
- **AND** a focused change or exploration session that would otherwise show a phase pill
- **WHEN** the chat composer area is shown
- **THEN** there is no phase-pill strip above the composer

> manual: composer chrome

### Scenario: List phase pills hide Sort Phase toggle

- **GIVEN** the phase-pill list setting is enabled
- **WHEN** the Change or Ideas Sort menu is shown
- **THEN** the Sort Phase control for long phase pillows is not offered

> manual: change list and ideas sort menus

### Scenario: List phase pills suppress long phase text pillows

- **GIVEN** the phase-pill list setting is enabled
- **AND** a change-linked row that would otherwise show a long phase pillow
- **WHEN** the list is shown
- **THEN** that row does not show long phase text as a density pillow
- **AND** short phase pills still appear on change-list rows per the list setting

> manual: change list chrome

## Requirement: Activation

Activating the lifecycle pill SHALL submit the display's lifecycle send text (empty-send
next-stage form) into the target change or exploration chat session when a lifecycle send
is present. Activating the uncommitted VCS pill SHALL submit `Commit`. Activation SHALL
NOT invent or write a freeform lifecycle status on disk.

List-origin activation SHALL select the target scope when it is not already selected, and
SHALL NOT start exploration rename when the target exploration is already selected.

> test: code

### Scenario: Lifecycle activation submits empty-send next-stage text

- **GIVEN** a phase-pill display with lifecycle send `/ds-design`
- **WHEN** the lifecycle pill is activated
- **THEN** the submitted prompt text is `/ds-design`

> test: code

### Scenario: Uncommitted activation submits Commit

- **GIVEN** a phase-pill display with VCS send `Commit`
- **WHEN** the VCS pill is activated
- **THEN** the submitted prompt text is `Commit`

> test: code

## Requirement: Placement chrome

When the list setting is enabled, phase pills SHALL appear on exploration, active-change,
and archived rows in the change list. When the chat setting is enabled, phase pills for
the focused change or exploration SHALL appear above the chat composer (outside the
transcript scroll). Caps and codex scopes SHALL NOT show a chat phase-pill strip.

> manual: change list and composer chrome

### Scenario: List pills sit on change rows when list setting is on

- **GIVEN** the phase-pill list setting is enabled
- **AND** an active change with a derivable phase display
- **WHEN** the change list is shown
- **THEN** that change's row shows its phase pill

> manual: change list chrome

### Scenario: Chat pills sit above the composer when chat setting is on

- **GIVEN** the phase-pill chat setting is enabled
- **AND** a focused change session with a derivable phase display
- **WHEN** the chat composer area is shown
- **THEN** the phase-pill strip appears above the composer input

> manual: composer chrome
