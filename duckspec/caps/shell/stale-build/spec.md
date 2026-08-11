# Stale build plaque

When the open project is duckboard itself and the workspace package version on disk is
ahead of the running binary’s baked version, the shell surfaces a quiet Update plaque and
a manual install recipe — never an automatic rebuild or restart.

When the open project is duckboard itself and the workspace is ahead of the running binary
— by package version or by source fingerprint — the shell surfaces a quiet Update plaque
and a manual install recipe — never an automatic rebuild or restart.

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
> - crates/duckboard/src/self_version.rs:277

### Scenario: Duckboard package tree is eligible for evaluation

- **GIVEN** a project root with a `crates/duckboard` package named `duckboard`
- **AND** a readable workspace package version on disk
- **AND** that disk version is ahead of the running binary version
- **WHEN** stale-build is evaluated
- **THEN** a stale-build signal is produced

> test: code
> - crates/duckboard/src/self_version.rs:290

## Requirement: Ahead comparison

The running package version SHALL be the binary’s baked package version. The disk package
version SHALL be the root `Cargo.toml` workspace package version when present, otherwise
the root package version. A version-ahead signal SHALL be produced when both versions
parse as `major.minor.patch` triples and the disk triple is strictly greater than the
running triple.

The running source fingerprint SHALL be the value baked into the binary at compile time
(empty when unavailable). The disk source fingerprint SHALL be derived from the open
project root with the same rules used at bake time: prefer a jj working-copy change id,
else a short git commit id; append a dirty marker when the working copy has local changes.
A source-ahead signal SHALL be produced when both fingerprints are non-empty and unequal.
When either fingerprint is empty, source comparison SHALL NOT produce a signal by itself.

A stale-build signal SHALL be produced when a version-ahead signal or a source-ahead
signal is present. Equal package versions alone SHALL NOT suppress a source-ahead signal.
A lower disk package version, or any unreadable or unparsable package version, SHALL NOT
produce a version-ahead signal. Unreadable or unavailable source fingerprints SHALL NOT
produce a source-ahead signal.

> test: code

### Scenario: Higher disk version is stale

- **GIVEN** an eligible duckboard project
- **AND** a disk package version that is strictly greater than the running package version
- **WHEN** stale-build is evaluated
- **THEN** a stale-build signal is produced
- **AND** the signal carries the running and disk package versions

> test: code
> - crates/duckboard/src/self_version.rs:291

### Scenario: Equal versions are not stale

- **GIVEN** an eligible duckboard project
- **AND** a disk package version equal to the running package version
- **AND** matching or unavailable source fingerprints on both sides
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal

> test: code
> - crates/duckboard/src/self_version.rs:303

### Scenario: Lower disk version is not stale

- **GIVEN** an eligible duckboard project
- **AND** a disk package version lower than the running package version
- **AND** matching or unavailable source fingerprints on both sides
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal

> test: code
> - crates/duckboard/src/self_version.rs:315

### Scenario: Unreadable disk version is not stale

- **GIVEN** an eligible duckboard project
- **AND** a root manifest with no usable package version
- **AND** matching or unavailable source fingerprints on both sides
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal

> test: code
> - crates/duckboard/src/self_version.rs:324

### Scenario: Differing source fingerprints are stale at equal version

- **GIVEN** an eligible duckboard project
- **AND** a disk package version equal to the running package version
- **AND** a non-empty running source fingerprint
- **AND** a different non-empty disk source fingerprint
- **WHEN** stale-build is evaluated
- **THEN** a stale-build signal is produced
- **AND** the signal’s display strings include the source fingerprints

> test: code
> - crates/duckboard/src/self_version.rs:334

### Scenario: Missing source fingerprint skips source comparison

- **GIVEN** an eligible duckboard project
- **AND** equal package versions
- **AND** an empty running source fingerprint or an unavailable disk source fingerprint
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal from source comparison

> test: code
> - crates/duckboard/src/self_version.rs:355

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
> - crates/duckboard/src/self_version.rs:367

### Scenario: Recipe escapes single quotes in the path

- **GIVEN** a project root path that contains a single quote

- **WHEN** the install recipe command is formed

- **THEN** the embedded single quote is escaped so the `cd` argument remains one shell
  word

> test: code
> - crates/duckboard/src/self_version.rs:375

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
- **THEN** it shows the running and disk display strings from the signal
- **AND** it shows the install recipe command
- **AND** it offers a control that copies that command to the clipboard

> manual: recipe panel chrome in the iced shell

### Scenario: Panel dismisses without installing

- **GIVEN** an open recipe panel
- **WHEN** the panel is closed
- **THEN** the panel is no longer shown
- **AND** no build or install process is started

> manual: recipe panel chrome in the iced shell
