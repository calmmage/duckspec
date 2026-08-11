# Shared Ideas paint order and restore digit resolver

Unify Ideas list + digit ordering on one pure path; restore `keybind_list_digit` and all
`@spec` tests so audit is green.

## Prerequisites

- [x] @step painted-rows-and-digit-resolver
- [x] @step wire-selection-dispatch

## Context

Steps 01–02 implemented the feature once; the working tree later lost `keybind_list_digit`
and its tests while `main` still dispatches digits. Design requires Ideas list and digits
to share one pure paint-order helper (no parallel walk).

## Tasks

- [x] 1. Restore `ListDigitAction` + `keybind_list_digit` in `keybinds.rs` (gates
         unchanged)

- [x] 2. Ensure `painted_live_queue_ids` / State list-digit helpers +
         `dispatch_list_digit` / Ctrl+1/2/3 wire still compile

- [x] 3. Extract pure Ideas idea-path paint-order helper (expanded sections, nested ideas,
         no tag chrome); drive digit index from it

- [x] 4. Refactor Ideas list walk so view order for ideas derives from that same helper
         (no parallel sort/tag-expand for digits alone)

- [x] 5. @spec shell/list-digit-switch Chord and eligibility: Change area with project resolves digit

- [x] 6. @spec shell/list-digit-switch Chord and eligibility: Ideas area with project resolves digit

- [x] 7. @spec shell/list-digit-switch Chord and eligibility: Other area does not resolve

- [x] 8. @spec shell/list-digit-switch Chord and eligibility: Modal or rename capture does not resolve

- [x] 9. @spec shell/list-digit-switch Chord and eligibility: Eligible with chat focused in Change or Ideas

- [x] 10. @spec shell/list-digit-switch Painted-row index: Change nth row matches live-queue order

- [x] 11. @spec shell/list-digit-switch Painted-row index: Ideas skips collapsed sections

- [x] 12. @spec shell/list-digit-switch Painted-row index: Ideas includes nested painted rows in order

- [x] 13. @spec shell/list-digit-switch Painted-row index: Out-of-range digit selects nothing

- [x] 14. @spec shell/list-digit-switch Selection without rename side effects: Digit selects a different live-queue row

- [x] 15. @spec shell/list-digit-switch Selection without rename side effects: Digit selects a different idea row

- [x] 16. @spec shell/list-digit-switch Selection without rename side effects: Already-selected exploration is no-op without rename
