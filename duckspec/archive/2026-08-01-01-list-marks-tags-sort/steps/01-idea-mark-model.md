# Idea mark model

Add exclusive `IdeaMark` and star pin time on idea frontmatter with cycle/apply helpers
and unit tests.

## Tasks

- [x] 1. Add `IdeaMark` enum and `mark` / `favored_at` fields to `Frontmatter` in
         `crates/duckboard/src/idea_store.rs` with backward-compatible serde defaults

- [x] 2. Implement `cycle_mark` and `apply_mark` (refresh pin time on enter star; clear
         when leaving star)

- [x] 3. Map unknown YAML mark values to `None` on load

- [x] 4. @spec ideas/marks Exclusive mark on the idea: Mark cycles through none, star, hot, cool

- [x] 5. @spec ideas/marks Exclusive mark on the idea: Mark persists across idea reload

- [x] 6. @spec ideas/marks Exclusive mark on the idea: Unknown stored mark loads as none

- [x] 7. @spec ideas/marks Star pin time: Entering star records a pin time

- [x] 8. @spec ideas/marks Star pin time: Leaving star clears pin time

- [x] 9. @spec ideas/marks Star pin time: Re-entering star refreshes pin time
