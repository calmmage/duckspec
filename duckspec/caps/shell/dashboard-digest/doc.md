# Dashboard digest

How the open project’s Dashboard left column answers “what next?” with short, ranked lists
instead of a full inventory dump — without changing the audit panel or repo artifacts.

## Sections and density

With a project open, the left column carries three inventories only: **Changes**,
**Explorations**, and **Archived**. Ideas stay in the Ideas area; there is no separate
Stuck bucket. The right-hand audit column is a different job and is not filtered here.

Resting density is **filter law**: at most three rows, then an “… N more” control that
expands that section alone. When a section has three or fewer items, every row shows and
the overflow control is absent. **New Exploration** always sits under the exploration list
and never consumes a shortlist slot.

**Archived** starts collapsed (header only). After expand it uses the same three-row
shortlist on newest-first order, without the day-seeded third slot used for live work.

Expand/collapse is UI state only. Opening another project resets all three sections to
resting density so each project lands calm.

## Ranking

Order is computed for presentation; nothing is written into `duckspec/` or idea
frontmatter.

```
Changes
  attention = max(shallow change mtime, latest chat for change name)
  order     = attention desc → needs_work desc → name asc
  needs_work = partial steps and/or validation errors

Explorations (live only)
  order = latest chat for exploration id → id timestamp fallback

Archived
  order = newest-first (owned with archive browse); no heat formula
```

Shallow mtime is the change directory and its immediate children, gathered on project
load/refresh. Chat activity is the latest signal for that scope in the app session store.

## Day-seeded 2+1 cut

When Changes or Explorations is at rest and has more than three ranked items:

```
heat slots     = ranked[0], ranked[1]
rotation slot  = day_seeded_pick(ranked[2..])   // local YYYY-MM-DD seed
shortlist      = heat + rotation
```

Same local day and same ranked inputs always produce the same shortlist. Expanding the
section shows the **full ranked list**, not only the three resting members. Archived never
uses day rotation.

## Boundaries

```
| In | Out |
| --- | --- |
| Dashboard left column density and order | Worlds / multi-project digest |
| Derived scores in the UI | Heat fields in repo artifacts |
| 2+1 shortlist for live sections | Dashboard search |
| | Ideas or Stuck sections on Dashboard |
```
