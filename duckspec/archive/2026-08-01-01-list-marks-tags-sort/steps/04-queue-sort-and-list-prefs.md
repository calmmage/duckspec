# Queue sort and list prefs

Pure queue sort ladder (top-3 star pin + sort keys), last-message activity helper, and
shared `ListConfig` preferences.

## Prerequisites

- [x] @step idea-mark-model

## Tasks

- [x] 1. Add `queue_list` (or equivalent) helpers: `QueueRowMeta`, `SortKey`, `sort_queue`
         with pin prefix + ordinary segment + title tie-break

- [x] 2. Implement last non-priming message activity scan/cache per chat scope

- [x] 3. Define stable phase ladder ordering aligned with `change_scope_facts` phases

- [x] 4. Add `ListConfig` to `crates/duckboard/src/config.rs` (sort key, show type/phase
         pillows; defaults)

- [x] 5. @spec ideas/queue-list Star pin before ordinary sort: Up to three newest stars pin above non-pinned rows

- [x] 6. @spec ideas/queue-list Star pin before ordinary sort: A fourth star follows ordinary sort among non-pinned rows

- [x] 7. @spec ideas/queue-list Star pin before ordinary sort: Pin ranking uses pin time newest-first

- [x] 8. @spec ideas/queue-list Ordinary sort keys: Default key orders by last non-priming message time newest-first

- [x] 9. @spec ideas/queue-list Ordinary sort keys: Missing activity sorts after rows with activity under last-message key

- [x] 10. @spec ideas/queue-list Ordinary sort keys: Phase key orders by lifecycle phase then title

- [x] 11. @spec ideas/queue-list Ordinary sort keys: Created key orders by creation time

- [x] 12. @spec ideas/queue-list List preferences and sort menu: Sort key preference is shared by Change and Ideas lists

- [x] 13. @spec ideas/queue-list List preferences and sort menu: Type and phase pillow visibility prefs default on and are toggled from the menu
