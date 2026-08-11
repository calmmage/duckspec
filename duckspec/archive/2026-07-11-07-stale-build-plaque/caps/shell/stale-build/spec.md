# Stale build plaque

When the open project is duckboard itself and the workspace package version on disk is
ahead of the running binary’s baked version, the shell surfaces a quiet Update plaque and
a manual install recipe — never an automatic rebuild or restart.

## Requirement: Self-project only

Stale-build evaluation SHALL run only when the open project is duckboard itself: the
project root has a parseable `Cargo.toml`, a `crates/duckboard/Cargo.toml` package whose
name is `duckboard`. For any other project, evaluation SHALL produce no stale-build
signal.

> test: code

### Scenario: Non-duckboard project yields no stale signal

- **GIVEN** a project root that is not a duckboard package tree
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal

> test: code

### Scenario: Duckboard package tree is eligible for evaluation

- **GIVEN** a project root with a `crates/duckboard` package named `duckboard`
- **AND** a readable workspace package version on disk
- **AND** that disk version is ahead of the running binary version
- **WHEN** stale-build is evaluated
- **THEN** a stale-build signal is produced

> test: code

## Requirement: Ahead comparison

The running version SHALL be the binary’s baked package version. The disk version SHALL be
the root `Cargo.toml` workspace package version when present, otherwise the root package
version. A stale-build signal SHALL be produced only when both versions parse as
`major.minor.patch` triples and the disk triple is strictly greater than the running
triple. Equal versions, a lower disk version, or any unreadable or unparsable version
SHALL produce no signal.

> test: code

### Scenario: Higher disk version is stale

- **GIVEN** an eligible duckboard project
- **AND** a disk version that is strictly greater than the running version
- **WHEN** stale-build is evaluated
- **THEN** a stale-build signal is produced
- **AND** the signal carries the running and disk version strings

> test: code

### Scenario: Equal versions are not stale

- **GIVEN** an eligible duckboard project
- **AND** a disk version equal to the running version
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal

> test: code

### Scenario: Lower disk version is not stale

- **GIVEN** an eligible duckboard project
- **AND** a disk version lower than the running version
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal

> test: code

### Scenario: Unreadable disk version is not stale

- **GIVEN** an eligible duckboard project
- **AND** a root manifest with no usable package version
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal

> test: code

## Requirement: Install recipe command

The install recipe command SHALL be `cd '<project-root>' && just install`, with the
project root single-quoted for a POSIX shell (embedded single quotes escaped). The command
SHALL NOT invoke build or install itself.

> test: code

### Scenario: Recipe uses just install under the project root

- **GIVEN** a project root path
- **WHEN** the install recipe command is formed
- **THEN** the command is a `cd` into that root followed by `just install`
- **AND** the path is single-quoted for POSIX

> test: code

### Scenario: Recipe escapes single quotes in the path

- **GIVEN** a project root path that contains a single quote

- **WHEN** the install recipe command is formed

- **THEN** the embedded single quote is escaped so the `cd` argument remains one shell
  word

> test: code

## Requirement: Update plaque when stale

While a stale-build signal is present for the open project, the shell SHALL show a
clickable **Update** plaque in the status bar. While no signal is present, the plaque
SHALL NOT appear. Opening the plaque SHALL open the recipe panel; it SHALL NOT start a
build, install, or process restart.

> manual: status-bar chrome in the iced shell

### Scenario: Plaque visible only when stale

- **GIVEN** an open project with a stale-build signal
- **WHEN** the status bar is shown
- **THEN** an Update plaque is visible
- **AND** when no stale-build signal is present the plaque is not visible

> manual: status-bar chrome in the iced shell

### Scenario: Plaque opens the recipe panel without installing

- **GIVEN** a visible Update plaque
- **WHEN** the plaque is activated
- **THEN** the recipe panel opens
- **AND** no build or install process is started

> manual: status-bar chrome in the iced shell

## Requirement: Recipe panel

The recipe panel SHALL show the running and disk versions, instruct the user to quit
duckboard before installing, show the install recipe command, and offer a control that
copies that command to the clipboard. Closing the panel SHALL dismiss it without running
the command. If the stale-build signal clears while the panel is open, the panel SHALL
close.

> manual: recipe panel chrome in the iced shell

### Scenario: Panel shows versions and copyable recipe

- **GIVEN** an open recipe panel for a stale-build signal
- **WHEN** the panel is shown
- **THEN** it shows the running and disk versions
- **AND** it shows the install recipe command
- **AND** it offers a control that copies that command to the clipboard

> manual: recipe panel chrome in the iced shell

### Scenario: Panel dismisses without installing

- **GIVEN** an open recipe panel
- **WHEN** the panel is closed
- **THEN** the panel is no longer shown
- **AND** no build or install process is started

> manual: recipe panel chrome in the iced shell
