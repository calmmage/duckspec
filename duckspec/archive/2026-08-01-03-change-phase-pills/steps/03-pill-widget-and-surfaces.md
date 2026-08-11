# Pill widget and surfaces

Shared phase-pill widget, list trailing and above-composer placement gated by settings,
and click routing into the target session via `send_prompt_text`.

## Prerequisites

- [x] @step phase-display-model

## Tasks

- [x] 1. Add `crates/duckboard/src/widget/phase_pill.rs` (`view_pill` / `view_pair`) with
         soft stage tints and tooltip hover; export from `widget.rs`

- [x] 2. When `config.ui.phase_pill_list`, attach phase pills via `ListRow::trailing` on
         exploration, active, and archived rows in `area/change.rs` `view_list`

- [x] 3. When `config.ui.phase_pill_chat`, show phase-pill strip above the composer for
         change/exploration scopes in `widget/agent_chat.rs` (not caps/codex)

- [x] 4. Wire `PhasePillSend` (or equivalent) through change-area messages and `main`:
         select target scope if needed, then `interaction::send_prompt_text` with
         activation text (mirror streaming queue/no-op policy of ordinary submit)

- [x] 5. Spot-check list pills, composer strip, toggles off, lifecycle click, and
         uncommitted Commit send (manual scenarios)

- [x] 6. Run `cargo test -p duckboard` for phase-display and config unit tests; fix
         regressions
