# Dashboard live sections filter-law

Wire ranked shortlists, “… N more” / show less, New Exploration outside N, expand flags +
reset on project switch for Changes and Explorations; keep section set.

## Prerequisites

- [x] @step attention-signal-gathering

## Tasks

- [x] 1. Extend `dashboard::State` with expand flags, messages, and reset on project open

- [x] 2. Rank + shortlist Changes and Explorations in the left panel; overflow row; full
         list when expanded

- [x] 3. Keep New Exploration always below explorations, outside N

- [x] 4. @spec shell/dashboard-digest Dashboard section set: Left column sections are Changes Explorations and Archived only

- [x] 5. @spec shell/dashboard-digest Dashboard section set: Ideas and Stuck sections are absent

- [x] 6. @spec shell/dashboard-digest Filter-law shortlist: More than three changes shows three rows and remainder count

- [x] 7. @spec shell/dashboard-digest Filter-law shortlist: Three or fewer changes omits overflow chrome

- [x] 8. @spec shell/dashboard-digest Filter-law shortlist: Expand and collapse restore full list and shortlist

- [x] 9. @spec shell/dashboard-digest Filter-law shortlist: New Exploration stays outside the shortlist slots

- [x] 10. @spec shell/dashboard-digest Expand state lifecycle: Project switch resets expand flags
