# Tui chat

Terminal presentation of the chat pane: how transcript segments, meta-card next tokens,
fast-response options, and the composer appear and accept input in a terminal. Segment
construction, slash semantics, and fast-response policy stay owned by existing `chat/*`
capabilities; this capability owns only terminal rendering and input bindings.

## Requirement: Settled segment collapse

Thinking and Activity segments SHALL render collapsed to one summary line each, expandable
per segment. Answer segments SHALL always render expanded. This matches the
settled-collapse defaults of `chat/transcript` without redefining segment construction.

> test: code

### Scenario: Settled Thinking shows one summary line until expanded

- **GIVEN** a settled Thinking segment in the transcript
- **WHEN** the chat pane renders the transcript
- **THEN** that segment appears as a single summary line
- **AND** expanding it reveals the full Thinking body

### Scenario: Settled Activity shows one summary line until expanded

- **GIVEN** a settled Activity segment in the transcript
- **WHEN** the chat pane renders the transcript
- **THEN** that segment appears as a single summary line
- **AND** expanding it reveals the full Activity body

### Scenario: Answer body is fully visible without expand

- **GIVEN** an Answer segment in the transcript
- **WHEN** the chat pane renders the transcript
- **THEN** the Answer body is visible without an expand action

## Requirement: Autoscroll pin during stream

While a turn is streaming and stick-to-bottom is engaged, the transcript viewport SHALL
pin to the latest content. Manual scroll away from the bottom SHALL release the pin until
the user returns to the bottom or an intentional session open or switch re-engages
stick-to-bottom.

> test: code

### Scenario: Streaming with stick engaged keeps latest lines in view

- **GIVEN** a streaming turn with stick-to-bottom engaged
- **WHEN** new transcript content arrives
- **THEN** the viewport remains pinned so the latest content is visible

### Scenario: Manual scroll up releases the pin

- **GIVEN** a streaming turn with stick-to-bottom engaged
- **WHEN** the user scrolls the transcript away from the bottom
- **THEN** stick-to-bottom is released
- **AND** further stream output does not force the viewport back to the bottom

## Requirement: Numbered action hints

Trailing `next` meta-card tokens and visible fast-response options SHALL render as
numbered hints (1–9). Pressing the matching number key while those hints are active SHALL
activate the corresponding token or option, using existing meta-card and fast-response
semantics.

> test: code

### Scenario: Trailing next tokens appear as numbered hints

- **GIVEN** a trailing `next` meta card with one or more tokens
- **WHEN** the chat pane renders the transcript
- **THEN** those tokens appear as numbered hints in order

### Scenario: Number key activates the matching next token

- **GIVEN** numbered next-token hints are active
- **WHEN** the user presses the number key for one of those hints
- **THEN** the matching next token is activated

### Scenario: Fast-response options appear as numbered hints while awaiting

- **GIVEN** a session awaiting a user choice with visible fast-response options
- **WHEN** the chat pane renders the composer region
- **THEN** those options appear as numbered hints

### Scenario: Number key activates the matching fast-response option

- **GIVEN** numbered fast-response hints are active while awaiting a user choice
- **WHEN** the user presses the number key for one of those hints
- **THEN** the matching fast-response option is activated

## Requirement: Composer send and newline

In the chat composer, Enter SHALL send the current input. Alt+Enter SHALL insert a
newline. Shift+Enter MAY insert a newline only when the terminal keyboard protocol reports
it.

> test: code

### Scenario: Enter with non-empty composer submits

- **GIVEN** a focused composer with non-empty input
- **WHEN** the user presses Enter
- **THEN** the input is submitted

### Scenario: Alt+Enter inserts a newline without submitting

- **GIVEN** a focused composer with non-empty input
- **WHEN** the user presses Alt+Enter
- **THEN** a newline is inserted in the composer
- **AND** the input is not submitted

## Requirement: Slash palette open

When the composer content starts with `/` at line start and is not the `//` pass-through
case, the slash palette SHALL open, fed by the shared slash catalog. The `//` escape
passes through per `chat/slash-commands` and SHALL NOT open the palette as a catalog query
for a single leading slash command.

> test: code

### Scenario: Slash at line start opens the palette

- **GIVEN** a focused composer whose content is empty or at line start
- **WHEN** the user types `/` as the first character of the line
- **THEN** the slash palette opens

### Scenario: Double-slash does not open as a single-slash catalog

- **GIVEN** a focused composer at line start

- **WHEN** the user types `//`

- **THEN** the slash palette does not open as a catalog query for a single leading slash
  command
