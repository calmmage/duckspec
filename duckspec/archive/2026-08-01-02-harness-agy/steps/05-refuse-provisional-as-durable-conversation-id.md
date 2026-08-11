# Refuse provisional as durable conversation id

On a successful print turn without a recovered AGY conversation uuid, do not treat the
provisional open id as durable resume identity.

## Prerequisites

- [x] @step cold-print-session-lifecycle-and-batch-answer
- [x] @step login-shell-wrap-for-inner-agy

## Context

Review finding 2: `run_prompt` falls back to `open_id` (`pending-*`) when log parse and
`last_conversations.json` both miss. The host then persists that id and later passes
`--conversation pending-…`, which is not a valid AGY uuid.

## Tasks

- [x] 1. After a successful `agy -p`, require a recovered conversation id (log and/or
         `last_conversations` fallback); if none, return a typed process/session error
         instead of rebinding to the provisional id

- [x] 2. Ensure a successful path still rebinds provisional → durable uuid when the log
         (or cache) supplies one (existing happy-path tests stay green)

- [x] 3. Add a regression test: scripted peer succeeds on stdout but writes no
         conversation line / no cache hit → error, no durable `sessionId` rebind to
         `pending-*`

- [x] 4. Confirm resume / session-not-found tests still pass under the new rule
