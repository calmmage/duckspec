# Pilot state module and reactivate setting

Add `build_pilot` policy and state types, ephemeral `AgentSession.pilot`, and the
config/Settings control for reactivate-on-error (default off; runtime ignore in this cut).

## Prerequisites

- [x] @step system-slash-build-pilot-commands

## Tasks

- [x] 1. Add `crates/duckboard/src/build_pilot.rs` with `PilotState`, mode, and `decide` /
         allowlist helpers

- [x] 2. Add `pilot` field on `AgentSession` (default Off; not session-file persisted)

- [x] 3. Add `ChatConfig.pilot_reactivate_on_error` with Settings checkbox and load/save

- [x] 4. @spec chat/build-pilot Reactivate setting: Default pilot reactivate setting is off

- [x] 5. @spec chat/build-pilot Reactivate setting: Setting can be enabled without reactivating pilot at runtime
