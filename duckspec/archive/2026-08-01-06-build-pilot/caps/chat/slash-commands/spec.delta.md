# @ Chat slash commands

Kinded slash-command catalog for chat completion, local system handlers (including `/help`
and build-pilot commands), and a double-slash escape so colliding agent skills stay
reachable.

## @ Requirement: Kinded completion catalog

### + Scenario: build-auto and build-fast are System

- **GIVEN** the system registry includes commands named `build-auto` and `build-fast`
- **WHEN** the completion catalog is built
- **THEN** the catalog includes an entry named `build-auto` whose kind is System
- **AND** the catalog includes an entry named `build-fast` whose kind is System

> test: code

## + Requirement: Build pilot system classification

Submitting `/build-auto` or `/build-fast`, with or without trailing free-text arguments,
SHALL be classified as a local build-pilot system submit — not as an ordinary agent-only
submit of the raw `/build-*` text. Those names SHALL appear in the duckboard system
registry. What arming, kick rewrite, and auto-send do after classification is owned by the
build-pilot capability, not this one.

> test: code

### Scenario: build-auto with args classifies as local build pilot

- **GIVEN** composer submit text `/build-auto add pilot that auto-sends next`
- **WHEN** the submit is classified
- **THEN** the submit is a local build-pilot submit in auto mode
- **AND** the classified arguments are `add pilot that auto-sends next`

> test: code

### Scenario: bare build-fast classifies as local build pilot

- **GIVEN** composer submit text `/build-fast`
- **WHEN** the submit is classified
- **THEN** the submit is a local build-pilot submit in fast mode
- **AND** the classified arguments are empty

> test: code

### Scenario: build pilot names are system registry commands

- **GIVEN** the duckboard system command registry
- **WHEN** registry membership is checked for `build-auto` and `build-fast`
- **THEN** both names are system registry commands

> test: code
