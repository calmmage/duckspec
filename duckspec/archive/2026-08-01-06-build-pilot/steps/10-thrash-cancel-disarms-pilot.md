# Thrash cancel disarms pilot

On answer-thrash trip, disarm pilot so the cancelled completion cannot pilot-auto-send.

## Prerequisites

- [x] @step cancel-disarms-pilot

## Context

Review 02: thrash path cancels the handle but leaves pilot armed and does not set
`cancel_in_flight`, so `TurnComplete` may still `maybe_auto_send`. No new `@spec` this
pass — step-only fix with a focused path test.

## Tasks

- [x] 1. On thrash trip in `main.rs` (or shared helper), disarm pilot and mark
         cancel-in-flight like user cancel (`cancel_streaming_main_turn` or equivalent)

- [x] 2. Ensure `TurnComplete` after thrash does not pilot-auto-send (via `was_cancel` /
         pilot Off)

- [x] 3. Focused unit/path test: thrash-style cancel leaves pilot Off and blocks auto-send
