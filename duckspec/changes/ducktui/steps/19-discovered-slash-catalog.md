# Discovered slash catalog

Merge harness `CommandsAvailable` into the ducktui slash palette via the shared completion
catalog.

## Prerequisites

- [x] @step slash-and-model-overlays
- [x] @step agent-and-file-event-loop

## Context

Review finding 3: `refresh_slash_list` uses `system_registry()` with an empty discovered
list; `CommandsAvailable` is ignored in the runtime. Duckboard merges discovery into
`chat_commands` on that event.

## Tasks

- [x] 1. Keep discovered harness commands on shell or runtime state; apply
         `AgentEvent::CommandsAvailable` into that state

- [x] 2. Build the slash palette with
         `build_completion_catalog(system_registry(), discovered)`; refresh an open slash
         overlay when the catalog updates

- [x] 3. Cover with a unit test that after discovery, workflow/agent names appear in
         palette rows (system entries still present)
