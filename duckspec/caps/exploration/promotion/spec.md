# Exploration promotion

When a newly-present change directory is detected, duckboard promotes an exploration into
it only on an authoritative binding recorded when that exploration's agent created the
change — never by inferring an owner from UI focus.

## Requirement: Promotion requires an authoritative binding

A change directory detected as newly present SHALL adopt an exploration only when a
binding for that change name exists — the binding recorded when an exploration session's
agent ran `ds create change <name>`. When no binding exists — an out-of-band creation, an
unarchive, or a version-control reappearance of a directory that already existed — the
change SHALL be left standalone and no exploration SHALL be adopted, irrespective of which
exploration is currently selected.

> test: code

### Scenario: Bound change adopts its originating exploration

- **GIVEN** an exploration whose agent created a change by name
- **AND** that change's directory is detected as newly present
- **WHEN** promotion is evaluated
- **THEN** the exploration is promoted into that change
- **AND** the exploration's chat sessions are accessible under the change's scope

> test: code
> - crates/duckboard/src/main.rs:7252

### Scenario: Unbound change adopts no exploration

- **GIVEN** a change directory detected as newly present with no binding for its name
- **AND** an unrelated exploration is currently selected
- **WHEN** promotion is evaluated
- **THEN** no exploration is promoted into the change
- **AND** the selected exploration's chat sessions remain under their own scope

> test: code
> - crates/duckboard/src/main.rs:7284

## Requirement: Bindings are single-use

A binding SHALL be consumed by the promotion it authorizes, so that a later detection of
the same change directory — such as a version-control reappearance after the change was
already promoted — does not promote an exploration again.

> test: code

### Scenario: A consumed binding does not re-promote on reappearance

- **GIVEN** a change whose binding was already consumed by promoting its exploration
- **WHEN** that change's directory is detected as newly present again
- **THEN** no exploration is promoted into the change

> test: code
> - crates/duckboard/src/main.rs:7375

## Requirement: Chat focus after bound promotion

When a newly-present change directory is promoted into from a binding, duckboard SHALL
restore keyboard focus to the chat input after that promotion so the user can continue
typing without re-selecting the input. Detecting a newly-present change directory with no
binding SHALL NOT force chat input focus as a result of that detection alone.

> test: code

### Scenario: Bound promotion restores chat input focus

- **GIVEN** an exploration whose agent created a change by name
- **AND** that change's directory is detected as newly present
- **WHEN** promotion is evaluated
- **THEN** the exploration is promoted into that change
- **AND** the chat input has keyboard focus

> test: code
> - crates/duckboard/src/main.rs:7316

### Scenario: Unbound new change does not force chat input focus

- **GIVEN** a change directory detected as newly present with no binding for its name
- **AND** the chat input does not have keyboard focus
- **WHEN** promotion is evaluated
- **THEN** no exploration is promoted into the change
- **AND** the chat input still does not have keyboard focus

> test: code
> - crates/duckboard/src/main.rs:7345

## Requirement: Binding key matches create-change folder name

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
> - crates/duckboard/src/main.rs:7424

### Scenario: Spaced title binds as the CLI folder name

- **GIVEN** a shell tool call whose command runs `ds create change` with a quoted argument
  that contains spaces

- **WHEN** the create-change binding key is parsed from that tool input

- **THEN** the binding key equals that spaced argument

- **AND** a newly present change directory with that same name can match the binding

> test: code
> - crates/duckboard/src/main.rs:7435
