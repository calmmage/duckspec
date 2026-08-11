# Review: Build pilot post-implementation

Three findings accepted: cancel must disarm the pilot (spec + step), arm/kick must be
atomic (step), pure-helper tests accepted for v1 (no contract change).

## Scope

Reviewed `proposal.md`, `design.md`, `caps/chat/build-pilot` + `chat/slash-commands`
deltas, all seven steps, `build_pilot.rs`, `slash_commands` (duckcore), `dispatch` /
Esc-Esc / `TurnComplete` pilot wiring, settings/config, plaque chrome, promotion test, and
`ds check` / `ds audit` (clean, 23/23 linked).

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Non–Esc-Esc cancel leaves pilot armed | Disarm on every main-turn cancel; Esc-Esc remains documented stop | `/ds-spec` |
| 2 | Arm before send can leave pilot armed without kick | Keep arm and kick atomic (arm only if kick turn starts, or roll back) | `/ds-step` |
| 3 | Spec scenarios tested mostly via pure helpers | Accept pure tests for v1; dogfood for wiring | — |
```

Finding 3 has no corrective stage (intentional accept). Primary route is earliest layer
among 1–2: **`/ds-spec`**.

## Findings

### 1. Non–Esc-Esc cancel leaves pilot armed

**Where:** `interaction.rs` `CancelPressed`; Esc-Esc path disarms then cancels; `main.rs`
`TurnComplete` still runs `maybe_auto_send` when pilot is armed.

**Evidence:** Esc-Esc clears `ax.pilot` then dispatches cancel. Plain `CancelPressed`
(composer cancel, empty-enter interrupt cancel) does not. After the cancelled turn
completes, non-priming `TurnComplete` can auto-send rank-1 next.

**Impact:** User believes the run stopped; pilot may continue the scenario automatically.

**Discussion:** Spec only mandates Esc-Esc disarm. Options: (A) disarm on every cancel,
(B) document cancel ≠ pilot stop, (C) skip pilot hop only when `cancel_in_flight` (pilot
stays armed). **A** matches stop semantics.

**Resolution:** Disarm whenever the user cancels the main turn; keep Esc-Esc as
documented; add scenario coverage that cancel disarms and does not auto-send after that
completion.

**Next:** `/ds-spec` — extend Disarm controls (and scenario), then `/ds-step` to implement
and test.

### 2. Arm before send can leave pilot armed without kick

**Where:** `run_build_pilot_submit` / `try_arm_build_pilot` then `send_agent_turn`
(`interaction.rs`).

**Evidence:** Pilot is set `Armed` before `send_agent_turn`, which may no-op (no handle /
model not allowed).

**Impact:** Plaque on and pilot armed with no kick turn in edge cases.

**Discussion:** (A) arm only after send commits or roll back on no-op, (B) leave, (C)
redefine arm as intent-only. **A** preserves arm+kick joint contract.

**Resolution:** Arm and kick are atomic from the user-visible path.

**Next:** `/ds-step` — fix `run_build_pilot_submit` (and a focused test if useful).

### 3. Spec scenarios tested mostly via pure helpers

**Where:** `build_pilot` unit tests for kick / auto-send; wiring in `dispatch` /
`TurnComplete` lightly covered by product code only.

**Evidence:** Kick and safe-auto-send `@spec` cases assert pure helpers; audit links
resolve; full iced path not simulated.

**Impact:** Wiring regressions possible while policy tests stay green.

**Discussion:** (A) add integration tests, (B) accept pure tests for v1, (C) rewrite specs
as policy-only. **B** chosen for this pass.

**Resolution:** Pure-helper coverage is accepted for v1; dogfood for wiring; revisit if
needed.

**Next:** none (no corrective stage this review).

## Resolved concerns

- Silent refuse on caps/codex matches “no arm, no turn.”

- Queue flush before pilot hop is intentional priority; pilot re-evaluates on a later
  `TurnComplete`.

- Plaque as chrome-only is correct.

## Outcome

Not archive-ready. Amend **Disarm controls** in `/ds-spec`, then plan/implement cancel
disarm + atomic arm/kick via `/ds-step`. Finding 3 needs no follow-up stage unless dogfood
demands tighter tests.
