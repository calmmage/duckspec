# Dashboard digest and filter law - Design

Dashboard left column becomes a short, ranked attention surface for the open project:
filter law (N=3 + expand), derived heat ranking, and a day-seeded 2+1 shortlist — audit
column and repo artifacts unchanged.

## Scope and layout

Keep the existing two-column Dashboard shell in `crates/duckboard/src/area/dashboard.rs`.

```
| Panel | Behavior |
| --- | --- |
| **Left — items** | Sections **Changes**, **Explorations**, **Archived** only |
| **Right — audit** | Unchanged (no filter law, no ranking) |
```

Out of this change: Ideas on Dashboard, dedicated Stuck section, worlds, multi-project
digest, inbound discovery, heat fields in `duckspec/`.

```
┌─────────────────────────────┬─────────────────────────┐
│ Changes    [≤3] … N more    │ Audit (as today)        │
│ Explorations [≤3] …         │                         │
│ Archived   (collapsed)      │                         │
└─────────────────────────────┴─────────────────────────┘
```

## Filter-law UX

Shared resting density for left sections:

```
| Rule | Value |
| --- | --- |
| Shortlist size N | **3** for Changes and Explorations |
| Overflow | **“… N more”** row under shortlist; click expands that section only |
| Collapse | Same control toggles **show less** |
| `len ≤ N` | Show all rows; **no** overflow chrome |
| Search | **None** on Dashboard in this change |
| **New Exploration** | Always under the exploration list; **outside** N |
| **Archived** | See two-level state below |
```

### Live sections (Changes / Explorations)

Single expand flag each: at rest → 2+1 shortlist; expanded → full ranked list.

### Archived two-level state

Archive is not a single expand bit. It uses **open** (section body) and **show-all**
(density when open):

```
| State | `archived_open` | `expanded_archived` | What the user sees |
| --- | --- | --- | --- |
| **Collapsed** | false | false (forced) | Header only, e.g. `Archived · K` |
| **Shortlist** | true | false | N newest-first rows + “… M more” |
| **Full** | true | true | All newest-first rows + “show less” |
```

```
Collapsed ──header──► Shortlist ──… more──► Full
   ▲                     │                    │
   └──── close section ──┴── show less ───────┘
```

- Closing the section sets `archived_open = false` and clears `expanded_archived`.
- Shortlist is a **prefix of newest-first** order — **not** day-seeded 2+1.
- Header click toggles open/closed; overflow toggles shortlist/full while open.

### UI state on `dashboard::State`

```rust
expanded_changes: bool,       // default false — shortlist vs full ranked
expanded_explorations: bool,  // default false
archived_open: bool,          // default false — header-only vs body
expanded_archived: bool,      // default false — prefix shortlist vs full (only when open)
chat_activity: HashMap<String, i128>,  // warm scope → latest attention nanos
```

On project switch, reset **all four** expand/open flags and clear `chat_activity`, then
recompute the warm map for the new project so it lands collapsed/resting with fresh
signals.

## Ranking (derived heat)

Nothing is written into `duckspec/` or idea frontmatter. Scores recompute from warm
inputs.

### Changes

```
attention_ts = max(change_shallow_mtime, latest_chat_activity(scope = change.name))
recency      = newer attention_ts ranks higher
needs_work   = (any step Partial ? 1 : 0) + (validation errors > 0 ? 1 : 0)
order        = (recency desc, needs_work desc, name asc)
```

- `StepCompletion::Partial` and validation error counts are the **stuck substitute** (no
  Stuck section).

- Missing mtime/chat ⇒ cold; still ordered by `needs_work` then name.

### Explorations

Live list only (`is_on_live_list`). Order by latest scoped chat activity; fallback:
timestamp embedded in `exploration-{nanos}` id. No step/error term.

### Archived

Newest-first only (existing archive date / reverse folder order). No heat formula;
filter-law density only after the section is open.

## Day-seeded shortlist (2 + 1)

For **Changes** and **Explorations** only, when at rest and `len > 3`:

