# Kick and arm on submit

Wire `dispatch_user_submit` for `LocalBuildPilot`: arm the mode, send the rewritten kick
as user and agent text, and refuse unsupported scopes without arming.

## Prerequisites

- [x] @step pilot-state-module-and-reactivate-setting

## Tasks

- [x] 1. Implement `kick_command` and `join_slash_args` from scope / lifecycle head

- [x] 2. Handle `LocalBuildPilot` in `dispatch_user_submit` (arm + `send_agent_turn`, or
         no-op)

- [x] 3. @spec chat/build-pilot Kick and arm: Exploration kick rewrites to ds-explore with args

- [x] 4. @spec chat/build-pilot Kick and arm: Change kick uses lifecycle head with args

- [x] 5. @spec chat/build-pilot Kick and arm: User bubble shows rewritten kick not build command

- [x] 6. @spec chat/build-pilot Kick and arm: Successful kick arms the requested mode

- [x] 7. @spec chat/build-pilot Kick and arm: Unsupported scope does not arm or start agent turn
