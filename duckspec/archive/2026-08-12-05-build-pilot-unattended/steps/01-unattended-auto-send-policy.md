# Unattended auto-send policy

Implement N1 denylist-based decide and R3 recommended promotion in `build_pilot` (and pure
helpers as needed); retarget existing Safe auto-send tests.

## Tasks

- [x] 1. Flip `is_auto_safe` / `decide` to hard-denylist model (keep Fast `/ds-review`
         rule)

- [x] 2. Promote recommended next-action to rank-1 before decide (label/`recommended` /
         Recommendation letter)

- [x] 3. Retarget existing `@spec` backlinks from Safe auto-send → Unattended auto-send

- [x] 4. @spec chat/build-pilot Unattended auto-send: Rank-1 confirm auto-sends while armed

- [x] 5. @spec chat/build-pilot Unattended auto-send: Rank-1 allowlisted stage slash auto-sends in auto mode

- [x] 6. @spec chat/build-pilot Unattended auto-send: Rank-1 ds-review auto-sends only in auto mode

- [x] 7. @spec chat/build-pilot Unattended auto-send: Rank-1 ds-review disarms in fast mode without sending

- [x] 8. @spec chat/build-pilot Unattended auto-send: Rank-1 archive codex or verify disarms without sending

- [x] 9. @spec chat/build-pilot Unattended auto-send: Missing rank-1 disarms without sending

- [x] 10. @spec chat/build-pilot Unattended auto-send: Rank-1 freeform decision token auto-sends while armed

- [x] 11. @spec chat/build-pilot Unattended auto-send: Rank-1 prose option line auto-sends while armed

- [x] 12. @spec chat/build-pilot Unattended auto-send: Recommended option is auto-sent over earlier non-recommended
