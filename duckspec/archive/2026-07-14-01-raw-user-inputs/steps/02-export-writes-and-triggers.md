# Export writes and triggers

Write the ledger to disk with skip rules, and refresh it after change-scoped durable saves
and after promotion.

## Prerequisites

- [x] @step render-inputs-markdown

## Tasks

- [x] 1. Implement `export_inputs_for_change` (load sessions, render, atomic write if
         changed; skip when no change dir, empty qualifying messages, or unchanged bytes)

- [x] 2. Hook export after durable `save_session` for change-scoped sessions only (single
         chokepoint preferred)

- [x] 3. Call export once after exploration→change `merge_scope` / promotion

- [x] 4. @spec chat/inputs-ledger Empty and missing change: No qualifying messages leaves no ledger file

- [x] 5. @spec chat/inputs-ledger Empty and missing change: Missing change directory skips export

- [x] 6. @spec chat/inputs-ledger Fresh full rebuild: Save refreshes the ledger from all sessions

- [x] 7. @spec chat/inputs-ledger Fresh full rebuild: Unchanged content does not rewrite the file

- [x] 8. @spec chat/inputs-ledger Non-change scopes: Non-change scope save does not create inputs.md under changes

- [x] 9. @spec chat/inputs-ledger Promotion export: Post-promotion export includes migrated user text
