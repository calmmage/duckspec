# Discover AGY models live

Replace the static AGY success path with `agy models` parsing (display name as id), and
keep the curated table as the empty/fail fallback.

## Prerequisites

- [x] @step parse-grok-advertised-default

## Tasks

- [x] 1. Add a pure parser for `agy models` stdout (`id<TAB>display`, skip status lines)

- [x] 2. Spawn `agy models` from `AgyProvider::list_models` with the same bin wrap as
         print mode

- [x] 3. Fall back to the curated table when spawn or parse yields no models

- [x] 4. @spec harness/agy Offered AGY models: Live discovery lists display names from agy models

- [x] 5. @spec harness/agy Offered AGY models: Empty or failed discovery falls back to the curated table
