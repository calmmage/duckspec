# Post-implementation review: prose option next-actions

Implementation matches design for trailing A–B–C → next-action ghost. One spec-hygiene fix
remains (stale `@spec` line anchors). Last-`next` (not EOF-only) was fixed during review.
Ready for a short step then archive.

## Scope

Reviewed change `prose-option-next-actions`: proposal, design, mainline
`chat/default-prompts` and `chat/meta-cards` (plus focus-answer wording),
`crates/duckcore/src/prose_options.rs`, `meta_card.rs` last-next extraction, `duckboard`
`default_prompts` / `focus_answer`, `ducktui` `recompute_next_hints`, unit tests,
`ds check` / `ds audit prose-option-next-actions` (green). No `steps/` directory
(implement-first). Working tree also holds unrelated dirty paths outside this change.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Stale `@spec` line anchors in default-prompts | Retarget all drifted duckboard anchors in that cap | /ds-step |
| 2 | No steps after full implementation | Accept implement-first; no retro steps | — |
| 3 | Table-row ABC untested | Skip; bullets are the product path | — |
| 4 | Next card required EOF (footer killed ghost) | Use last `next` card, not end-of-message-only | done in review |
```

## Findings

### 1. Stale `@spec` line anchors in default-prompts

**Where:** `duckspec/caps/chat/default-prompts/spec.md` →
`crates/duckboard/src/default_prompts.rs` (and pre-existing drift in the same file).

**Evidence:** After inserting the prose-ABC list test, several next-action and related
scenario anchors point a few lines away from the test `fn` (e.g. empty session seeds
`:244` vs `fn` at 248; empty submit `:374` vs 394). Audit remains green (soft line
matching). Prose-options anchors in `prose_options.rs` were accurate at write time.

**Impact:** Jump-to-spec / backlink navigation misses the test; contracts and behavior
still align by scenario name.

**Discussion:** Full retarget of this cap’s duckboard anchors (A) is cheap and cleans the
file we own for this change. Touching only new scenarios (B) leaves older drift. Accepting
(C) is low product risk but weakens `@spec` hygiene.

**Resolution:** **A** — retarget drifted `default_prompts.rs` anchors in
`chat/default-prompts` (full pass on that file’s duckboard markers).

**Next:** `/ds-step` — update line anchors to current test `fn` lines; format and check
the cap.

### 2. No steps after full implementation

**Where:** `duckspec/changes/prose-option-next-actions/` (proposal + design only; no
`steps/`).

**Evidence:** Code and mainline cap edits already landed without an apply trail.

**Impact:** Weaker historical package; not a runtime defect. Audit does not require steps
for green status.

**Discussion:** Retro thin steps (A) document what shipped; full ceremonial steps (C) are
noise. Accept implement-first (B) matches how this change was built under time pressure.

**Resolution:** **B** — accept no `steps/`.

**Next:** none (process only).

### 3. Table-row ABC untested

**Where:** Design “optional crude leading `|`”; `parse_option_line` table tolerance; no
dedicated scenario.

**Evidence:** Bullet and bare `A —` paths have tests. GFM table body rows can parse letter
+ multi-cell body, but nothing asserts that shape.

**Impact:** Rare product path — agents usually end with bullet A–B–C, not table-only
trailing options. Mid-message tables do not arm the composer (trailing-run rule).

**Discussion:** Add a table scenario (A) is nice-to-have. Narrow design claim (B) is doc
honesty without code. Accept (C) prioritizes the bullet path.

**Resolution:** **C** — skip table-row coverage for this change.

**Next:** none.

### 4. Next card required EOF (footer killed ghost)

**Where:** `duckcore::trailing_next_actions` / focus open region; agent pattern `next`
card then footer (“Send `confirm design`…”).

**Evidence:** Old rule required the card to end at the last non-empty line. Footer prose
cleared next-actions. Observed in UI with `confirm design` / `reject design` gates.

**Impact:** Highest-frequency write gates failed to arm composer ghost.

**Discussion:** Keep EOF-only (strict) vs last-`next` in message (relaxed). Relaxed
matches real agent output; multiple `next` cards use the last one.

**Resolution:** Last `next` card supplies actions and focus open region; prose after the
card allowed. Landed during review in `meta_card.rs`, `focus_answer.rs`, `chat/meta-cards`
(and focus-answer wording).

**Next:** none (already implemented). Verify in tryout after rebuild.

## Resolved concerns

- Proposal “letter + label + full text” vs design `{Letter} — {body}` — intentional design
  simplification; ghost ≡ send with char-budget clip.

- Recommended-letter initial Tab index — deferred by design (always start at 0).

- Design/code fidelity for meta-then-prose, A–C strict order, ≥2 cap 3, fence skip —
  rechecked; unit tests green.

## Outcome

Product direction and implementation are sound. Remaining work is mechanical `@spec`
anchor retarget in `chat/default-prompts`, then archive when that step is done (and a
quick UI tryout of last-next + ABC ghost is advised).
