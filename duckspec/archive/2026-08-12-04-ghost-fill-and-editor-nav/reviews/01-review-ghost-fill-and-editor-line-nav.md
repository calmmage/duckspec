# Review: Ghost fill and editor line nav

Change is ready to archive: fill/cycle/chrome, TextEdit line/word bindings, and
composer-focused Cmd+horizontal landmark yield match design and specs; audit is clean.

## Scope

Reviewed proposal, design (including Cmd+arrow ownership amendment), capability deltas/new
caps (`chat/default-prompts`, `chat/answer-landmarks`, `editor/line-word-nav`), steps
01–03, and implementation in `default_prompts`, `interaction`, `agent_chat`, `main`,
`keybinds`, and `text_edit`. Mechanical: `ds audit` ok, 10/10 scenarios linked.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| — | *(none open)* | — | — |
```

## Resolved concerns

**Cmd+←/→ double-use (landmarks + line nav).** While chat was open, app landmarks claimed
Cmd+horizontal arrows even with the composer focused, so line ends never reached TextEdit
and the user saw both behaviors. Settled rule: chat input focused → only in-input line
nav; chat open but input not focused → answer landmarks; Cmd+↑/↓ history jumps unchanged.
Implemented via horizontal eligibility (`chat_landmark_horizontal_allowed` /
`keybind_chat_landmark_horizontal`), design ownership section, answer-landmarks
eligibility rewrite, and step 03.

**Fill scenario depth.** Fill is covered by pure eligibility/token helpers (same style as
existing empty-Enter next-action tests), not a full handler integration test. Accepted as
consistent with the capability’s existing test layer.

**Content file editor vs landmarks.** With chat open and the composer not focused, Cmd+←/→
still jump answers rather than line-ending a content editor. Left intentional under the
settled “input focused vs not” rule; not expanded to content-column focus in this change.

## Outcome

No remaining accepted findings. Design, contracts, steps, and code are aligned for
archive.
