# Simplicity audit

Add a thin `/ds-simplicity-audit` workflow skill that cuts early wishlist bloat into core
/ defer / drop before a proposal freezes Intent — defaulting changes to minimal and
non-intrusive for multi-user and fork-friendly work.

## Problem

Exploration is deliberately open: ideas dump freely. That is useful, but thoughts often
run past the real unit of change. The first draft of Intent then absorbs a multi-feature
wishlist. Non-goals and late review (“Simple” / quality) exist, but they are soft or late:
sprawl is cheap to add early and expensive to undo after design and caps.

This matters more when more than one person (or fork) must accept the change. Large
surface area raises review cost, merge risk, and “do I want this whole bundle?” friction.
The unit of change should default small enough that a second person can take it without
swallowing the whole brain-dump.

## Direction

Ship **one** new agent template skill: `/ds-simplicity-audit`.

```
explore (dump OK)
    →  /ds-simplicity-audit  (cut: core / defer / drop)
    →  /ds-propose           (Intent + Non-goals absorb the cut)
```

Role of the skill:

- **Cut pass**, not a second design stage and not architecture work

- Bias: **minimal / non-intrusive by default**; prefer split change over fat change;
  prefer lightest intervention that still works

- Agree what is **core** for this change, what to **defer**, what to **drop**

- Leave explore open for brainstorming; the audit is the deliberate edge before (or when
  reshaping) the decision record

- Re-runnable when Intent has already grown fat mid-change

How cuts land (product intent, not file layout):

- Prefer the **proposal** as the durable home of the cut (shrunk success picture + clearer
  out-of-scope)

- **Hybrid write (settled):** no `proposal.md` → chat-only cut, handoff `/ds-propose`;
  `proposal.md` exists → confirm-then-rewrite the proposal (scope edit only)

- v1 does **not** require a new review-style log artifact or a new lifecycle phase

- Late review simplicity remains a safety net, not the primary control

Trigger posture:

- **Opt-in** skill; explore offers it by **judgment** when sprawl signals are present
  (multi-outcome, “also / while we’re here”, no single success picture, multi-user/fork
  concern) — not ranked on every create-change

- Propose may cut in chat or offer audit before gating a wishlist

- Not a forced gate on every exploration

- Not three parallel “weight class” workflows

## Rejected for this change

```
| Idea | Why not |
| --- | --- |
| light / medium / heavy design (or lifecycle) routes | Excessive surface; label inflation (“everything is medium”); debates the tier instead of the cut list |
| Separate `/ds-simplify` plus `/ds-simplicity-audit` | One skill is enough for v1; audit can include agreeing and applying cuts into the decision boundary |
| Mandatory complexity tier on every design | Fights thin-template norms; overspecifies process |
| Primary control only via soft propose/explore voice | Too easy to skip when the dump already ran hot |
| New mandatory append-only simplicity log in v1 | Extra ceremony; defer unless multi-user history proves necessary |
```

## Boundaries

In scope:

- New (or revised) agent template(s) and any minimal wiring so `/ds-simplicity-audit` is a
  real workflow command

- Guidance for when to offer it and what a good cut looks like

- Alignment with existing propose / non-goals / review “simple” posture without replacing
  those stages

Out of scope:

- Complexity scale product UX (chips, tiers, multi-route design trees)
- Redesigning explore into a forced funnel
- Duckboard-only polish as a prerequisite for the skill existing
- Automatic machine scoring of “how complex is this change?”
