# Phase display model

Implement pure `PhaseDisplay` builders over `change_scope_facts` + repo dirty +
exploration emptiness, and cover every derived-display and activation send scenario with
unit tests.

## Prerequisites

- [x] @step config-and-settings-toggles

## Tasks

- [x] 1. Add `PhaseShort`, `VcsPill`, `PhaseDisplay`, and builders
         (`phase_display_for_change`, `phase_display_for_archived`,
         `phase_display_for_exploration`) next to `change_scope_facts` in
         `crates/duckboard/src/area/change.rs` (or sibling module re-exported from change)
         per design

- [x] 2. Map short labels and lifecycle send via `format_lifecycle_command` from the
         existing ladder; long hover from `ChangeScopeFacts.phase`; VCS pill only for
         `ready`/`archived`

- [x] 3. @spec shell/phase-pills Derived stage display: Empty change yields empty short and propose send

- [x] 4. @spec shell/phase-pills Derived stage display: Proposal-only yields proposal short and design send

- [x] 5. @spec shell/phase-pills Derived stage display: Open steps yield steps short and apply send

- [x] 6. @spec shell/phase-pills Derived stage display: Complete steps without review yield ready short and archive send

- [x] 7. @spec shell/phase-pills Derived stage display: No open steps with review yield review short and step send

- [x] 8. @spec shell/phase-pills Derived stage display: Archived yields archived short without lifecycle send

- [x] 9. @spec shell/phase-pills Derived stage display: Active-change long hover is the recognized phase description

- [x] 10. @spec shell/phase-pills Late-stage VCS pill: Ready with dirty tree is uncommitted with Commit send

- [x] 11. @spec shell/phase-pills Late-stage VCS pill: Ready with clean tree is committed without send

- [x] 12. @spec shell/phase-pills Late-stage VCS pill: Pre-ready stage omits VCS pill

- [x] 13. @spec shell/phase-pills Late-stage VCS pill: Archived includes VCS pill from tree dirty state

- [x] 14. @spec shell/phase-pills Exploration display: Empty exploration offers explore short with explore send

- [x] 15. @spec shell/phase-pills Exploration display: Non-empty exploration is explore short without send

- [x] 16. Add a tiny pure activation helper (or assert display send fields) used by click
          routing

- [x] 17. @spec shell/phase-pills Activation: Lifecycle activation submits empty-send next-stage text

- [x] 18. @spec shell/phase-pills Activation: Uncommitted activation submits Commit
