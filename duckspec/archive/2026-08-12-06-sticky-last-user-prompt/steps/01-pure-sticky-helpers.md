# Pure sticky helpers

Add pure source, body, visibility, and max-height helpers for sticky last-user, with unit
tests for every scenario.

## Tasks

- [x] 1. Add `last_non_priming_user` / source resolution over transcript blocks

- [x] 2. Add sticky body-text helper from the source block

- [x] 3. Add top-edge visibility helper (`content_top`, `offset_y`, source present;
         missing offset as 0; reuse `VIEWPORT_TOP_EPS`)

- [x] 4. Add sticky max-height helper (six content line heights)

- [x] 5. @spec chat/sticky-last-user Source identity: Last non-priming User wins over priming and later non-user segments

- [x] 6. @spec chat/sticky-last-user Source identity: Priming-only transcript has no source

- [x] 7. @spec chat/sticky-last-user Source identity: User-choice answer is not the source

- [x] 8. @spec chat/sticky-last-user Visibility: Content top above offset shows sticky

- [x] 9. @spec chat/sticky-last-user Visibility: Content top at or below offset hides sticky

- [x] 10. @spec chat/sticky-last-user Visibility: No source hides sticky

- [x] 11. @spec chat/sticky-last-user Visibility: Missing offset treated as zero

- [x] 12. @spec chat/sticky-last-user Mirror body: Body equals the source User segment text

- [x] 13. @spec chat/sticky-last-user Height cap: Maximum height is six content line heights
