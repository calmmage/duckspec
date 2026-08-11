# TurnComplete safe auto-send

After a non-priming `TurnComplete` and next-action refresh, auto-send a safe rank-1 token
or fully disarm the pilot.

## Prerequisites

- [x] @step kick-and-arm-on-submit

## Tasks

- [x] 1. Hook non-priming `TurnComplete` to pilot decide and `send_agent_turn` or disarm

- [x] 2. Skip priming turns and never auto-send when the pilot is off

- [x] 3. @spec chat/build-pilot Safe auto-send: Rank-1 confirm auto-sends while armed

- [x] 4. @spec chat/build-pilot Safe auto-send: Rank-1 allowlisted stage slash auto-sends in auto mode

- [x] 5. @spec chat/build-pilot Safe auto-send: Rank-1 ds-review auto-sends only in auto mode

- [x] 6. @spec chat/build-pilot Safe auto-send: Rank-1 ds-review disarms in fast mode without sending

- [x] 7. @spec chat/build-pilot Safe auto-send: Rank-1 archive codex or verify disarms without sending

- [x] 8. @spec chat/build-pilot Safe auto-send: Missing or unsafe rank-1 disarms without sending

- [x] 9. @spec chat/build-pilot Disarm controls: Disarm stays off until build command relaunch