```
ranked = full rank order
heat   = ranked[0], ranked[1]
spin   = day_seeded_pick(ranked[2..])   // local YYYY-MM-DD seed
shortlist = heat + [spin]
```

```
| Property | Rule |
| --- | --- |
| Seed | Local `YYYY-MM-DD` (host clock) |
| Determinism | Same day + same inputs ⇒ same shortlist |
| Expand (live) | Show **full ranked list** |
| Archived | **No** day rotation — newest-first prefix shortlist only |
| `len ≤ 3` | No spin; show all |
```

Rejected for v1: pure heat with no rotation; session-seeded shuffle. Chosen:
plaintask-style **2 heat + 1 day rotation** for live sections only.

## Signal plumbing

```
ProjectData load / watcher refresh
        │
        ├─ existing: structure, steps, validations
        └─ shallow mtime per active change
              max(change dir, immediate children)

Warm chat activity (dashboard::State.chat_activity)
        │
        ├─ full recompute: project open, project reload / watcher reconcile,
        │                  external session-file events
        └─ upsert: session write / flush paths that advance activity
                   (flush tick, turn-end persist, promotion flush)

Dashboard paint (view_items_panel)
        │
        ├─ READ warm chat_activity map only — no session file I/O
        ├─ rank_changes / rank_explorations (pure helper)
        ├─ shortlist(2 heat + 1 day spin) if !expanded && len > 3
        ├─ archive density from archived_open / expanded_archived
        └─ render rows + overflow / expand
```

```
| Concern | Decision |
| --- | --- |
| Rank/shortlist code | **Pure duckboard helper** (dashboard-adjacent module). Not duckpond |
| Mtime gather | On **project load / watcher refresh**, not every frame |
| Mtime depth | Change directory + **immediate children** only |
| Chat activity formula | Max of session `created_at_nanos` and last message activity; scope = change name or exploration id |
| Chat activity ownership | **Warm map** on `dashboard::State` (`HashMap<String, i128>`) |
| Chat activity refresh | Full recompute on open / reload / external session events; upsert on session write/flush |
| Paint path | Ranking **only reads** the warm map — never `load_sessions_for` / collect on paint |
| Persistence | **None** for scores |
```

Brief lag until the next recompute/upsert is accepted for v1; do not re-fetch sessions
from the paint path to compensate.

## Responsibilities

```
| Component | Owns |
| --- | --- |
| `area/dashboard.rs` | Layout, open/expand state, **warm `chat_activity` map**, messages, wiring shortlist into sections |
| Pure rank/shortlist helper | Deterministic order + 2+1 cut + archive prefix shortlist |
| `ProjectData` (or reload path) | Shallow mtime per active change on load/refresh |
| Chat / exploration store | Read-only activity inputs for recompute/upsert |
| `duckpond` / committed artifacts | **No change** for heat or filter law |
```

## Compatibility and risk

- Empty project / no project: empty state and recent-projects UI unchanged.

- Projects with ≤3 changes/explorations: behavior looks like today plus stable ordering by
  rank.

- Large inventories: resting view stays short; expand is the full-list escape hatch.

- Stale mtime if only deep nested files change: accepted for v1; deepen walk later if
  needed.

- Chat store missing for a scope: cold recency; ranking still defined.

- Warm map briefly behind a just-written session until flush/upsert: accepted for v1.

## Settled choices (session)

```
| Area | Decision |
| --- | --- |
| Sections | Changes / Explorations / Archived; no Ideas; no Stuck section |
| Layout | Audit right column untouched |
| Live expand | One flag each: shortlist vs full |
| Archived | **`archived_open` + `expanded_archived`**; Collapsed → Shortlist → Full |
| Archive shortlist | Newest-first prefix of N; no 2+1 |
| Filter law | N=3; … N more; no Dashboard search |
| Ranking | Recency + needs_work for changes; activity for explorations |
| Rotation | **B** — 2 heat + 1 day-seeded from tail (live only) |
| Plumbing | Derived scores; load-time mtimes; **warm chat activity map**; pure UI helper; flags + map on `dashboard::State` |
| Review amend | Two-level archive state (finding 1); warm map ownership (finding A / review 01 #3) |
```
