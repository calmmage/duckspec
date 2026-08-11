# Cancel disarms pilot

Disarm on every main-turn cancel so a cancelled completion cannot pilot-auto-send.

## Prerequisites

- [x] @step esc-esc-disarm-pilot

## Tasks

- [x] 1. Disarm `ax.pilot` in `CancelPressed` (and any other main-turn cancel path that
         does not already go through Esc-Esc)

- [x] 2. Skip pilot `maybe_auto_send` when the completed turn was user-cancelled
         (`cancel_in_flight` / equivalent), or ensure disarm-before-complete makes it a
         no-op

- [x] 3. @spec chat/build-pilot Disarm controls: User cancel of main turn disarms without pilot auto-send on that completion
