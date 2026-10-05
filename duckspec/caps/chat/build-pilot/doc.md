# Chat build pilot

Session pilot that arms from system `/build-auto` / `/build-fast`, kicks a rewritten
workflow turn, and keeps auto-advancing the session — rank-1 next actions, recommended or
first A–B–C options, and mid-turn choice chips — until a real stop, including while
another change or window is focused.

## Modes

```
| Mode | Arm command   | After each settled turn                                      |
| ---- | ------------- | ------------------------------------------------------------ |
| Auto | `/build-auto` | Auto-send rank-1 next unless hard-denylisted (includes review) |
| Fast | `/build-fast` | Same, but never auto-sends `/ds-review`                      |
```

Both modes auto-send bare `confirm`, allowlisted stage slashes, freeform rank-1 decision
tokens, and prose option lines. Neither auto-sends `/ds-archive`, `/ds-codex`,
`/ds-verify`, `reject`, or `revise`. Fast additionally never auto-sends `/ds-review`.

## Arm and kick

```
/build-auto|fast [args]
        │
        ├─ exploration → agent user message `/ds-explore [args]`
        ├─ change      → agent user message `/{lifecycle-head} [args]`
        └─ caps/codex/archived → no arm, no agent turn
```

The transcript user bubble is the rewritten kick, not the `/build-*` line. Successful kick
arms the mode for that chat session only (process memory — not written into the session
file).

## Auto-send loop

```
TurnComplete (non-priming, not user-cancelled)
        │
        ▼
refresh next actions (meta next, else prose A–B–C)
        │
        ▼
promote recommended option to rank-1 when marked
        │
        ├─ pilot off                         → idle
        ├─ rank-1 present and not denylisted → send as user message; stay armed
        └─ missing / hard-denylisted rank-1  → disarm (full off)
```

Hard denylist: `reject`, `revise`, `/ds-archive`, `/ds-codex`, `/ds-verify`, and in fast
mode `/ds-review`. After disarm, only a new `/build-auto` or `/build-fast` re-arms;
ordinary replies do not.

Recommended promotion: option text containing `recommended` / `(recommended)`, or a
trailing Recommendation letter line. Otherwise rank-1 stays the first list entry (A first
for prose runs).

## Stop controls

- **Esc-Esc** while streaming: cancel the turn and disarm.

- **Esc-Esc** while idle and armed: disarm only.

- **Cancel** of a streaming main turn (composer cancel or the same cancel path): disarm;
  that completion does not pilot-auto-send.

- **Missing or hard-denylisted rank-1:** disarm without auto-send.

- **Failed hop** (cannot start the decided send): disarm with a system notice.

Any of these leaves the pilot fully off until `/build-auto` or `/build-fast` again.

## Plaque

While armed, a quiet **Build auto** / **Build fast** plaque sits above the composer input.
It is chrome only — never a transcript line. Hiding the plaque means the pilot is off.

## Promotion

Exploration → change promotion keeps the same in-memory session state, including pilot
mode, so a pilot armed during explore continues on the new change under the same rules.

## Reactivate setting

Config key (chat settings): pilot reactivate after crash / restart / error. Default
**off**. The control is present so a later cut can wire recovery; enabling it does not yet
re-arm or restore pilot state.

## Mid-turn chips

While armed, a live mid-turn user-choice request is answered automatically: recommended
label if marked, else the first option. Answer uses the same in-band selected-option path
as clicking a chip. The pilot stays armed and does not wait for the session or window to
be focused.

## Focus independence

Pilot state and hop routing are per session. Selecting another change, switching area, or
leaving the window does not cancel the background turn, disarm the pilot, or skip an
otherwise-due hop. The operator can leave a piloted chat running and return later.

## Failed hop

If a hop is decided but the agent turn cannot start (for example no agent handle), the
pilot disarms and a short system notice records that the pilot stopped — never a silent
forever-armed wait.
