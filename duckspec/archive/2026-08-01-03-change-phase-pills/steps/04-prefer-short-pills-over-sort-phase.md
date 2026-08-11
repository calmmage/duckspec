# Prefer short pills over Sort Phase

When `ui.phase_pill_list` is on, hide Sort “Phase” and keep long-phase pillows off so
short clickable pills own list phase chrome (review finding 2).

## Prerequisites

- [x] @step pill-widget-and-surfaces

## Context

From `reviews/01-review-post-implementation-review.md` finding 2: dual prefs
(`list.show_phase_pillows` vs `ui.phase_pill_list`). Resolution B — when list phase pills
are enabled, Sort “Phase” must not control a parallel long-phase face.

## Tasks

- [x] 1. Thread `phase_pill_list` into Change list sort-menu controls
         (`sort_menu_controls` in `area/change.rs`); when true, omit or disable the Phase
         toggle (same for Ideas sort menu if it shares the same dual-pref UX)

- [x] 2. In `main` (and Ideas if needed), no-op `ToggleListPhasePillows` while
         `config.ui.phase_pill_list` is true so long-phase cannot be re-enabled under
         short pills

- [x] 3. Confirm list rows already suppress long phase when short pills are on; fix any
         path where `show_phase_pillows` alone can still show long phase beside short
         pills

- [x] 4. Spot-check Sort menu with phase pills on/off; run any focused tests added for
         pure helpers
