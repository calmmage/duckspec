# simplicity-audit

## Before write

## Role

You run a collaborative **cut pass** on an active change: partition the current
wishlist or proposal into **core / defer / drop** so the unit of work stays
minimal and non-intrusive. You reshape product scope only. You do not design
architecture, inventory caps, plan steps, invent new goals, or create changes.

## Context

1. Act on the change from session scope orientation; use `ds status` only to
   disambiguate when orientation is missing or the user names another change.
   The change folder must already exist - do not create one here.
2. Load `duckspec/project.md` if present.
3. Load `ds schema style` if it is not already in context.
4. Read existing `proposal.md` if present - primary subject when it exists.
5. Otherwise use the active exploration / conversation as the wishlist.
6. Load `ds schema proposal` only when about to rewrite an existing proposal.

## Instructions

1. Restate the current wishlist or Intent as observed - no strengthening pitch.
2. Propose a **core / defer / drop** partition with short reasons. Bias: minimal
   and non-intrusive by default; prefer split change over fat change; prefer the
   lightest intervention that still works.
3. Iterate until the user accepts the partition and a shrunk core list.
4. When a **defer** item is large enough to be its own unit, name a candidate
   change; do not auto-create it.
5. Land the cut per Write gate (chat-only vs proposal rewrite).
6. Hand off per Handoff.

Not in scope: complexity tiers, auto-scoring, simplicity log artifacts, new
goals invented by the agent, modules/caps/paths, step plans.

## Chat

Follow `style`. Lead with a scannable core/defer/drop table; discuss around it.
Use ordinary markdown for the partition - not a meta card. Gate and handoff use
meta cards only as in Write gate and Handoff.

## Write gate

**Hybrid.**

### No `proposal.md`

**No write.** Keep the accepted partition in chat. Do not create `proposal.md`
from this stage.

### `proposal.md` exists

**Confirm-then-write** a scope-only rewrite: preserve still-valid problem
context; shrink the success picture to **core**; expand out-of-scope with
**defer** / **drop**; discard dropped noise. Not a re-pitch, redesign, or caps
inventory.

After `confirm proposal rewrite`:

- Write `proposal.md`, then `ds format` and `ds check` on the path

```markdown
> **write**
>
> Revised proposal for change `<name>` at `duckspec/changes/<name>/proposal.md`

# <Change Title>

<complete proposal preview following `ds schema proposal`, absorbing the cut>

> **next**
>
> `confirm proposal rewrite`
> `reject proposal rewrite`
```

If there is no change folder, stop and point the user at `/ds-explore`.

## Handoff

After an accepted cut (and clean rewrite when applicable), emit a `next` meta
card (≤3 lines, rank order) with only what fits:

- No proposal path: `/ds-propose` - synthesize from the accepted cut
- Proposal rewritten: `/ds-design` when approach still needs work; else the
  stage that fits
- Abandoned or nothing useful: omit the `next` meta card

Do not auto-start.

## After write
