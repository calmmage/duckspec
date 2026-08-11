# Archive handoff path-scoped commit

Lock `archive.md` Handoff to the path-scoped commit contract and cover all archive
scenarios with template unit tests + `@spec` backlinks.

## Context

Behavior was largely shipped under archived `scoped-change-commit` without caps. This step
records and verifies the archive handoff contract only — no duckboard commit executor.

## Tasks

- [x] 1. Confirm stock `crates/duckspec/content/templates/archive.md` Handoff still states
         include-set membership (dirty ∩ this change; ambiguous → ask; never whole-tree
         default), pre-commit visibility (message + include set), nonempty `` `commit` ``
         offer, empty-set / no invent, path-scoped execute, and no auto-commit — edit only
         if gaps

- [x] 2. Extend `archive_handoff_requires_path_scoped_commit` in
         `crates/duckspec/src/cmd/template.rs` (or sibling unit tests) so each scenario
         below is falsifiable from the handoff text

- [x] 3. @spec archive/path-scoped-commit Change-owned include set: Include set is dirty paths that belong to this change

- [x] 4. @spec archive/path-scoped-commit Change-owned include set: Ambiguous membership never defaults to the whole dirty tree

- [x] 5. @spec archive/path-scoped-commit Pre-commit visibility: Message and include set are shown before any VCS write

- [x] 6. @spec archive/path-scoped-commit Commit offer and empty set: Nonempty include set offers commit

- [x] 7. @spec archive/path-scoped-commit Commit offer and empty set: Empty include set reports no owned dirt and does not invent a commit

- [x] 8. @spec archive/path-scoped-commit Path-scoped execution: On commit, only the include set is committed

- [x] 9. @spec archive/path-scoped-commit Path-scoped execution: Handoff never auto-commits without user commit
