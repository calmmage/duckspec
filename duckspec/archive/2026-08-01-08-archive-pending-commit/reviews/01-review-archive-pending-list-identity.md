# Review: Archive pending list identity

Implementation of predicate, list split, and uncommitted chrome matches the design; one
identity gap remains — pending packages still present as finished Archive for breadcrumbs
and selection reveal.

## Scope

Reviewed `archive-pending-commit` proposal, design, `archive/pending-commit` +
`archive/browse` deltas, all three steps, duckboard list helpers/UI in
`crates/duckboard/src/area/change.rs` and Dashboard dirty wiring, `@spec`-linked unit
tests, and working-tree diff for this change. `ds check` and
`ds audit archive-pending-commit` were clean; related unit tests passed.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Pending uses finished-archive list identity | Change-list presentation while pending: breadcrumbs `Changes / <id>`, reveal `picker` | /ds-step |
```

## Findings

### 1. Pending uses finished-archive list identity

**Where:** `duckspec/changes/archive-pending-commit/design.md` (Identity);
`crates/duckboard/src/area/change.rs` breadcrumbs (~1382–1389) and `SelectChange` section
expand (~630–636)

**Evidence:** Design requires breadcrumbs while pending in Change: `Changes / <id>`, with
selection/tabs/overview/chat still matching archived package identity. Code treats any
name in `project.archived_changes` as list-Archived: breadcrumb root is `Archive`, and
select expands the `archived` section. Pending rows live only in the Change (`picker`)
section, so that reveal targets the wrong collapsible. Artifact paths
(`prefix = archive/…`), chat, and no lifecycle next-stage are already correct.

**Impact:** The finish-queue signal is inconsistent (Change row + `uncommitted` vs Archive
breadcrumb). If Change is collapsed, plaque/`Commit` selection can expand Archived and
leave the selected row hidden.

**Discussion:** (A) Pending-aware list presentation — breadcrumb `Changes`, reveal
`picker`, keep archive tabs/chat. (B) Drop the design breadcrumb exception and keep
Archive chrome. (C) Spec then implement A. A is the smallest fidelity fix against
already-settled design; B weakens the queue cue; C is optional ceremony.

**Resolution:** Option A — while pending, use Change-list presentation for breadcrumbs and
section reveal; keep archive identity for tabs, overview, and chat.

**Next:** `/ds-step` — plan and implement pending-aware breadcrumb root and `picker`
expand on select; add a focused test if useful (optional thin scenario later).

## Resolved concerns

- **Owned dirt only under `duckspec/archive/<id>/`** — intentional first-cut design;
  proposal non-goal defers code-path membership.

- **Chrome tests are pure helpers / message shape** — activation reuses `PhasePillSend` →
  `handle_list_phase_pill_send` (select + send, no VCS).

- **Pending omitted from Dashboard** — design places the queue on the Change list;
  Archived UIs share finished-only helpers.

## Outcome

Not ready to archive. Core pending-commit behavior is in place; fix list identity for
pending packages via `/ds-step`, then re-check before archive.
