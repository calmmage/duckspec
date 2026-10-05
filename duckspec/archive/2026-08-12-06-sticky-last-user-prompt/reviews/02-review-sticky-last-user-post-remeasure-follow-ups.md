# Review: Sticky last-user post-remeasure follow-ups

Follow-up review after steps 03–04. Runtime behavior matches proposal and specs. One
design-table lag (remeasure-without-clear) was found and amended in-session.

## Scope

Proposal, design (post step 04 + in-session amend), `chat/sticky-last-user` spec/doc,
steps 01–04, sticky helpers/view/measure, interaction cache/sync/remeasure flag, main
chrome measure batch, unit tests. `ds audit sticky-last-user-prompt` clean (9/9 linked).
Prior review `01-review-sticky-last-user-post-implementation.md`.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Design measurement row lagged step 04 remeasure-without-clear | Split clear vs remeasure in design; remeasure after rematerialize when source exists | `/ds-design` (done in-session) |
```

## Findings

### 1. Design measurement row lagged step 04 remeasure-without-clear

**Where:** `design.md` Measurement and state; `sync_sticky_last_user` +
`sticky_last_user_remeasure` + `maybe_measure_sticky_last_user`

**Evidence:** Design said rematerialize clears and remeasures **only if** idx/body
changed. Code clears only then, but always sets remeasure when a source exists so layout
measure overwrites the top after auto-collapse without force-hiding.

**Impact:** Runtime correct; design alone under-described remeasure. Cap doc already said
top remeasures when transcript layout changes.

**Discussion:** Amend design (A) vs leave as implementation detail (B). **A** chosen and
applied.

**Resolution:** Design table split into clear vs remeasure columns: clear only on source
change / no source / collapse / focus / session reset; remeasure on every rematerialize
with a source (and on explicit clear paths).

**Next:** `/ds-design` — applied in-session to `design.md`; no further design work
required for this finding

## Resolved concerns

- Review 01 #1 (always clear on materialize) and #2 (width remeasure claim) remain fixed
  in code and design.

- Width reflow stays intentional v1 out of scope.

- Live visual smoke still not required for archive; pure helpers + wiring cover the
  contract.

## Outcome

Ready to archive. No open product findings; design matches selective clear + rematerialize
remeasure-without-clear.
