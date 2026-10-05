# Mid-turn auto-answer requires handle

Only settle mid-turn pilot answer when in-band `answer_user_choice` runs (review finding
3).

## Context

Review 01 finding 3: `try_pilot_auto_answer_user_choice` settles transcript even when
`agent_handle` is missing. Fail closed: leave awaiting without fake settle.

## Tasks

- [x] 1. In `try_pilot_auto_answer_user_choice`, do not settle if `agent_handle` is
         missing; leave awaiting

- [x] 2. Adjust existing armed mid-turn tests for the no-handle leave-awaiting case and/or
         provide a path that proves settle when handle answers

- [x] 3. @spec chat/build-pilot Mid-turn choice auto-answer: Armed pilot auto-selects first chip when none marked recommended

- [x] 4. @spec chat/build-pilot Mid-turn choice auto-answer: Armed pilot auto-selects recommended-marked chip over first
