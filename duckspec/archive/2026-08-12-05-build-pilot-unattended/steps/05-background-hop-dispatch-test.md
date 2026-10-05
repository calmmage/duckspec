# Background hop dispatch test

Prove hop dispatch targets the non-active session (review finding 2).

## Context

Review 01 finding 2: linked scenario claims turn start; test only covered
`maybe_auto_send` on a non-active session id.

## Tasks

- [x] 1. Strengthen `hop_after_turncomplete_does_not_require_active_session` (or
         successor) so background session receives `dispatch_pilot_auto_send` / turn-start
         attempt, not only `maybe_auto_send`

- [x] 2. @spec chat/build-pilot Focus independence: Hop after TurnComplete does not require active session
