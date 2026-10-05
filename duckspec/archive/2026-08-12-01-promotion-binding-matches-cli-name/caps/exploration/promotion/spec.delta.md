# @ Exploration promotion

## + Requirement: Binding key matches create-change folder name

The binding key recorded from an exploration session's `ds create change` tool call SHALL
be the create-change argument string exactly as it will appear as the change directory
name under `changes/`. Duckboard SHALL NOT transform that argument (including not
slugifying it) before recording the binding or looking it up when a new change directory
is detected.

> test: code

### Scenario: Quoted multi-word argument binds under that exact string

- **GIVEN** a shell tool call whose command runs `ds create change` with a quoted
  multi-word argument

- **WHEN** the create-change binding key is parsed from that tool input

- **THEN** the binding key is that argument string exactly

- **AND** the binding key is not a slugified form of the argument

> test: code

### Scenario: Spaced title binds as the CLI folder name

- **GIVEN** a shell tool call whose command runs `ds create change` with a quoted argument
  that contains spaces

- **WHEN** the create-change binding key is parsed from that tool input

- **THEN** the binding key equals that spaced argument

- **AND** a newly present change directory with that same name can match the binding

> test: code
