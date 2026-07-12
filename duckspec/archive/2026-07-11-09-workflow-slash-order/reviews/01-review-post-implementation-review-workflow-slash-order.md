# Post-implementation review: Workflow slash order

Reviewed proposal → design → `chat/slash-commands` deltas → discovery/sort code and tests.
Behavior matches the decision record; audit 8/8; remaining gaps are low-cost coverage, not
structural defects.

## Scope

Artifacts under `duckspec/changes/workflow-slash-order/` (proposal, design, spec/doc
deltas, both steps), `crates/duckchat` discovery + `SlashCommand`, `crates/duckboard`
`slash_commands` / `filter_commands`, installable `ds-*.md` frontmatter, and
`ds audit workflow-slash-order`. Stage: post-implementation, full chain to code.

## Summary

```
| # | sev | lens | title | → next |
| --- | --- | --- | --- | --- |
| 1 | minor | quality | Help Workflow sort lacks None-last coverage | ignore |
| 2 | minor | quality | Equal order-key name tie-break untested | ignore |
```

## Findings

### 1. Help Workflow sort lacks None-last coverage - quality/minor

**Where:** `Local system submit` prose in `spec.delta.md`; `build_system_help_body` /
`append_kind_section` in `crates/duckboard/src/slash_commands.rs:114`; only tested with
two ordered keys

**Why:** Spec requires Workflow entries without an order key after ordered ones in
`/help`. Code path uses `slash_order_rank` (same as completion), but a regression on the
help branch alone would not fail CI.

**Action:** Optional one test mirroring the completion None-last scenario; safe to ignore
if you prefer not to expand coverage.

### 2. Equal order-key name tie-break untested - quality/minor

**Where:** Kind cues + Local system submit (equal keys → name ascending);
`filter_commands` and help sort both implement `.then_with(|| name)`

**Why:** Tie-break is specified and implemented but never asserted; low freeze risk given
simple `Ord` chain.

**Action:** Optional unit test with two Workflow rows same `order_key`, reversed names; or
ignore.

## Verdict

**Archive-ready.** Frontmatter parse is strict as designed, descriptions land on rows,
`/ds` empty/`ds`/`ds-` queries keep equal fuzzy scores so lifecycle order shows, and help
Workflow listing follows order keys. No soundness or fidelity blockers. Two minor coverage
nits only.
