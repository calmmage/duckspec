# Dashboard digest and filter law

Make the open project’s Dashboard a short, heat-aware attention surface instead of a full
inventory dump — filter law (≤3 + “… N more”) and derived ranking, single project only.

## Why

Duckboard already opens one project at a time. The Dashboard is the navigation hub: active
changes, explorations, archive, quick stats. As that inventory grows, the default view
stops being a place to *decide what to do next* and becomes a scroll of everything that
exists.

An exploration of plaintask showed the same attention problem solved with a few reusable
laws: always filter views (shortlist + overflow count), never surface the whole backlog by
default, and rank by *attention heat* derived from recent signals rather than a stored
priority field. Those laws transfer cleanly **inside one open project**. Worlds,
cross-project digests, global discovery, and non-dev project kinds do not — they belong to
a multi-project life OS, not to duckspec’s filesystem-per-repo model.

## Intent

```
today                         target
─────                         ──────
full active-change list  ──►  shortlist (≤3) + "… N more"
full idea / exploration  ──►  heat-ranked shortlist
archive as dump          ──►  searchable / expand-on-demand
(no ranking signal)      ──►  heat from recent activity
```

Dashboard should answer “what deserves attention in **this** project now?”:

- **Active work** — changes (and explorations) ranked by recent activity / progress
  signals

- **Capture queue** — hot ideas when that surface is part of the Dashboard story

- **Stuck / stale** — work that has gone quiet or is incomplete, so cold items still
  surface sometimes

Default density follows **filter law**: a small shortlist per section plus an explicit
overflow affordance. Expanding or searching can reveal the full set; the resting view
stays short.

**Heat is derived**, not a new artifact field under `duckspec/`. Signals come from things
the app already knows or can observe cheaply (e.g. recent chat activity, recent file
edits, step progress). No multi-project index and no new source-of-truth store beyond
app-local data already outside the committed tree if needed for timestamps.

## Boundaries

```
| In | Out (explicit) |
| --- | --- |
| Single open project Dashboard | Worlds / grouping many repos |
| Shortlists + “… N more” | Cross-project daily 2×2 digest |
| Derived heat ranking | Stored priority / heat fields in repo artifacts |
| Optional light rotation so backlog still appears | Full plaintask discovery (disk, vault, GitHub) |
| | Non-dev / mixed project kinds in duckpond |
| | Chat auto-discovery / review-quota systems (later slices) |
```

Filesystem remains source of truth for duckspec artifacts. This change improves **how
duckboard presents** the open project; it does not invent a parallel task DB.

## Relationship to the wider exploration

Plaintask features were ranked for transfer. This change takes only the **highest-ROI,
in-project** slice: filter law + project digest (+ heat as the ranking engine).
Nominate-only inbound ideas, chat/session hygiene, soft start gates, and review quotas
remain separate candidates — not part of this proposal’s success criteria.

## Open questions

- **Rotation in v1:** is day-seeded (or session-seeded) rotation required, or is a pure
  heat shortlist enough until something feels stuck on “always the same three”?

- **Section set:** exact Dashboard sections (changes / explorations / ideas / stuck) vs a
  minimal “active changes first” cut — design can narrow; proposal only requires at least
  active work + overflow discipline.
