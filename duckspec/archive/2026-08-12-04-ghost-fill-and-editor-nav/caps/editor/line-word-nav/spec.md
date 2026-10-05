# Editor line and word nav

Shared TextEdit horizontal navigation: Cmd moves by line ends, Option moves by word.

## Requirement: Line and word arrow navigation

When a TextEdit has keyboard focus, Cmd+left and Cmd+right without Option SHALL move the
caret to the start and end of the current line respectively, including the chat composer
when it is focused (app-level previous/next Answer landmarks SHALL NOT claim Cmd+left /
Cmd+right while the chat input is focused). Option+left and Option+right without Cmd SHALL
move the caret by one word left and right respectively. When Shift is held with either of
those bindings, the selection SHALL extend to the new caret position. Home and End SHALL
continue to move to line start and end. Cmd+Option together with left or right arrow SHALL
NOT be bound to line or word movement.

> test: code

### Scenario: Cmd+arrow moves to line start or end

- **GIVEN** a focused TextEdit with the caret mid-line
- **WHEN** the user presses Cmd+left or Cmd+right without Option
- **THEN** the caret moves to the start or end of that line respectively

> test: code

### Scenario: Option+arrow moves by word

- **GIVEN** a focused TextEdit with the caret mid-line
- **WHEN** the user presses Option+left or Option+right without Cmd
- **THEN** the caret moves one word left or right respectively

> test: code

### Scenario: Shift with Cmd or Option arrow extends selection

- **GIVEN** a focused TextEdit with a caret position and no selection
- **WHEN** the user presses Shift together with Cmd+left/right or Option+left/right
- **THEN** a selection extends from the original caret to the new caret position

> test: code
