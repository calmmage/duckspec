# @ AGY harness

## @ Requirement: Offered AGY models

Models offered by the AGY harness SHALL be tagged with the AGY harness identity so the
picker and dispatch can distinguish them from other harnesses' models. Listing models
SHALL use the live `agy models` catalog when that command yields a non-empty list, with
each listed model's display name as its id. When live discovery is empty or fails, listing
SHALL fall back to the curated table.

> test: code

### + Scenario: Live discovery lists display names from agy models

- **GIVEN** a successful `agy models` listing that includes display names
- **WHEN** the AGY harness lists models
- **THEN** each returned model id is the corresponding display name
- **AND** each returned model names the AGY harness

> test: code

### + Scenario: Empty or failed discovery falls back to the curated table

- **GIVEN** live `agy models` discovery that is empty or fails
- **WHEN** the AGY harness lists models
- **THEN** the listed models are the curated fallback table
- **AND** each returned model names the AGY harness

> test: code
