# Prose option next-actions

When an assistant turn ends with a usual multi-option A–B–C list in ordinary prose, the
empty composer should arm the same next-action UX already used for trailing `next` meta
cards: ghost text, Tab cycle, empty Enter to send.

## Problem

Agents often present choices as prose (review findings, design forks, simplicity options)
rather than a trailing `next` meta card. Review discussion is even steered toward ordinary
conversation, so those options never seed the composer. The user must retype a letter or
freeform reply even though the list is already on screen.

Next-action ghost + Tab already exists for lifecycle bootstrap, inherited seeds, and
trailing `next` cards. The gap is **detection of prose option lists**, not a new
interaction model.

## Direction

After a settled assistant turn, parse the last assistant message for a **trailing run** of
usual option lines (A / B / C and similar). When the run has at least two options, fill
the empty-composer next-action list from that run (cap three).

Ghost and send text are one string shaped as **letter + label + fuller option text**,
clipped to a single line so it fits the composer. Empty Enter sends that string as a
normal user message; Tab / Shift-Tab cycle as today.

Applies to **any** multi-option turn, not review-only and not stage-specific.

## Boundaries

```
| In | Out |
| --- | --- |
| Trailing A–B–C-style option runs in the last assistant message | Mid-turn structured user-choice (existing chips / in-band answer path) |
| Idle empty composer after the turn | Streaming / mid-turn |
| Reuse next-action ghost + Tab + Enter | A fourth choice surface or chip-only solution for this case |
| Explicit trailing `next` meta card still wins when present | Requiring templates to emit meta cards for every option list |
| Conservative parse (≥2 consecutive options, cap 3) | Heuristic scrape of every freeform list or numbered essay outline |
```

Rejected for this change: template-only discipline (agent must always emit `next`);
heuristic that overrides an explicit `next` card; building a separate chip UI for prose
options when the next-action path already matches the desired interaction.

## Success picture

User sees an A–B–C list in the transcript, empty composer shows a one-line ghost for the
first (or recommended) option with Tab affordance, Tab walks the options, Enter sends the
chosen line without retyping.
