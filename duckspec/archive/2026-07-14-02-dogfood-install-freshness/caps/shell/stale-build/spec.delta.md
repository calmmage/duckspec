# @ Stale build plaque

When the open project is duckboard itself and the workspace is ahead of the running binary
— by package version or by source fingerprint — the shell surfaces a quiet Update plaque
and a manual install recipe — never an automatic rebuild or restart.

## ~ Requirement: Ahead comparison

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

### Scenario: Equal versions are not stale

- **GIVEN** an eligible duckboard project
- **AND** a disk package version equal to the running package version
- **AND** matching or unavailable source fingerprints on both sides
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal

> test: code

### Scenario: Lower disk version is not stale

- **GIVEN** an eligible duckboard project
- **AND** a disk package version lower than the running package version
- **AND** matching or unavailable source fingerprints on both sides
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal

> test: code

### Scenario: Unreadable disk version is not stale

- **GIVEN** an eligible duckboard project
- **AND** a root manifest with no usable package version
- **AND** matching or unavailable source fingerprints on both sides
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal

> test: code

### Scenario: Differing source fingerprints are stale at equal version

- **GIVEN** an eligible duckboard project
- **AND** a disk package version equal to the running package version
- **AND** a non-empty running source fingerprint
- **AND** a different non-empty disk source fingerprint
- **WHEN** stale-build is evaluated
- **THEN** a stale-build signal is produced
- **AND** the signal’s display strings include the source fingerprints

> test: code

### Scenario: Missing source fingerprint skips source comparison

- **GIVEN** an eligible duckboard project
- **AND** equal package versions
- **AND** an empty running source fingerprint or an unavailable disk source fingerprint
- **WHEN** stale-build is evaluated
- **THEN** there is no stale-build signal from source comparison

> test: code

## @ Requirement: Recipe panel

### ~ Scenario: Panel shows versions and copyable recipe

- **GIVEN** an open recipe panel for a stale-build signal
- **WHEN** the panel is shown
- **THEN** it shows the running and disk display strings from the signal
- **AND** it shows the install recipe command
- **AND** it offers a control that copies that command to the clipboard

> manual: recipe panel chrome in the iced shell
