# Simplicity audit - Design

Stock `/ds-simplicity-audit`: collaborative core/defer/drop cut pass. Content-only
(template + harness wrappers + small explore/propose handoff nudges).

## Flow

```
explore ─sprawl?─► /ds-simplicity-audit ─┬─ no proposal ─► chat cut ─► /ds-propose
         └─tight──► /ds-propose            └─ has proposal ► rewrite proposal (gated)
```

Opt-in. Not a lifecycle phase, tier system, schema, cap, or duckboard feature.

## Cut model

```
| Bucket | Meaning |
| --- | --- |
| core | Must succeed for *this* change |
| defer | Later / split candidate |
| drop | Discard |
```

Spine: restate wishlist → propose partition → iterate until accepted. Bias: minimal,
non-intrusive; prefer split change; lightest intervention. Only cut/reshape product scope
— no new goals, architecture, caps, or steps. Large defer → name a candidate change; do
not auto-create.

## Writes

```
| Situation | Behavior |
| --- | --- |
| No `proposal.md` | No write. Handoff `/ds-propose` from the accepted cut. |
| Has `proposal.md` | Confirm-then-write shrunk proposal (scope edit only); format + check. |
```

## Handoffs

- **Explore:** default `/ds-propose`. If sprawl (multi-outcome, “also/while we’re here”,
  no single success picture, multi-user/fork concern), rank audit then propose.

- **Propose:** before gating a wishlist, cut in chat or offer audit. Post-write handoff
  unchanged.

- **Audit:** no proposal → `/ds-propose`; proposal rewritten → `/ds-design` (or fitting
  stage).

## Packaging

```
crates/duckspec/content/
  templates/simplicity-audit.md          NEW
  commands/{claude,opencode}/ds-simplicity-audit.md  NEW
  commands/codex/ds-simplicity-audit/SKILL.md        NEW
  templates/explore.md                   handoff nudge
  templates/propose.md                   don’t rubber-stamp wishlist
```

- `ds template simplicity-audit`; slash `/ds-simplicity-audit`
- Wrapper pattern = other `ds-*` commands; frontmatter `order: 1.5`
- Install via existing `ds init <harness>`; rebuild `ds` for embed

## Non-goals

New artifact/schema/cap; complexity tiers; mandatory gate; simplicity log; duckboard-only
registry; separate `/ds-simplify`.
