# Build pilot unattended - Design

Widen armed-pilot auto-advance so sessions keep hopping without focus or Enter: safe stage
tokens, rank-1 `next` (N1), recommended/first A–B–C (R3), and mid-turn chips (M1)—all
session-keyed and focus-independent.

## Goals and non-goals

```
| In | Out |
| --- | --- |
| Focus-independent hop for armed session S | List/global “background pilot” chrome |
| Auto-send rank-1 next except hard denylist (N1) | Auto freeform with no structured choice |
| Prose/meta A–B–C: recommended else first | Persist pilot across app quit |
| Mid-turn chips: recommended else first (M1) | NLP critical-issue detection |
| Failed hop → disarm + system notice | Redesign auto vs fast allowlist modes as separate products |
```

## Architecture

```
                    session S armed
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
   UserChoiceRequest  TurnComplete     UI focus leave
          │                │                │
   pick chip          refresh next     no cancel /
   (rec/first)        actions          no disarm
          │                │
   answer_user_choice  decide(N1/R3)
          │                │
   stay armed         send or disarm
```

Routing stays by `instance_id/session_id` (`main` AgentEvent path). Agent subscriptions
remain for all loaded sessions. Active scope and window focus are not inputs to pilot
decisions.

## Policy module (`build_pilot`)

Ownership: pure decide/auto-send rules in `crates/duckboard/src/build_pilot.rs` (and
shared pure helpers in `duckcore` when reordering/recommended parse is reusable).

```
is_hard_denylist(send):
  reject | revise | /ds-archive | /ds-codex | /ds-verify
  (optional leading / normalized)

decide after TurnComplete while Armed(mode):
  rank-1 missing → Disarm
  hard denylist → Disarm
  else → AutoSend(rank-1)   // N1: includes confirm, /ds-*, freeform decision tokens, A—… lines

  // mode Fast: still never auto-sends /ds-review only if we keep that product rule.
  // Settled: keep Fast vs Auto for /ds-review as today; N1 does not remove that.
```

**Fast vs Auto:** keep existing `/ds-review` only-in-auto rule. All other N1 freeform
rank-1 sends apply in both modes.

**Prose / next list order:** `next_actions` already comes from trailing meta `next` else
`prose_options`. Before decide, optionally **promote recommended** option to index 0:

- Body/label contains `recommended` / `(recommended)`
- Or trailing `Recommendation: **X**` / similar → letter X
- Else keep list order (A first)

Pilot auto-sends rank-1 after that promotion.

## Mid-turn chips

On `UserChoiceRequest` while pilot armed:

1. Resolve target option: label with recommended mark, else **first** option (M1).

2. Call existing `answer_user_choice(Selected { option_id })` + transcript settle path
   (`activate_fast_response` equivalent without requiring UI focus).

3. Stay armed; do not wait for Enter or focus.

If no options, leave awaiting (or clear) without inventing an answer — edge only.

## TurnComplete send path

Unchanged shape in `main.rs`:

1. `refresh_next_actions(true)`
2. `maybe_auto_send` → `pending_pilot_send`
3. After follow-up/queue priority: `send_prompt_text` on **that** session

Guarantees:

- Lookup by event key only
- No `active_scope` / visible-session guard on pilot hop
- SelectChange / area switch must not cancel S or clear `pilot`

If `send_agent_turn` cannot run (no handle / blocked model): **disarm**, append a short
**system** notice (e.g. pilot stopped: agent disconnected), clear pending hop. No infinite
armed-idle.

## Focus independence

```
| Event | Required behavior |
| --- | --- |
| Select another change | Background S keeps streaming/hopping |
| Area switch | Same |
| Window unfocused | Same (no focus gate in pilot or agent subscription) |
| Esc-Esc / composer cancel / thrash on S | Full disarm (+ cancel if streaming) as today |
```

Audit only: confirm no accidental cancel on selection paths; fix if found. Primary product
gap is policy widen (N1/R3/M1), not re-architecting subscriptions.

## Operator surface

Composer plaque **Build auto** / **Build fast** while armed — unchanged. No change-list
badge. Returning to the session shows plaque + transcript hops.

## Verification

```
| Layer | What |
| --- | --- |
| Unit `build_pilot` | N1 denylist vs freeform rank-1; Fast review rule; recommended promotion |
| Unit / interaction | Armed + UserChoiceRequest → first/recommended selected; pilot stays armed |
| Unit / path | Hop decision does not take active-scope; SelectChange does not disarm background pilot |
| Specs | Delta `chat/build-pilot` for unattended hop, R3/M1/N1, focus independence, failed-send disarm |
```

Reuse existing `@spec chat/build-pilot` suite; extend rather than replace stage-slash
tests.

## Failure and compatibility

- Existing sessions: pilot still process-local, default Off

- Promotion exploration→change: pilot arm still rides in-memory session (unchanged)

- `pilot_reactivate_on_error` setting: still not wired to auto re-arm after restart;
  failed mid-session hop uses disarm+notice only

- Queue editor and priming follow-up keep priority over pilot hop on the same TurnComplete
  (existing order)

## Settled choices

- R3 + M1 + N1 as the auto-run policy
- Focus independence as hard constraint, not a separate feature flag
- Silent forever-armed with dead handle rejected → disarm + notice
- No background list chrome in this cut
