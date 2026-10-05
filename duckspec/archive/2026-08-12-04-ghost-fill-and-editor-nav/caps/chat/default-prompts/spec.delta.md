# @ Chat default prompts

Conversation-local empty-input defaults from a cheap-model oneshot: parse ordered `REPLY:`
suggestions (heuristic passed only as a soft hint), show and arm them only after the
oneshot settles, and drive empty Enter plus fill and cycle from the next-action list
alone.

Empty-composer next actions from an inherited donor list (empty change sessions after
new-session creation), lifecycle bootstrap, a trailing `next` meta card, or a trailing
A–B–C prose option run when no meta card is present, shown as ghost text with empty Enter
send, Tab/right-arrow fill, and Shift-Tab cycle; optional settings-gated oneshot reply
suggestions (up to three freeform `REPLY:` lines) that may fill fast-response chips only
when there is no next-action ghost.

## = Requirement: Next-action empty-input send and cycle

Requirement: Next-action empty-input send, fill, and cycle

## ~ Requirement: Next-action empty-input send, fill, and cycle

When the composer input is empty and the next-action list is non-empty, submit SHALL send
the active entry's send text. When the next-action list is empty, empty submit SHALL NOT
send a next action. When the composer input is empty, the main turn is not streaming, the
next-action list is non-empty, and slash-command completion is not consuming Tab, bare Tab
and bare right-arrow SHALL insert the active entry's send text into the composer input and
SHALL NOT start a turn. When the input is empty, the list has at least two entries, and
the main turn is not streaming, Shift-Tab SHALL advance the active next-action index with
wrap. Cycling SHALL NOT insert text into the composer input. Reverse cycle SHALL NOT be
required. When the next-action list has more than one entry and the input is empty, a
cycle marker SHALL be shown before the ghost (next-action) affordance as a display-only
Shift-Tab cue. When the list has zero or one entry, that cycle marker SHALL NOT be shown.
The empty composer's ghost (placeholder) text SHALL be the active next-action send text
when the list is non-empty and the main turn is not streaming. Display-only marker
prefixes SHALL NOT be part of fill or send text.

> test: code

### Scenario: Empty submit sends the active next action

- **GIVEN** an empty composer input
- **AND** a non-empty next-action list
- **AND** an active index into that list
- **WHEN** the user submits
- **THEN** the sent text is the send text of the entry at the active index

> test: code

### Scenario: Empty submit is a no-op when the next-action list is empty

- **GIVEN** an empty composer input
- **AND** an empty next-action list
- **WHEN** the user submits
- **THEN** no next-action message is sent

> test: code

### Scenario: Shift-Tab cycles next actions with wrap

- **GIVEN** an empty composer input
- **AND** a next-action list of at least two entries
- **AND** the main turn is not streaming
- **WHEN** the user presses Shift-Tab at the last index
- **THEN** the active index wraps to the first entry
- **AND** the composer input remains empty

> test: code

### Scenario: Tab or right-arrow fills the active next action

- **GIVEN** an empty composer input
- **AND** a non-empty next-action list
- **AND** an active index into that list
- **AND** the main turn is not streaming
- **AND** slash-command completion is not consuming Tab
- **WHEN** the user presses bare Tab or bare right-arrow
- **THEN** the composer input text is the send text of the entry at the active index
- **AND** no agent turn is started

> test: code

### Scenario: Multi next shows a cycle marker

- **GIVEN** an empty composer input
- **AND** a next-action list of at least two entries
- **AND** the main turn is not streaming
- **WHEN** the empty-composer next-action chrome is rendered
- **THEN** a cycle marker is shown before the ghost text

> test: code
