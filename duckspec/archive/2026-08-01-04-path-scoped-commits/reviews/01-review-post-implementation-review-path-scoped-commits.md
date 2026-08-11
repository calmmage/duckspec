# Post-implementation review: Path-scoped commits

Path-scoped archive handoff and VCS workflow priming match the design; two small spec
cleanups remain before archive (drop an untested destructive-ops SHALL; add Settings
exposure coverage).

## Scope

- `proposal.md`, `design.md`
- `caps/archive/path-scoped-commit/{spec,doc}.md`
- `caps/shell/vcs-workflow/{spec,doc}.md`
- steps `01`–`02` (all tasks checked)
- Source: `archive.md` Handoff, `VcsWorkflow` / Settings / `assemble_priming_body`
- Tests: template handoff units, config standing-instruction units, priming inject
- `ds check` ok; `ds audit path-scoped-commits` ok

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Destructive-ops SHALL has no scenario | Drop the untested SHALL from the requirement; product standing text may keep the bullet | `/ds-spec` |
| 2 | Settings exposure claimed but untested | Keep Settings as contract; add scenario + test | `/ds-spec` |
```

## Findings

### 1. Destructive-ops SHALL has no scenario

**Where:** `caps/shell/vcs-workflow/spec.md` — Requirement: Standing instructions

**Evidence:** Requirement lists tool choice, path-scoped commits, no auto-commit / no
invent, worktrees preference (each with scenarios), plus a destructive-ops confirmation
SHALL with no scenario. Standing product text already includes destructive bullets; they
were introduced in Petr's grab-bag commit `xmuvwvoz`
(`feat: exploration rename/retitle, system slash commands, priming
archive`), not as this
change's intent and not by upstream squareduck.

**Impact:** Spec claims more than it pins; a future edit can drop the bullet while tests
stay green, or force unearned scenario maintenance for ambient hygiene.

**Discussion:** Adding a scenario (A) locks protocol the operator does not need for this
change. Leaving a bare SHALL (C) is half-contract noise. Dropping the SHALL (B) keeps the
contract on path-scoping / workflow / no auto-commit while allowing the priming text to
remain as non-contract guidance.

**Resolution:** B — remove the destructive-ops SHALL from the requirement. Product
`standing_instructions` may still mention destructive ops.

**Next:** `/ds-spec` — edit Standing instructions requirement prose in
`shell/vcs-workflow` (and doc only if it over-claims).

### 2. Settings exposure claimed but untested

**Where:** `caps/shell/vcs-workflow/spec.md` — Requirement: Global workflow choice;
`crates/duckboard/src/area/settings.rs` `vcs_section`

**Evidence:** Requirement says Settings SHALL expose the three workflows. Scenarios only
cover default git and config persistence. Settings UI exists and persists via
`VcsWorkflowSelected`; no test pins the three-choice exposure.

**Impact:** Settings could be removed while tests pass; contract over-claims.

**Discussion:** Narrowing the requirement to config-only (B) under-describes a real
product surface the operator uses. Adding scenario + test (A) matches “feature exists →
cover it,” using a cheap pure check (e.g. picker choices = `VcsWorkflow::ALL`, selection
updates config) rather than full GUI automation.

**Resolution:** A — keep Settings SHALL; add scenario and `test: code` coverage.

**Next:** `/ds-spec` — add scenario under Global workflow choice; then `/ds-step` to
implement the test and `@spec` backlink.

## Resolved concerns

- Archive scenarios verified via template text (not a live agent VCS run) remain
  intentional: design chose instruction surface + string tests.

- Mid-lifecycle path-scoping via priming only (no `/ds-commit`) remains intentional.

- Worktrees “plumbing not automatic” is owned by `session-worktrees`, not this change.

## Outcome

Not ready to archive until the two `/ds-spec` items land and the Settings test is applied.
Core path-scoped handoff + VCS priming behavior is accepted.
