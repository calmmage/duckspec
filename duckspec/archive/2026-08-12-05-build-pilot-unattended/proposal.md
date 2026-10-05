# Build pilot unattended

Keep an armed `/build-auto` / `/build-fast` pilot hopping on its session while the
operator is on another change or another window — unattended progress until a real stop
condition.

## Problem

Build pilot exists so a change can advance without pressing Enter after every turn. In
practice, auto-forward stops when the operator switches to another change tab or leaves
the duckboard window. That breaks the main use case: start pilot, work elsewhere, come
back further along.

## Intent

- Once armed, pilot hops on **that session** after each non-priming `TurnComplete` with a
  safe rank-1 next — whether or not the session is the active UI selection and whether or
  not the app window is focused

- Real stop conditions stay: Esc-Esc, user cancel, thrash cancel, missing/unsafe rank-1
  next

- Soft stalls (mid-turn user-choice, dead agent handle while plaque still shows armed) are
  diagnosed and fixed or made obvious — not silent “stopped for no reason”

- Focus change, area switch, and selecting another change **must not** cancel, disarm, or
  skip pilot hops for the background session

## Boundaries

```
| In | Out |
| --- | --- |
| Focus-independent continuous hop for an armed pilot | Auto-answering freeform questions or inventing next tokens |
| Background change / other window while duckboard stays running | Persist pilot across app quit / cold restart (reactivate setting remains separate) |
| Silent stall: armed plaque but no hop (e.g. dead handle) | Auto-archive, cost caps, NLP “critical issue” scanning |
| Spec + product clarity that unattended use is supported | Redesign of allowlist or mode rules (auto vs fast) |
```

## Grounding

Routing and agent subscriptions are already per-session and multi-scope; exploration found
**no intentional “must be focused” gate**. The change is to make unattended continuity
**true and testable** — fix any path that cancels, drops, or skips hops when focus leaves,
and close gaps where the pilot looks armed but cannot hop.
