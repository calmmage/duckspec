# List membership

View-only list split: pending archives in the Change section after actives; Archived and
Dashboard show finished archives only.

## Prerequisites

- [x] @step pending-predicate-helpers

## Tasks

- [x] 1. Pass dirty paths into Change/Archived list builders; filter finished archives for
         `archived_entries` / section presence (Dashboard included)

- [x] 2. Append pending archives to the Change queue after active changes, newest-first
         among pendings

- [x] 3. @spec archive/pending-commit Change list placement: Pending package appears in the Change section

- [x] 4. @spec archive/pending-commit Change list placement: Multiple pending archives order newest-first after actives

- [x] 5. @spec archive/browse Interleaved archived rows: Pending archived package is omitted from Archived lists

- [x] 6. Keep browse scenarios green under finished-only wording (non–idea-owned
         explorations with finished archives; section present when only explorations
         remain)
