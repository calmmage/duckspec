# Tui shell

The ducktui application shell: which full screens exist, how the work screen is laid out,
how focus and overlays capture input, and what the status bar shows.

## Requirement: Three screens

Ducktui SHALL expose exactly three full screens: project picker (when no project is
bound), work screen (when a project is bound), and settings. Leaving settings returns to
the work screen when a project is bound, otherwise to the project picker.

> test: code

### Scenario: No bound project opens the project picker

- **GIVEN** no project is bound
- **WHEN** the app starts
- **THEN** the project picker screen is shown

### Scenario: Binding a project opens the work screen

- **GIVEN** the project picker is shown
- **WHEN** a project path is bound
- **THEN** the work screen is shown

### Scenario: Settings returns to the bound or unbound surface

- **GIVEN** settings is open
- **WHEN** the user leaves settings
- **THEN** the work screen is shown if a project is bound
- **AND** the project picker is shown if no project is bound

## Requirement: Work screen two panes

The work screen SHALL be exactly two panes — scope navigator (left) and chat (right) —
with no middle content column. Exactly one of the two panes is focused at a time; Tab
toggles focus between them.

> test: code

### Scenario: Work screen has navigator and chat only

- **GIVEN** a bound project
- **WHEN** the work screen is shown
- **THEN** the layout contains a scope navigator pane and a chat pane
- **AND** the layout contains no middle content column

### Scenario: Tab toggles pane focus

- **GIVEN** the work screen with the navigator focused
- **WHEN** the user presses Tab
- **THEN** the chat pane is focused
- **AND** when Tab is pressed again the navigator is focused

## Requirement: Overlay input capture

While an overlay is open (model picker, slash palette, session switcher, quick idea, or
help), the overlay SHALL capture all input that is not a global key. Closing the overlay
restores pane dispatch.

> test: code

### Scenario: Open overlay consumes pane-destined keys

- **GIVEN** the work screen with an overlay open
- **AND** a key that would otherwise dispatch to the focused pane
- **WHEN** the user presses that key
- **THEN** the overlay handles the key
- **AND** the focused pane does not receive it

### Scenario: Closing the overlay restores pane dispatch

- **GIVEN** an open overlay on the work screen
- **WHEN** the overlay is closed
- **THEN** subsequent non-global keys dispatch to the focused pane

## Requirement: Global keys before pane dispatch

Global keys (quit, help, screen switches) SHALL resolve on a top layer before pane or
overlay content handling, except where an open overlay documents a conflicting binding it
owns while open.

> test: code

### Scenario: Help opens from either focused pane

- **GIVEN** the work screen with the navigator focused
- **WHEN** the user presses the global help key
- **THEN** the help overlay opens
- **AND** the same key opens help when the chat pane is focused

### Scenario: Quit is available without a pane handler

- **GIVEN** the work screen
- **WHEN** the user presses the global quit key
- **THEN** the app begins shutdown
- **AND** quit does not require a focused-pane key handler

## Requirement: Status bar

A bottom status bar SHALL span every screen and show the current scope identity when a
project is bound, the selected model, context fill, and turn state.

> test: code

### Scenario: Work screen status reports scope, model, context, and turn

- **GIVEN** the work screen with a bound scope and a selected model
- **WHEN** the status bar is rendered
- **THEN** it shows the bound scope identity
- **AND** it shows the selected model
- **AND** it shows context fill
- **AND** it shows the current turn state

### Scenario: Project picker status has no scope

- **GIVEN** the project picker with no project bound
- **WHEN** the status bar is rendered
- **THEN** it shows no scope identity
- **AND** it shows the selected model
- **AND** it shows an idle turn state
