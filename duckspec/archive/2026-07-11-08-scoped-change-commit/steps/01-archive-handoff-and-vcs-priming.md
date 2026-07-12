# Archive handoff and VCS priming

Rewrite archive handoff for path-scoped commit, align VCS standing instructions, and cover
both with unit tests.

## Tasks

- [x] 1. Rewrite `crates/duckspec/content/templates/archive.md` Handoff for path set +
         scoped commit + empty-set / no invent

- [x] 2. Extend `VcsWorkflow::standing_instructions` (Git / Jj / Worktrees) with
         change-only path-scoped commit rules

- [x] 3. Unit-test standing instructions mention path-scoped / change-only commit for each
         workflow

- [x] 4. Unit-test archive template handoff requires path set, scoped commit, and
         empty-set handling
