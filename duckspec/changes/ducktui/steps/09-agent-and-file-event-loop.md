# Agent and file event loop

Wire the live ducktui binary to drive agents and apply external session file events per
design.

## Prerequisites

- [x] @step extract-transcript-segments
- [x] @step session-sharing
- [x] @step chat-pane

## Context

Review finding 1: `AppEvent::Agent` / `Files` are stubs; submit does not spawn
`drive_harness`; no project watcher runs in ducktui.

## Tasks

- [x] 1. Spawn the duckcore project watcher when a project is bound; forward batches as
         `AppEvent::Files`

- [x] 2. Apply session-path file events via `session_sharing` (reload/remove idle
         sessions; shield in-flight)

- [x] 3. On composer submit: persist if driven, spawn `drive_harness`, forward
         `AgentEvent`s as `AppEvent::Agent`

- [x] 4. Apply agent stream events to the session and `ChatPane` with tick cadence per
         design (`chat/stream-ui` shape)

- [x] 5. Wire numbered next/fast-response activation to real session/agent paths where
         applicable
