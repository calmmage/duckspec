# Local install

Developer install from this repo refreshes cargo bins and the macOS Applications app from
the current tree so Dock and Finder launch the build that was just installed.

## Requirement: Cargo bins and Applications app

`just install` from the project root SHALL install the workspace CLI tools into the cargo
bin directory and SHALL place a `Duckboard.app` bundle at `/Applications/Duckboard.app`
built from the same tree. The Applications deploy SHALL follow a successful release app
bundle assembly for this project (including the Claude ACP agent sibling binary inside the
bundle). The recipe SHALL NOT start or restart a running duckboard process.

> test: code

### Scenario: Install recipe deploys cargo bins and Applications

- **GIVEN** the project root justfile install recipe
- **WHEN** the recipe’s install targets are inspected
- **THEN** the recipe installs duckspec, duckboard, and duckchat-claude-acp via cargo
- **AND** the recipe assembles the macOS app bundle
- **AND** the recipe replaces `/Applications/Duckboard.app` with that bundle

> test: code
