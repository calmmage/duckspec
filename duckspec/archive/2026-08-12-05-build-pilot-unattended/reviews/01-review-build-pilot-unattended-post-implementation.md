# Review: Build pilot unattended post-implementation

Implementation matches the unattended N1/R3/M1 direction and focus-independent routing;
three follow-ups are needed before archive — Recommendation-letter promotion, a stronger
background-hop test, and fail-closed mid-turn answer without a handle.

## Scope

Proposal, design, `chat/build-pilot` spec/doc deltas, steps 01–03, `build_pilot.rs`,
mid-turn path in `interaction.rs`, TurnComplete hop in `main.rs`, linked tests. `ds check`
and `ds audit build-pilot-unattended` were clean (14/14 linked).

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Recommendation letter promotion missing | Implement letter-line parse and wire into promote/decide | /ds-step |
| 2 | Background hop scenario under-tests turn start | Strengthen test to prove dispatch on non-active session | /ds-step |
| 3 | Mid-turn auto-answer settles without handle | Only settle when in-band answer runs; no fake settle | /ds-step |
```

## Findings

### 1. Recommendation letter promotion missing

**Where:** `design.md` promote-recommended rules; Unattended auto-send in `spec.delta.md`
/ `doc.delta.md`; step 01 task 2; `crates/duckboard/src/build_pilot.rs`
(`promote_recommended` / `is_recommended_mark`)

**Evidence:** Contract requires promoting via option text marks **or** a trailing
Recommendation letter line. Code only scans option send/label for `recommended` /
`(recommended)`. No assistant-body letter parse and no extra input into `decide`.

**Impact:** Agents that only write `**Recommendation:** **B**` without putting
“recommended” on the option line can get the wrong auto-send (first list entry).
Mark-in-option path is covered by tests and works.

**Discussion:** Trim the contract to marks-only (smaller) vs implement letter parse
(matches design and common agent prose). Leave-as-is was rejected as false readiness.

**Resolution:** Implement trailing Recommendation-letter promotion and wire it into the
pilot decide path; add a scenario/test for letter-only recommendation.

**Next:** `/ds-step` — pure parse helper + promote integration + `@spec` coverage

### 2. Background hop scenario under-tests turn start

**Where:** Spec *Focus independence: Hop after TurnComplete does not require active
session*; `hop_after_turncomplete_does_not_require_active_session` in `interaction.rs`

**Evidence:** Product path routes by session key and calls `dispatch_pilot_auto_send`
without using active selection. The linked test only checks `maybe_auto_send` on a
non-active session id; it does not prove a turn is started (or dispatch attempted) for
that session.

**Impact:** Low runtime risk with current `main` routing; medium fidelity risk if a later
change decides hops but fails to dispatch for background sessions while the `@spec` stays
green.

**Discussion:** Soften the scenario to “decide only” vs strengthen the test. Soften was
rejected so the contract’s “turn started” claim stays honest.

**Resolution:** Strengthen the test (or thin observable) so non-active-session hop
dispatch is proven.

**Next:** `/ds-step` — extend test plumbing toward `dispatch_pilot_auto_send` /
turn-started signal on the background session

### 3. Mid-turn auto-answer settles without handle

**Where:** Spec *Mid-turn choice auto-answer*; `try_pilot_auto_answer_user_choice` in
`interaction.rs`

**Evidence:** When pilot is armed, recommended/first option is chosen;
`answer_user_choice` runs only if `agent_handle` is `Some`, but
`settle_user_choice_transcript` always runs and pilot stays armed. Manual chip activation
uses the same optional handle, but pilot auto path never requires a user click.

**Impact:** Rare (handle usually present mid-turn). On miss: UI shows answered, wire may
still await the choice; pilot remains armed and can mis-hop later.

**Discussion:** Fail-closed settle only with wire answer (A); fail_pilot_hop on no handle
(B); leave parity with manual chips (C). A preferred for unattended honesty without always
full-disarming.

**Resolution:** Only settle when the in-band answer is sent; if there is no handle, do not
fake-settle (leave awaiting or equivalent fail-closed behavior).

**Next:** `/ds-step` — guard settle on successful wire answer; tests for no-handle armed
path

## Outcome

Direction and core unattended policy are sound and audit-clean. **Not archive-ready**
until the three `/ds-step` follow-ups land. Primary next: plan and implement those fixes.
