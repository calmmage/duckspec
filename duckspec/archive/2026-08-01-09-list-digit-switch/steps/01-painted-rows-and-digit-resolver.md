# Painted rows and digit resolver

Expose Change/Ideas painted-row sequences and resolve `Ctrl+1/2/3` targets in `keybinds`.

## Tasks

- [x] 1. Extract or share pure painted-row order for Change (`ordered_live_queue`) and
         Ideas (expanded-section flat walk, nested rows included)

- [x] 2. Add `ListDigitAction` + `keybind_list_digit(state, n)` (gates: project, area,
         modals/rename, range)

- [x] 3. @spec shell/list-digit-switch Chord and eligibility: Change area with project resolves digit

- [x] 4. @spec shell/list-digit-switch Chord and eligibility: Ideas area with project resolves digit

- [x] 5. @spec shell/list-digit-switch Chord and eligibility: Other area does not resolve

- [x] 6. @spec shell/list-digit-switch Chord and eligibility: Modal or rename capture does not resolve

- [x] 7. @spec shell/list-digit-switch Chord and eligibility: Eligible with chat focused in Change or Ideas

- [x] 8. @spec shell/list-digit-switch Painted-row index: Change nth row matches live-queue order

- [x] 9. @spec shell/list-digit-switch Painted-row index: Ideas skips collapsed sections

- [x] 10. @spec shell/list-digit-switch Painted-row index: Ideas includes nested painted rows in order

- [x] 11. @spec shell/list-digit-switch Painted-row index: Out-of-range digit selects nothing
