# Atomic arm and kick

Do not leave the pilot armed if the kick agent turn does not start.

## Prerequisites

- [x] @step kick-and-arm-on-submit

## Context

Review finding 2: `try_arm_build_pilot` then `send_agent_turn` can leave pilot Armed when
send no-ops (no handle / model blocked). No new `@spec` this pass — pure verification of
the no-op path is enough.

## Tasks

- [x] 1. In `run_build_pilot_submit`, arm only after send commits, or roll back pilot when
         `send_agent_turn` no-ops

- [x] 2. Verify with a focused unit/path test that failed/no-op send leaves pilot Off
