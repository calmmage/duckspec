# Chat build pilot

Session pilot that arms from system `/build-auto` / `/build-fast`, kicks a rewritten
workflow turn, and auto-sends only safe trailing next tokens until disarmed.

## Modes

```
| Mode | Arm command   | After each settled turn                                      |
| ---- | ------------- | ------------------------------------------------------------ |
| Auto | `/build-auto` | Auto-send safe rank-1 next (includes `/ds-review` when safe) |
| Fast | `/build-fast` | Same, but never auto-sends `/ds-review`                      |
```

Both modes always treat `confirm` as safe. Neither auto-sends `/ds-archive`, `/ds-codex`,
or `/ds-verify`. Fast does not use a special mega-prompt for design+spec — it simply
auto-sends allowlisted stage tokens as they appear (so design handoff then spec is a
normal two-step auto sequence).

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
TurnComplete (non-priming)
        │
        ▼
refresh trailing next actions
        │
        ├─ pilot off                    → idle
        ├─ rank-1 safe for mode         → send as user message; stay armed
        └─ missing / unsafe rank-1      → disarm (full off)
```

Unsafe includes empty next list, `reject`, `revise`, freeform text, and non-allowlisted
slashes. After disarm, only a new `/build-auto` or `/build-fast` re-arms; ordinary replies
do not.

## Stop controls

- **Esc-Esc** while streaming: cancel the turn and disarm.

- **Esc-Esc** while idle and armed: disarm only.

- **Cancel** of a streaming main turn (composer cancel or the same cancel path): disarm;
  that completion does not pilot-auto-send.

- **Unsafe or missing next:** disarm without auto-send.

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
