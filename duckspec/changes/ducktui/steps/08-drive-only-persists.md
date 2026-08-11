# Drive-only persists

Route every duckboard session write through the drive-aware session-sharing policy so
displayed sessions are never written.

## Prerequisites

- [x] @step session-sharing

## Context

Review finding 3: `persist_session_snapshot` and dirty flush are gated, but many
`chat_store::save_session` call sites still write unconditionally.

## Tasks

- [x] 1. Inventory all `chat_store::save_session` call sites in duckboard

- [x] 2. Ensure each path that represents a local turn calls `mark_driven` (or equivalent)
         before persist

- [x] 3. Route persists through `persist_driven` / gated helpers; displayed-only paths
         must not write

- [x] 4. Keep in-flight flush and turn-boundary durability behavior; re-run
         session-sharing and related duckboard tests
