# Review: Build pilot post-followup recheck

Prior cancel/atomic findings are fixed. One new accepted finding: answer-thrash cancel
must disarm the pilot so `TurnComplete` cannot auto-send after a thrash stop.

## Scope

Re-reviewed after steps 08–09: disarm controls + cancel path, atomic arm/kick,
`TurnComplete` pilot hop, thrash cancel path in `main.rs`, prior review record, `ds check`
/ `ds audit` (clean, 24/24 linked).

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Thrash cancel leaves pilot armed | Disarm pilot when thrash trips and cancels the turn | `/ds-step` |
```

## Findings

### 1. Thrash cancel leaves pilot armed

**Where:** `main.rs` `ContentDelta` thrash branch; `cancel_streaming_main_turn` /
`was_cancel` only on user cancel paths.

**Evidence:** Thrash trip captures draft, cancels the handle, clears priming follow-up,
but does not clear `ax.pilot` or set `cancel_in_flight`. Non-priming `TurnComplete` then
may `maybe_auto_send` while pilot remains armed.

**Impact:** After “Stopped: the assistant kept rewriting…”, pilot can still auto-send the
next hop.

**Discussion:** Spec names user cancel only. Options: (A) disarm on thrash like cancel,
(B) leave as non-user stop, (C) expand contract for system stops. **A** matches product
stop semantics with minimal code.

**Resolution:** Thrash-trip cancel disarms the pilot (and should not pilot-auto-send on
that completion).

**Next:** `/ds-step` — wire thrash path to disarm (reuse cancel helper or set pilot Off +
`cancel_in_flight`); add a focused test if cheap.

## Resolved concerns

- Review 01 F1 (user cancel disarm) and F2 (atomic arm/kick) verified present in current
  code.

- Pure-helper test strategy remains intentional for v1.

## Outcome

Almost archive-ready. One small step to disarm pilot on thrash cancel, then re-check and
archive.
