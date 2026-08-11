# Archive pending commit

Keep an archived change in the Change list until its residual work is path-scoped
committed, so finish does not stop at duckspec archive.

## Problem

`ds archive` moves the package to `duckspec/archive/` and the Change list immediately
shows it under Archived. Path-scoped commit of that change’s dirt is a later, optional
step (chat handoff). In a multi-change dirty tree it is easy to leave that work
uncommitted and forget it.

Commit-before-archive is the wrong fix: it forces a clean tree before the duckspec
lifecycle is done and fights how people actually work.

## Intent

- After a successful archive, the change **stays in the Change section** of the list until
  that change’s residual dirty work is path-scoped committed (or there is nothing owned to
  commit)

- While it stays there, the row carries clear **pending / uncommitted** chrome so the
  reason is visible without hunting chat history

- Only when owned residual dirt is clean does the row **leave Change and appear only under
  Archived** — UI “fully finished”

- Disk layout is unchanged: archive still lands under `duckspec/archive/…`; the list is
  presentation of finish state, not a second archive store

- Archive remains allowed with a dirty tree; **do not** make commit-before-archive the
  default product rule

- Commit remains human-reviewed (message + path set) — not silent auto-commit on archive
  success

## Non-goals

- Auto-commit without message review

- A separate dual list for “needs commit” (the Change row is the queue)

- Forcing commit before archive

- A native VCS commit executor beyond what design later needs for the plaque

- Perfect multi-tenant membership of every path under `crates/` in the first cut (honest
  owned-dirt rules can start simple and tighten in design)

## Settled direction

```
ds archive
    │
    ▼
folder under duckspec/archive/…     (filesystem, today)
    │
    ▼
Change list: still shown + pending plaque
    │
 path-scoped commit of owned dirt
    │
    ▼
Archived section only               (UI done)
```

**Done for the human** = duckspec archive **and** path-scoped commit of that change’s
residual dirt. Archived in the UI means fully finished, not only duckspec-archived.
