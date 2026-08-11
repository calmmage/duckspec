# VCS workflow standing instructions and priming

Lock Git / Jj / Worktrees config, standing text, Settings surface, and first-turn priming;
cover all VCS scenarios with unit tests + `@spec` backlinks.

## Context

`VcsWorkflow`, Settings picker, standing instructions, and priming injection already exist
from the prior cut. This step closes gaps and attaches scenario backlinks for
`shell/vcs-workflow`.

## Tasks

- [x] 1. Confirm `VcsWorkflow` + `standing_instructions` in
         `crates/duckboard/src/config.rs`, Settings picker in
         `crates/duckboard/src/area/settings.rs`, and `assemble_priming_body` in
         `crates/duckboard/src/area/interaction.rs` still match the shell/vcs-workflow
         contract — edit only if gaps

- [x] 2. Extend config / interaction unit tests for auto-commit ban, worktrees dedicated
         worktree preference, and any other unasserted scenario bits from the list below

- [x] 3. @spec shell/vcs-workflow Global workflow choice: Default workflow is plain git

- [x] 4. @spec shell/vcs-workflow Global workflow choice: Selected workflow persists in config

- [x] 5. @spec shell/vcs-workflow Standing instructions: Each workflow names its VCS tool

- [x] 6. @spec shell/vcs-workflow Standing instructions: Each workflow requires path-scoped change-only commits

- [x] 7. @spec shell/vcs-workflow Standing instructions: Each workflow forbids auto-commit and inventing an empty commit

- [x] 8. @spec shell/vcs-workflow Standing instructions: Worktrees workflow prefers a dedicated worktree per parallel change

- [x] 9. @spec shell/vcs-workflow First-turn priming injection: First-turn priming body includes the selected workflow's standing instructions
