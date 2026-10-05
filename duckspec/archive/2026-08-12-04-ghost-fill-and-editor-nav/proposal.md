# Ghost fill and editor line nav

Let the empty next-action ghost stage into the composer for edit, and make editor
Cmd-arrows match macOS line ends.

Today the grey next-action token is send-or-cycle only: empty Enter sends the active
token, Tab / Shift-Tab rotate when there are two or more, and nothing inserts the ghost
without starting a turn. That blocks the common path of accepting a suggestion, tweaking
it, then sending. Separately, TextEdit maps Cmd+← / → to word jumps instead of line
start/end.

```
empty composer + ghost
──────────────────────
Tab / →     ──►  fill active send into input (no turn)
Shift+Tab   ──►  cycle next actions forward only (wrap; ≥2)
Enter       ──►  send active send (unchanged)

TextEdit
────────
Cmd+← / →     ──►  line start / end
Option+← / →  ──►  word left / right
```

**Settled rules**

- Fill applies only when the composer is empty, a next-action ghost is armed, and
  slash-command completion is not consuming Tab.

- After fill, the input holds the token text with the caret at the end so the user can
  edit before Enter.

- Cycle drops reverse: Shift+Tab only advances with wrap; lists are small (at most a few
  next tokens).

- Tab no longer cycles next actions; its empty-ghost role is fill.

- Affordance chrome that currently marks “Tab cycles” must be updated so it does not lie
  about the binding.

- Slash-completion Tab accept while the popup is open stays as today.

**Boundaries**

- Enter remains the only empty-composer send path for next actions.

- No change to oneshot chips, fast-response activation, or Cmd-arrow chat landmarks when
  those keybinds apply outside editor focus.

- Scope is duckboard composer + TextEdit keyboard behavior, not agent harness protocol.
