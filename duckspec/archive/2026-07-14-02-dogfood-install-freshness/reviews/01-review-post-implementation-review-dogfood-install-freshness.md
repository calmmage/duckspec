# Post-implementation review: dogfood install freshness

Full chain to code. Install→Applications and source-ahead plaque match intent; VCS
fingerprint logic is duplicated and the quit warning is post-hoc.

## Scope

`proposal.md`, `design.md`, `caps/shell/local-install`, `caps/shell/stale-build` deltas,
steps 01–02, `justfile` install, `crates/duckboard/build.rs`,
`crates/duckboard/src/self_version.rs`, plaque widget (display-only). Post-implementation;
`ds audit` clean.

## Summary

```
| # | sev | lens | title | → next |
| --- | --- | --- | --- | --- |
| 1 | major | quality | Fingerprint VCS logic duplicated in build.rs and runtime | /ds-step |
| 2 | minor | fidelity | Install warns to quit after deploy, not before | /ds-step |
```

## Findings

### 1. Fingerprint VCS logic duplicated - quality/major

**Where:** `crates/duckboard/build.rs` and `crates/duckboard/src/self_version.rs`
(`jj_change_id` / dirty / git fallback)

**Why:** Bake-time and disk-time fingerprints must stay identical; two copies will drift
(dirty command, template, fallback order) and silently break the plaque.

**Action:** One shared resolver (`include!` or small module both call); single place for
jj/git + dirty rules.

### 2. Install warns to quit after deploy, not before - fidelity/minor

**Where:** `justfile` install recipe; design risk claims “quit first”

**Why:** Printed message runs after `rm`/`cp`; if the app is open, replace fails with a
late “quit” note. Design/plaque already say quit first.

**Action:** Echo quit-before-replace at the start of install (keep final reopen note).

## Verdict

**Accept with small follow-up.** Behavior matches proposal; not blocked on craft. Dedup
fingerprints before freezing if you want zero dual-source risk; quit-message order is
cheap polish. Long dirty-session false-fresh remains an explicit non-goal.
