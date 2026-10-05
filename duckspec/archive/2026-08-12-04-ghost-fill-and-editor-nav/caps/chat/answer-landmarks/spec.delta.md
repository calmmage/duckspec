# @ Chat answer landmarks

## ~ Requirement: Landmark shortcut eligibility

Landmark shortcuts SHALL be available when the chat tab is the active interaction with a
session (terminal not focused; modals that own navigation keys still suppress all
landmarks). History-top and history-bottom shortcuts (Cmd+up / Cmd+down) SHALL remain
available while the composer is focused. Previous/next Answer shortcuts (Cmd+left /
Cmd+right) SHALL NOT claim those keys while the chat input is focused — the focused
composer SHALL receive them for caret line navigation. When the chat input is not focused
and the base landmark gate holds, Cmd+left / Cmd+right SHALL jump previous/next Answer.
Bare arrow keys SHALL remain available to the focused composer for caret movement.

> test: code

### Scenario: Horizontal landmarks yield while chat input is focused

- **GIVEN** the chat tab is active with a session

- **AND** the chat input is focused

- **WHEN** horizontal landmark eligibility is evaluated

- **THEN** previous/next Answer shortcuts are not eligible

- **AND** history-top/history-bottom shortcuts remain eligible under the base landmark
  gate

> test: code

### Scenario: Horizontal landmarks available when chat input is not focused

- **GIVEN** the chat tab is active with a session
- **AND** the chat input is not focused
- **WHEN** horizontal landmark eligibility is evaluated
- **THEN** previous/next Answer shortcuts are eligible under the base landmark gate

> test: code
