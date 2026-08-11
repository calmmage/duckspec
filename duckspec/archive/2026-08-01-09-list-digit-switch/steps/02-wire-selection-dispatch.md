# Wire selection dispatch

On `KeyPress`, dispatch digit actions to Change/Ideas select without rename side effects.

## Prerequisites

- [x] @step painted-rows-and-digit-resolver

## Tasks

- [x] 1. Wire `Ctrl+1/2/3` in `main` KeyPress (after modal capture; Control, not
         Command-as-primary)

- [x] 2. Dispatch only when selection changes; already-selected exploration must not
         rename

- [x] 3. @spec shell/list-digit-switch Selection without rename side effects: Digit selects a different live-queue row

- [x] 4. @spec shell/list-digit-switch Selection without rename side effects: Digit selects a different idea row

- [x] 5. @spec shell/list-digit-switch Selection without rename side effects: Already-selected exploration is no-op without rename
