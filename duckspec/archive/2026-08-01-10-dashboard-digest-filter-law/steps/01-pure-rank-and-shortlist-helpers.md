# Pure rank and shortlist helpers

Add a pure duckboard helper (rank changes/explorations, day-seeded 2+1 cut) with unit
tests; no Dashboard UI yet.

## Tasks

- [x] 1. Add pure rank/shortlist module under `crates/duckboard/src/` with injected
         timestamps/needs_work — no `duckspec/` writes

- [x] 2. @spec shell/dashboard-digest Change ranking: Newer attention ranks above older

- [x] 3. @spec shell/dashboard-digest Change ranking: Needs-work breaks recency ties

- [x] 4. @spec shell/dashboard-digest Change ranking: Missing activity sorts colder than known activity

- [x] 5. @spec shell/dashboard-digest Exploration ranking: Newer chat activity ranks explorations

- [x] 6. @spec shell/dashboard-digest Exploration ranking: Missing chat falls back to id timestamp

- [x] 7. @spec shell/dashboard-digest Exploration ranking: Archived exploration omitted from Explorations

- [x] 8. @spec shell/dashboard-digest Day-seeded shortlist cut: Resting shortlist is two heat plus one spun tail

- [x] 9. @spec shell/dashboard-digest Day-seeded shortlist cut: Same day and inputs yield the same shortlist

- [x] 10. @spec shell/dashboard-digest Day-seeded shortlist cut: Different days may spin a different tail member

- [x] 11. @spec shell/dashboard-digest Day-seeded shortlist cut: Expand shows full ranked list

- [x] 12. @spec shell/dashboard-digest Derived-only scores: Ranking leaves duckspec free of heat fields
