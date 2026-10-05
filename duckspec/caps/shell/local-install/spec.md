# Local install

Developer install from this repo refreshes cargo bins and the macOS Applications app from
the current tree so Dock and Finder launch the build that was just installed.

## Requirement: Cargo bins and Applications app

`just install` from the project root SHALL install the workspace CLI tools into the cargo
bin directory and SHALL place a `Duckboard.app` bundle at `/Applications/Duckboard.app`
built from the same tree. The Applications deploy SHALL follow a successful release app
bundle assembly for this project (including ACP agent sibling binaries inside the bundle).
The recipe SHALL build release binaries once for both cargo bins and the app bundle (no
separate recompile per package). The recipe SHALL NOT start or restart a running duckboard
process.

> test: code

### Scenario: Install recipe deploys cargo bins and Applications

- **GIVEN** the project root justfile install recipe

- **WHEN** the recipe’s install targets are inspected

- **THEN** the recipe builds release binaries once and installs ds, duckboard, and
  duckchat-claude-acp into the cargo bin directory

- **AND** the recipe assembles the macOS app bundle from that same build

- **AND** the recipe replaces `/Applications/Duckboard.app` with that bundle

> test: code
> - crates/duckboard/src/self_version.rs:369
